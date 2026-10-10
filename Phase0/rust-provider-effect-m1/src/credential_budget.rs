//! Offline, deterministic admission simulation for #91 spec-3 M1 (not a broker).
//!
//! Credential-group keys MUST come from an independent, current authoritative
//! equivalence proof in a real adapter. GitHub logins, repository names and
//! different connector sessions do not establish separate rate-limit buckets.
//! This module has no network, clock, credential, durable queue, provider
//! dispatch or trusted authorization. Candidate selection is not a lease,
//! mutation authority, HTTP success, or a promise of available provider quota.
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Normal,
    Recovery,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub operation_id: String,
    pub credential_group: String,
    pub repository: String,
    pub tenant: String,
    pub target: String,
    /// Versioned operation class, not a client-supplied authority claim.
    pub operation_class: String,
    pub priority: Priority,
}

impl Request {
    fn valid(&self) -> bool {
        [
            &self.operation_id,
            &self.credential_group,
            &self.repository,
            &self.tenant,
            &self.target,
            &self.operation_class,
        ]
        .iter()
        .all(|field| crate::bounded_machine_id(field))
    }
}

/// Synthetic booleans for a negative/positive fixture, NEVER real authority.
#[derive(Clone, Copy, Debug)]
pub struct Boundary {
    pub domain_authority_current: bool,
    pub provider_fence_current: bool,
}

impl Boundary {
    fn current(self) -> bool {
        self.domain_authority_current && self.provider_fence_current
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidConfig,
    DuplicateGroup,
    UnknownGroup,
    DuplicateClass,
    UnknownClass,
    InvalidRequest,
    OperationIdentityConflict,
    QueueFull,
    OperationHistoryFull,
    CircuitOpen,
    StaleBoundary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// A simulated admission candidate; no actual provider call is sent.
    pub request: Request,
    pub estimated_remaining: u32,
    pub estimated_class_remaining: u32,
}

type TenantQueues = BTreeMap<String, VecDeque<Request>>;

struct ClassBudget {
    limit: u32,
    remaining: u32,
}

struct CredentialLane {
    limit: u32,
    remaining: u32,
    reserved_for_recovery: u32,
    max_pending: usize,
    /// Class-specific limits share the same aggregate credential budget.
    classes: BTreeMap<String, ClassBudget>,
    circuit_open: bool,
    pending_count: usize,
    normal: TenantQueues,
    recovery: TenantQueues,
    last_normal_tenant: Option<String>,
    last_recovery_tenant: Option<String>,
    last_class: Option<Priority>,
}

impl CredentialLane {
    fn new(limit: u32, reserved_for_recovery: u32, max_pending: usize) -> Self {
        Self {
            limit,
            remaining: limit,
            reserved_for_recovery,
            max_pending,
            classes: BTreeMap::new(),
            circuit_open: false,
            pending_count: 0,
            normal: TenantQueues::new(),
            recovery: TenantQueues::new(),
            last_normal_tenant: None,
            last_recovery_tenant: None,
            last_class: None,
        }
    }
}

/// Maximum number of operation identities retained by this *offline simulation*.
/// The journal never evicts completed identities: evicting one would allow a
/// changed payload to reuse an old operation ID without a replay conflict.
/// A real broker needs a durable, bounded receipt/reconciliation contract.
pub const MAX_TRACKED_OPERATIONS: usize = 1024;

/// All repositories sharing an effective credential group consume ONE budget.
/// Different group keys in fixtures model separately proved credential buckets;
/// this type does not establish their independence in the real world.
#[derive(Default)]
pub struct Scheduler {
    groups: BTreeMap<String, CredentialLane>,
    known_operations: BTreeMap<String, Request>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_group(
        &mut self,
        credential_group: &str,
        limit: u32,
        reserved_for_recovery: u32,
        max_pending: usize,
    ) -> Result<(), Error> {
        if !crate::bounded_machine_id(credential_group)
            || limit == 0
            || reserved_for_recovery > limit
            || max_pending == 0
        {
            return Err(Error::InvalidConfig);
        }
        if self.groups.contains_key(credential_group) {
            return Err(Error::DuplicateGroup);
        }
        self.groups.insert(
            credential_group.to_owned(),
            CredentialLane::new(limit, reserved_for_recovery, max_pending),
        );
        Ok(())
    }

    /// Register explicit synthetic operation-class limits for this effective credential.
    /// A real broker must source these classes and capacities from trusted policy,
    /// and must not interpret a fixture label as provider budget authority.
    pub fn register_operation_class(
        &mut self,
        group: &str,
        class: &str,
        limit: u32,
    ) -> Result<(), Error> {
        if !crate::bounded_machine_id(class) || limit == 0 {
            return Err(Error::InvalidConfig);
        }
        let lane = self.groups.get_mut(group).ok_or(Error::UnknownGroup)?;
        if lane.classes.contains_key(class) {
            return Err(Error::DuplicateClass);
        }
        lane.classes.insert(
            class.to_owned(),
            ClassBudget {
                limit,
                remaining: limit,
            },
        );
        Ok(())
    }

    /// Exact duplicate enqueue is a no-op. The same operation ID may NEVER
    /// silently change credential group, repository, target, tenant or class.
    pub fn enqueue(&mut self, request: Request) -> Result<bool, Error> {
        if !request.valid() {
            return Err(Error::InvalidRequest);
        }
        if let Some(existing) = self.known_operations.get(&request.operation_id) {
            return if existing == &request {
                Ok(false)
            } else {
                Err(Error::OperationIdentityConflict)
            };
        }
        let lane = self
            .groups
            .get_mut(&request.credential_group)
            .ok_or(Error::UnknownGroup)?;
        if !lane.classes.contains_key(&request.operation_class) {
            return Err(Error::UnknownClass);
        }
        if lane.pending_count >= lane.max_pending {
            return Err(Error::QueueFull);
        }
        // The pending queue bound is NOT a bound on the replay journal:
        // completed operations remain in known_operations indefinitely.
        // Fail closed when the simulation reaches its retained-history limit;
        // never evict an old identity to admit an apparently fresh replay.
        if self.known_operations.len() >= MAX_TRACKED_OPERATIONS {
            return Err(Error::OperationHistoryFull);
        }
        let queues = match request.priority {
            Priority::Normal => &mut lane.normal,
            Priority::Recovery => &mut lane.recovery,
        };
        queues
            .entry(request.tenant.clone())
            .or_default()
            .push_back(request.clone());
        lane.pending_count += 1;
        self.known_operations
            .insert(request.operation_id.clone(), request);
        Ok(true)
    }

    /// A throttle/ambiguous write disables simulated candidate selection.
    /// This explicit circuit does NOT auto-reopen on clock passage or refill.
    pub fn set_circuit_open(&mut self, group: &str, open: bool) -> Result<(), Error> {
        let lane = self.groups.get_mut(group).ok_or(Error::UnknownGroup)?;
        lane.circuit_open = open;
        Ok(())
    }

    /// Explicit test-only budget-window rollover; never resets circuit state
    /// and never reestablishes a lost domain or provider authority fence.
    pub fn replenish(&mut self, group: &str) -> Result<(), Error> {
        let lane = self.groups.get_mut(group).ok_or(Error::UnknownGroup)?;
        lane.remaining = lane.limit;
        for budget in lane.classes.values_mut() {
            budget.remaining = budget.limit;
        }
        Ok(())
    }

    pub fn remaining(&self, group: &str) -> Result<u32, Error> {
        self.groups
            .get(group)
            .map(|lane| lane.remaining)
            .ok_or(Error::UnknownGroup)
    }

    pub fn remaining_class(&self, group: &str, class: &str) -> Result<u32, Error> {
        self.groups
            .get(group)
            .ok_or(Error::UnknownGroup)?
            .classes
            .get(class)
            .map(|budget| budget.remaining)
            .ok_or(Error::UnknownClass)
    }

    pub fn pending(&self, group: &str) -> Result<usize, Error> {
        self.groups
            .get(group)
            .map(|lane| lane.pending_count)
            .ok_or(Error::UnknownGroup)
    }

    /// Deterministically select ONE *candidate*, never emit an effect.
    /// Recovery has access to reserved capacity; normal work does not. When
    /// both are eligible, the two classes alternate, and tenants round-robin
    /// independently inside each class to avoid sorted-key starvation.
    pub fn simulate_candidate(
        &mut self,
        group: &str,
        boundary: Boundary,
    ) -> Result<Option<Candidate>, Error> {
        let lane = self.groups.get_mut(group).ok_or(Error::UnknownGroup)?;
        if !boundary.current() {
            return Err(Error::StaleBoundary);
        }
        if lane.circuit_open {
            return Err(Error::CircuitOpen);
        }

        let can_recover = lane.remaining > 0 && has_eligible(&lane.recovery, &lane.classes);
        let can_normal = lane.remaining > lane.reserved_for_recovery
            && has_eligible(&lane.normal, &lane.classes);
        let class = match (can_normal, can_recover, lane.last_class) {
            (false, false, _) => return Ok(None),
            (true, false, _) => Priority::Normal,
            (false, true, _) => Priority::Recovery,
            (true, true, Some(Priority::Recovery)) => Priority::Normal,
            (true, true, _) => Priority::Recovery,
        };
        let request = match class {
            Priority::Normal => take_round_robin(
                &mut lane.normal,
                &mut lane.last_normal_tenant,
                &lane.classes,
            ),
            Priority::Recovery => take_round_robin(
                &mut lane.recovery,
                &mut lane.last_recovery_tenant,
                &lane.classes,
            ),
        }
        .expect("eligible queue has at least one request");
        lane.remaining -= 1;
        let class_budget = lane
            .classes
            .get_mut(&request.operation_class)
            .expect("selected class must be registered");
        class_budget.remaining -= 1;
        let estimated_class_remaining = class_budget.remaining;
        lane.pending_count -= 1;
        lane.last_class = Some(class);
        Ok(Some(Candidate {
            request,
            estimated_remaining: lane.remaining,
            estimated_class_remaining,
        }))
    }
}

/// A depleted endpoint class must not block unrelated operation classes.
/// This is a deterministic simulation of finite class quotas, not a quota
/// measurement, provider-side reservation, or an authorization to dispatch.
fn has_eligible(queues: &TenantQueues, classes: &BTreeMap<String, ClassBudget>) -> bool {
    queues.values().any(|pending| {
        pending.iter().any(|request| {
            classes
                .get(&request.operation_class)
                .is_some_and(|budget| budget.remaining > 0)
        })
    })
}

fn take_round_robin(
    queues: &mut TenantQueues,
    previous_tenant: &mut Option<String>,
    classes: &BTreeMap<String, ClassBudget>,
) -> Option<Request> {
    let tenants: Vec<String> = queues.keys().cloned().collect();
    if tenants.is_empty() {
        return None;
    }
    let start = previous_tenant
        .as_ref()
        .and_then(|last| tenants.iter().position(|tenant| tenant > last))
        .unwrap_or(0);
    for offset in 0..tenants.len() {
        let tenant = &tenants[(start + offset) % tenants.len()];
        let queue = queues.get_mut(tenant).expect("selected tenant exists");
        let eligible_position = queue.iter().position(|request| {
            classes
                .get(&request.operation_class)
                .is_some_and(|budget| budget.remaining > 0)
        });
        if let Some(position) = eligible_position {
            // Keep blocked requests intact; scheduling other classes cannot
            // bypass the exhausted class limit or lose FIFO order within it.
            let request = queue.remove(position);
            if queue.is_empty() {
                queues.remove(tenant);
            }
            *previous_tenant = Some(tenant.clone());
            return request;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT: Boundary = Boundary {
        domain_authority_current: true,
        provider_fence_current: true,
    };

    fn request(id: &str, group: &str, repo: &str, tenant: &str, priority: Priority) -> Request {
        Request {
            operation_id: id.into(),
            credential_group: group.into(),
            repository: repo.into(),
            tenant: tenant.into(),
            target: "issue:91/comments".into(),
            operation_class: "comment-create".into(),
            priority,
        }
    }

    fn scheduler(limit: u32, reserve: u32, pending: usize) -> Scheduler {
        let mut s = Scheduler::new();
        s.register_group("effective-credential-1", limit, reserve, pending)
            .unwrap();
        s.register_operation_class("effective-credential-1", "comment-create", limit)
            .unwrap();
        s
    }

    fn next(s: &mut Scheduler) -> Option<Candidate> {
        s.simulate_candidate("effective-credential-1", CURRENT)
            .unwrap()
    }

    #[test]
    fn c10_two_repositories_use_one_effective_credential_budget() {
        let mut s = scheduler(3, 0, 10);
        for id in 0..4 {
            s.enqueue(request(
                &format!("op-{id}"),
                "effective-credential-1",
                if id % 2 == 0 { "repo-a" } else { "repo-b" },
                "tenant-a",
                Priority::Normal,
            ))
            .unwrap();
        }
        for _ in 0..3 {
            assert!(next(&mut s).is_some());
        }
        assert!(next(&mut s).is_none());
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
        assert_eq!(s.remaining("effective-credential-1"), Ok(0));
        s.replenish("effective-credential-1").unwrap();
        assert!(next(&mut s).is_some());
        assert_eq!(s.pending("effective-credential-1"), Ok(0));
    }

    #[test]
    fn hundred_workers_do_not_starve_tenants_or_exceed_aggregate_budget() {
        let mut s = scheduler(50, 5, 100);
        for worker in 0..100 {
            s.enqueue(request(
                &format!("worker-{worker:03}"),
                "effective-credential-1",
                if worker % 2 == 0 { "repo-a" } else { "repo-b" },
                &format!("tenant-{:02}", worker % 10),
                Priority::Normal,
            ))
            .unwrap();
        }
        let mut first_tenants = Vec::new();
        for _ in 0..45 {
            let candidate = next(&mut s).unwrap();
            first_tenants.push(candidate.request.tenant);
        }
        assert_eq!(
            first_tenants[..10],
            (0..10)
                .map(|n| format!("tenant-{n:02}"))
                .collect::<Vec<_>>()
        );
        assert_eq!(s.remaining("effective-credential-1"), Ok(5));
        assert_eq!(s.pending("effective-credential-1"), Ok(55));
        assert!(next(&mut s).is_none());
    }

    #[test]
    fn recovery_can_consume_reserved_capacity_but_normal_cannot() {
        let mut s = scheduler(3, 1, 10);
        for i in 0..4 {
            s.enqueue(request(
                &format!("normal-{i}"),
                "effective-credential-1",
                "repo-a",
                "tenant-a",
                Priority::Normal,
            ))
            .unwrap();
        }
        for _ in 0..2 {
            assert_eq!(next(&mut s).unwrap().request.priority, Priority::Normal);
        }
        assert!(next(&mut s).is_none());
        s.enqueue(request(
            "safety-reconciliation",
            "effective-credential-1",
            "repo-b",
            "tenant-b",
            Priority::Recovery,
        ))
        .unwrap();
        assert_eq!(next(&mut s).unwrap().request.priority, Priority::Recovery);
        assert_eq!(s.remaining("effective-credential-1"), Ok(0));
        assert!(next(&mut s).is_none());
    }

    #[test]
    fn recovery_and_normal_share_capacity_without_class_starvation() {
        let mut s = scheduler(6, 0, 12);
        for i in 0..3 {
            for priority in [Priority::Normal, Priority::Recovery] {
                s.enqueue(request(
                    &format!("{priority:?}-{i}"),
                    "effective-credential-1",
                    "repo-a",
                    "tenant-a",
                    priority,
                ))
                .unwrap();
            }
        }
        let order: Vec<Priority> = (0..6)
            .map(|_| next(&mut s).unwrap().request.priority)
            .collect();
        assert_eq!(
            order,
            vec![
                Priority::Recovery,
                Priority::Normal,
                Priority::Recovery,
                Priority::Normal,
                Priority::Recovery,
                Priority::Normal
            ]
        );
    }

    #[test]
    fn throttle_does_not_consume_estimated_budget_or_discard_pending_work() {
        let mut s = scheduler(2, 0, 5);
        s.enqueue(request(
            "op-1",
            "effective-credential-1",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        ))
        .unwrap();
        s.set_circuit_open("effective-credential-1", true).unwrap();
        assert!(matches!(
            s.simulate_candidate("effective-credential-1", CURRENT),
            Err(Error::CircuitOpen)
        ));
        s.replenish("effective-credential-1").unwrap();
        assert!(matches!(
            s.simulate_candidate("effective-credential-1", CURRENT),
            Err(Error::CircuitOpen)
        ));
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
        assert_eq!(s.remaining("effective-credential-1"), Ok(2));
        s.set_circuit_open("effective-credential-1", false).unwrap();
        assert!(next(&mut s).is_some());
    }

    #[test]
    fn revoked_domain_or_provider_boundary_cannot_admit_even_recovery() {
        let mut s = scheduler(2, 1, 5);
        s.enqueue(request(
            "revoked",
            "effective-credential-1",
            "repo-a",
            "tenant-a",
            Priority::Recovery,
        ))
        .unwrap();
        for boundary in [
            Boundary {
                domain_authority_current: false,
                provider_fence_current: true,
            },
            Boundary {
                domain_authority_current: true,
                provider_fence_current: false,
            },
        ] {
            assert!(matches!(
                s.simulate_candidate("effective-credential-1", boundary),
                Err(Error::StaleBoundary)
            ));
        }
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
        assert_eq!(s.remaining("effective-credential-1"), Ok(2));
    }

    #[test]
    fn duplicated_operation_id_cannot_cross_repositories_or_credential_groups() {
        let mut s = scheduler(2, 0, 5);
        s.register_group("credential-two", 2, 0, 5).unwrap();
        let original = request(
            "same-id",
            "effective-credential-1",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        );
        assert_eq!(s.enqueue(original.clone()), Ok(true));
        assert_eq!(s.enqueue(original.clone()), Ok(false));
        for variant in 0..5 {
            let mut changed = original.clone();
            match variant {
                0 => changed.repository = "repo-b".into(),
                1 => changed.credential_group = "credential-two".into(),
                2 => changed.target = "issue:92".into(),
                3 => changed.operation_class = "ref-update".into(),
                _ => changed.priority = Priority::Recovery,
            }
            assert_eq!(s.enqueue(changed), Err(Error::OperationIdentityConflict));
        }
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
        assert_eq!(s.pending("credential-two"), Ok(0));
    }

    #[test]
    fn queue_bound_rejects_excess_without_stealing_a_slot() {
        let mut s = scheduler(2, 0, 1);
        let a = request(
            "a",
            "effective-credential-1",
            "repo-a",
            "t",
            Priority::Normal,
        );
        assert_eq!(s.enqueue(a.clone()), Ok(true));
        assert_eq!(s.enqueue(a), Ok(false));
        assert_eq!(
            s.enqueue(request(
                "b",
                "effective-credential-1",
                "repo-b",
                "t",
                Priority::Normal,
            )),
            Err(Error::QueueFull)
        );
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
    }

    #[test]
    fn malformed_names_and_unknown_groups_are_not_silently_accepted() {
        let mut s = scheduler(2, 0, 2);
        for bad in ["", "   ", "a\nb"] {
            let candidate = request(
                bad,
                "effective-credential-1",
                "repo-a",
                "t",
                Priority::Normal,
            );
            assert_eq!(s.enqueue(candidate), Err(Error::InvalidRequest));
        }
        assert_eq!(
            s.enqueue(request("unknown", "other", "repo-a", "t", Priority::Normal)),
            Err(Error::UnknownGroup)
        );
        assert_eq!(
            s.register_group("other", 1, 2, 10),
            Err(Error::InvalidConfig)
        );
        assert_eq!(
            s.register_group("other", 1, 0, 0),
            Err(Error::InvalidConfig)
        );
    }

    #[test]
    fn independently_proved_groups_are_separate_but_login_is_not_the_key() {
        let mut s = scheduler(1, 0, 2);
        s.register_group("effective-credential-2", 1, 0, 2).unwrap();
        s.register_operation_class("effective-credential-2", "comment-create", 1)
            .unwrap();
        for (id, group) in [
            ("one", "effective-credential-1"),
            ("two", "effective-credential-2"),
        ] {
            s.enqueue(request(
                id,
                group,
                "same-repo",
                "same-tenant",
                Priority::Normal,
            ))
            .unwrap();
        }
        assert!(next(&mut s).is_some());
        assert!(next(&mut s).is_none());
        assert!(s
            .simulate_candidate("effective-credential-2", CURRENT)
            .unwrap()
            .is_some());
    }
    #[test]
    fn opaque_budget_keys_are_bounded_before_queue_or_group_mutation() {
        let mut s = scheduler(4, 1, 10);
        for bad in [
            "with space".to_owned(),
            "a\tb".to_owned(),
            "hidden\u{202e}suffix".to_owned(),
            "é".to_owned(),
            "x".repeat(513),
        ] {
            assert_eq!(s.register_group(&bad, 4, 1, 10), Err(Error::InvalidConfig));
            for field in 0..6 {
                let mut invalid = request(
                    "safe-id",
                    "effective-credential-1",
                    "repo-a",
                    "tenant-a",
                    Priority::Normal,
                );
                match field {
                    0 => invalid.operation_id = bad.clone(),
                    1 => invalid.credential_group = bad.clone(),
                    2 => invalid.repository = bad.clone(),
                    3 => invalid.tenant = bad.clone(),
                    4 => invalid.target = bad.clone(),
                    _ => invalid.operation_class = bad.clone(),
                }
                assert_eq!(s.enqueue(invalid), Err(Error::InvalidRequest));
            }
        }
        assert_eq!(s.pending("effective-credential-1"), Ok(0));
        assert_eq!(s.remaining("effective-credential-1"), Ok(4));

        let mut maximum = request(
            "safe-id",
            "effective-credential-1",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        );
        maximum.operation_id = "x".repeat(512);
        assert_eq!(s.enqueue(maximum), Ok(true));
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
    }
    #[test]
    fn endpoint_exhaustion_does_not_spend_aggregate_budget_or_block_other_classes() {
        let mut s = scheduler(4, 0, 8);
        s.register_operation_class("effective-credential-1", "ref-update", 1)
            .unwrap();
        let mut a = request(
            "ref-a",
            "effective-credential-1",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        );
        a.operation_class = "ref-update".into();
        let mut b = a.clone();
        b.operation_id = "ref-b".into();
        s.enqueue(a).unwrap();
        s.enqueue(b).unwrap();
        s.enqueue(request(
            "comment-a",
            "effective-credential-1",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        ))
        .unwrap();

        assert_eq!(next(&mut s).unwrap().request.operation_id, "ref-a");
        assert_eq!(
            s.remaining_class("effective-credential-1", "ref-update"),
            Ok(0)
        );
        assert_eq!(s.remaining("effective-credential-1"), Ok(3));
        // The second ref is still pending, but it must not head-of-line block
        // an authorized separate class in the same tenant's FIFO.
        let selected = next(&mut s).unwrap();
        assert_eq!(selected.request.operation_id, "comment-a");
        assert_eq!(selected.estimated_class_remaining, 3);
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
        assert_eq!(s.remaining("effective-credential-1"), Ok(2));
        assert!(next(&mut s).is_none());
        s.replenish("effective-credential-1").unwrap();
        assert_eq!(next(&mut s).unwrap().request.operation_id, "ref-b");
    }

    #[test]
    fn endpoint_limit_is_shared_across_repositories_and_tenants() {
        let mut s = scheduler(5, 1, 10);
        s.register_operation_class("effective-credential-1", "ref-update", 1)
            .unwrap();
        for (id, repo, tenant) in [("r1", "repo-a", "tenant-a"), ("r2", "repo-b", "tenant-b")] {
            let mut req = request(
                id,
                "effective-credential-1",
                repo,
                tenant,
                Priority::Recovery,
            );
            req.operation_class = "ref-update".into();
            s.enqueue(req).unwrap();
        }
        assert_eq!(next(&mut s).unwrap().request.operation_id, "r1");
        assert_eq!(
            s.remaining_class("effective-credential-1", "ref-update"),
            Ok(0)
        );
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
        assert_eq!(s.remaining("effective-credential-1"), Ok(4));
        assert!(next(&mut s).is_none());
    }

    #[test]
    fn unknown_or_invalid_class_fails_closed_without_enqueuing() {
        let mut s = scheduler(3, 0, 3);
        let mut req = request(
            "unknown",
            "effective-credential-1",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        );
        req.operation_class = "not-configured".into();
        assert_eq!(s.enqueue(req), Err(Error::UnknownClass));
        assert_eq!(s.pending("effective-credential-1"), Ok(0));
        assert_eq!(
            s.remaining_class("effective-credential-1", "unknown"),
            Err(Error::UnknownClass)
        );
        assert_eq!(
            s.register_operation_class("effective-credential-1", "bad class", 1),
            Err(Error::InvalidConfig)
        );
        assert_eq!(
            s.register_operation_class("effective-credential-1", "ref-update", 0),
            Err(Error::InvalidConfig)
        );
        assert_eq!(
            s.register_operation_class("effective-credential-1", "ref-update", 2),
            Ok(())
        );
        assert_eq!(
            s.register_operation_class("effective-credential-1", "ref-update", 2),
            Err(Error::DuplicateClass)
        );
        assert_eq!(
            s.remaining_class("effective-credential-1", "ref-update"),
            Ok(2)
        );
        assert_eq!(
            s.remaining_class("missing-group", "ref-update"),
            Err(Error::UnknownGroup)
        );
    }

    #[test]
    fn completed_history_cannot_bypass_queue_bounds_or_reuse_operation_ids() {
        let mut s = Scheduler::new();
        let limit = (MAX_TRACKED_OPERATIONS + 1) as u32;
        // A queue of one can still admit arbitrarily many distinct IDs over
        // time unless completed-operation history is independently bounded.
        s.register_group("group-history", limit, 0, 1).unwrap();
        s.register_operation_class("group-history", "comment-create", limit)
            .unwrap();
        for index in 0..MAX_TRACKED_OPERATIONS {
            let original = request(
                &format!("history-{index:04}"),
                "group-history",
                "repo-a",
                "tenant-a",
                Priority::Normal,
            );
            assert_eq!(s.enqueue(original.clone()), Ok(true));
            let selected = s
                .simulate_candidate("group-history", CURRENT)
                .unwrap()
                .unwrap();
            assert_eq!(selected.request, original);
        }
        assert_eq!(s.pending("group-history"), Ok(0));

        // A historical identical operation is still a no-op at capacity.
        let old = request(
            "history-0000",
            "group-history",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        );
        assert_eq!(s.enqueue(old.clone()), Ok(false));
        let mut conflicting = old;
        conflicting.target = "issue:changed".into();
        assert_eq!(
            s.enqueue(conflicting),
            Err(Error::OperationIdentityConflict)
        );

        // Fresh IDs fail closed: no eviction, queue insertion, or budget use.
        let fresh = request(
            "history-over-capacity",
            "group-history",
            "repo-a",
            "tenant-a",
            Priority::Normal,
        );
        assert_eq!(s.enqueue(fresh.clone()), Err(Error::OperationHistoryFull));
        assert_eq!(s.pending("group-history"), Ok(0));
        assert_eq!(s.remaining("group-history"), Ok(1));
        s.replenish("group-history").unwrap();
        assert_eq!(s.enqueue(fresh), Err(Error::OperationHistoryFull));
        assert_eq!(s.pending("group-history"), Ok(0));
    }
}
