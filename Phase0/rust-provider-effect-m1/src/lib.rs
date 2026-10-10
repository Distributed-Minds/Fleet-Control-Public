//! M1 offline provider-effect feasibility and ambiguous-create attribution model.
//! This crate has NO network access, credentials, repository-write adapter or
//! authority to admit an actual provider mutation. It is not the #91 broker.
use std::collections::BTreeSet;

pub const EMBEDDED_MANIFEST: &str = include_str!("../manifest/provider_effects_v1.tsv");
pub const MANIFEST_HEADER: &str = "version\toperation\ttransport\tpositive_ack\tlost_ack_attribution\tautomatic_replay\ttarget_cas\tproof_root";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Unknown,
    Unsupported,
    SupportedConditionally,
}

impl Capability {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "UNSUPPORTED" => Some(Self::Unsupported),
            "SUPPORTED_CONDITIONALLY" => Some(Self::SupportedConditionally),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEffectCapability {
    pub operation: String,
    pub transport: String,
    pub positive_ack: Capability,
    pub lost_ack_attribution: Capability,
    pub automatic_replay: Capability,
    pub target_cas: Capability,
    pub proof_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    InvalidHeader,
    Empty,
    InvalidRow(usize),
    DuplicateOperation(usize),
    UnsupportedPositiveClaim(usize),
    UnverifiedProofRoot(usize),
    IncompleteMatrix,
}

/// Parse a *data-only* support matrix. A row is not runtime permission.
/// Unknown and unsupported are distinct; proof-free SUPPORT claims are invalid.
/// No parser result can upgrade an operation into current effect authorization.
fn known_v1_effect(operation: &str, transport: &str) -> bool {
    match transport {
        "github-rest" => matches!(
            operation,
            "create_issue"
                | "create_comment"
                | "create_pr"
                | "edit_metadata"
                | "contents_update"
                | "move_ref"
                | "merge_pr"
                | "rerun_workflow"
        ),
        "github-connected-chat" => matches!(operation, "create_comment" | "merge_pr"),
        _ => false,
    }
}

pub fn parse_manifest(contents: &str) -> Result<Vec<ProviderEffectCapability>, ManifestError> {
    let mut lines = contents.lines();
    if lines.next() != Some(MANIFEST_HEADER) {
        return Err(ManifestError::InvalidHeader);
    }
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    for (offset, line) in lines.enumerate() {
        let lineno = offset + 2;
        if line.is_empty() {
            return Err(ManifestError::InvalidRow(lineno));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 8
            || fields[0] != "1"
            || !known_v1_effect(fields[1], fields[2])
            || fields[7].is_empty()
            || !matches!(
                fields[1],
                "create_issue"
                    | "create_comment"
                    | "create_pr"
                    | "edit_metadata"
                    | "contents_update"
                    | "move_ref"
                    | "merge_pr"
                    | "rerun_workflow"
            )
        {
            return Err(ManifestError::InvalidRow(lineno));
        }
        let statuses: Vec<Capability> = fields[3..7]
            .iter()
            .map(|s| Capability::parse(s).ok_or(ManifestError::InvalidRow(lineno)))
            .collect::<Result<_, _>>()?;
        if statuses.contains(&Capability::SupportedConditionally) && fields[7] == "none" {
            return Err(ManifestError::UnsupportedPositiveClaim(lineno));
        }
        // V1 has no independently authenticated transport proof root. A text
        // label (including a plausible receipt name) is not causal evidence.
        if fields[7] != "none" {
            return Err(ManifestError::UnverifiedProofRoot(lineno));
        }
        if !seen.insert((fields[1], fields[2])) {
            return Err(ManifestError::DuplicateOperation(lineno));
        }
        output.push(ProviderEffectCapability {
            operation: fields[1].to_owned(),
            transport: fields[2].to_owned(),
            positive_ack: statuses[0],
            lost_ack_attribution: statuses[1],
            automatic_replay: statuses[2],
            target_cas: statuses[3],
            proof_root: fields[7].to_owned(),
        });
    }
    if output.is_empty() {
        return Err(ManifestError::Empty);
    }
    // Ten unique allowlisted pairs constitute the complete V1 denominator.
    // A truncated manifest cannot silently remove an unsupported operation.
    if output.len() != 10 {
        return Err(ManifestError::IncompleteMatrix);
    }
    Ok(output)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationIdentity {
    pub operation_id: String,
    pub attempt_id: String,
    pub repository_incarnation: String,
    pub parent_incarnation: String,
    pub credential_group_incarnation: String,
    pub payload_digest: String,
}

impl OperationIdentity {
    fn is_valid(&self) -> bool {
        [
            &self.operation_id,
            &self.attempt_id,
            &self.repository_incarnation,
            &self.parent_incarnation,
            &self.credential_group_incarnation,
            &self.payload_digest,
        ]
        .iter()
        .all(|s| !s.is_empty())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Prepared,
    Dispatching,
    EffectUnknown,
    Reconciling,
    ManualHold,
    RemoteProven,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelError {
    InvalidIdentity,
    InvalidState,
    NotAuthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteObject {
    pub id: u64,
    pub repository_incarnation: String,
    pub parent_incarnation: String,
    pub credential_group_incarnation: String,
    pub payload_digest: String,
}

/// Fields are intentionally PRIVATE. There is no production constructor.
/// An external caller cannot invent a trusted receipt through this library.
/// A future adapter must earn an audited, provider-specific verification path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustedAttemptReceipt {
    operation_id: String,
    attempt_id: String,
    remote_id: u64,
    repository_incarnation: String,
    parent_incarnation: String,
    credential_group_incarnation: String,
    payload_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateOperation {
    identity: OperationIdentity,
    state: State,
    transmitted_calls: u32,
    unattributed_remote_present: bool,
    transitions: Vec<State>,
}

impl CreateOperation {
    pub fn new(identity: OperationIdentity) -> Result<Self, ModelError> {
        if !identity.is_valid() {
            return Err(ModelError::InvalidIdentity);
        }
        Ok(Self {
            identity,
            state: State::Prepared,
            transmitted_calls: 0,
            unattributed_remote_present: false,
            transitions: vec![State::Prepared],
        })
    }

    fn advance(&mut self, next: State) {
        self.state = next;
        self.transitions.push(next);
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn transitions(&self) -> &[State] {
        &self.transitions
    }

    pub fn transmitted_calls(&self) -> u32 {
        self.transmitted_calls
    }

    pub fn unattributed_remote_present(&self) -> bool {
        self.unattributed_remote_present
    }

    /// Fake dispatch only; this never calls a provider.
    pub fn dispatch(
        &mut self,
        domain_authority_current: bool,
        provider_fence_current: bool,
    ) -> Result<(), ModelError> {
        if self.state != State::Prepared {
            return Err(ModelError::InvalidState);
        }
        if !domain_authority_current || !provider_fence_current {
            return Err(ModelError::NotAuthorized);
        }
        self.transmitted_calls += 1;
        self.advance(State::Dispatching);
        Ok(())
    }

    pub fn lose_ack(&mut self) -> Result<(), ModelError> {
        if self.state != State::Dispatching {
            return Err(ModelError::InvalidState);
        }
        self.advance(State::EffectUnknown);
        Ok(())
    }

    /// Complete enumeration is necessary, not sufficient. Search matches,
    /// identical visible actor/marker/body, and a claimed local mutex cannot
    /// create attribution. Only an internally constructed trusted receipt
    /// tied to the exact attempt can establish it in this offline model.
    pub fn reconcile(
        &mut self,
        objects: &[RemoteObject],
        enumeration_complete: bool,
        authority_and_provider_current: bool,
        receipt: Option<&TrustedAttemptReceipt>,
    ) -> Result<State, ModelError> {
        if self.state != State::EffectUnknown {
            return Err(ModelError::InvalidState);
        }
        self.advance(State::Reconciling);
        self.unattributed_remote_present = !objects.is_empty();

        let attributable = if enumeration_complete && authority_and_provider_current {
            receipt.is_some_and(|r| {
                r.operation_id == self.identity.operation_id
                    && r.attempt_id == self.identity.attempt_id
                    && r.repository_incarnation == self.identity.repository_incarnation
                    && r.parent_incarnation == self.identity.parent_incarnation
                    && r.credential_group_incarnation == self.identity.credential_group_incarnation
                    && r.payload_digest == self.identity.payload_digest
                    && objects
                        .iter()
                        .filter(|o| {
                            o.id == r.remote_id
                                && o.repository_incarnation == r.repository_incarnation
                                && o.parent_incarnation == r.parent_incarnation
                                && o.credential_group_incarnation == r.credential_group_incarnation
                                && o.payload_digest == r.payload_digest
                        })
                        .count()
                        == 1
            })
        } else {
            false
        };
        self.advance(if attributable {
            State::RemoteProven
        } else {
            State::ManualHold
        });
        Ok(self.state)
    }

    /// Unknown outcomes and terminal manual holds never permit automatic
    /// duplicate creates; no generic recovery path silently retries.
    pub fn automatic_retry_allowed(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> OperationIdentity {
        OperationIdentity {
            operation_id: "op-1".into(),
            attempt_id: "attempt-1".into(),
            repository_incarnation: "repo-1".into(),
            parent_incarnation: "issue-91-v3".into(),
            credential_group_incarnation: "credential-1".into(),
            payload_digest: "sha256:abc".into(),
        }
    }

    fn remote() -> RemoteObject {
        RemoteObject {
            id: 123,
            repository_incarnation: "repo-1".into(),
            parent_incarnation: "issue-91-v3".into(),
            credential_group_incarnation: "credential-1".into(),
            payload_digest: "sha256:abc".into(),
        }
    }

    fn trusted_receipt() -> TrustedAttemptReceipt {
        TrustedAttemptReceipt {
            operation_id: "op-1".into(),
            attempt_id: "attempt-1".into(),
            remote_id: 123,
            repository_incarnation: "repo-1".into(),
            parent_incarnation: "issue-91-v3".into(),
            credential_group_incarnation: "credential-1".into(),
            payload_digest: "sha256:abc".into(),
        }
    }

    fn lost_ack_operation() -> CreateOperation {
        let mut op = CreateOperation::new(identity()).unwrap();
        op.dispatch(true, true).unwrap();
        op.lose_ack().unwrap();
        op
    }

    #[test]
    fn c13_same_credential_exact_spoof_is_not_attribution() {
        let mut op = lost_ack_operation();
        assert_eq!(
            op.reconcile(&[remote()], true, true, None),
            Ok(State::ManualHold)
        );
        assert_eq!(op.transmitted_calls(), 1);
        assert!(op.unattributed_remote_present());
        assert_eq!(
            op.transitions(),
            &[
                State::Prepared,
                State::Dispatching,
                State::EffectUnknown,
                State::Reconciling,
                State::ManualHold
            ]
        );
        assert!(!op.automatic_retry_allowed());
    }

    #[test]
    fn c13_two_matching_remote_objects_still_hold() {
        let mut op = lost_ack_operation();
        let mut second = remote();
        second.id = 124;
        assert_eq!(
            op.reconcile(&[remote(), second], true, true, None),
            Ok(State::ManualHold)
        );
    }

    #[test]
    fn c14_exact_internal_receipt_and_complete_readback_can_prove() {
        let mut op = lost_ack_operation();
        assert_eq!(
            op.reconcile(&[remote()], true, true, Some(&trusted_receipt())),
            Ok(State::RemoteProven)
        );
        assert_eq!(op.transmitted_calls(), 1);
        assert!(!op.automatic_retry_allowed());
    }

    #[test]
    fn c14_wrong_attempt_or_resource_incarnation_does_not_prove() {
        let mut bad = trusted_receipt();
        bad.attempt_id = "another-attempt".into();
        let mut op = lost_ack_operation();
        assert_eq!(
            op.reconcile(&[remote()], true, true, Some(&bad)),
            Ok(State::ManualHold)
        );

        let mut op = lost_ack_operation();
        let mut bad_object = remote();
        bad_object.repository_incarnation = "restored-repo".into();
        assert_eq!(
            op.reconcile(&[bad_object], true, true, Some(&trusted_receipt())),
            Ok(State::ManualHold)
        );
    }

    #[test]
    fn c15_complete_listing_without_receipt_is_not_causal_proof() {
        for complete in [true, false] {
            let mut op = lost_ack_operation();
            assert_eq!(
                op.reconcile(&[remote()], complete, true, None),
                Ok(State::ManualHold)
            );
            assert!(!op.automatic_retry_allowed());
        }
    }

    #[test]
    fn c16_lost_provider_currentness_invalidates_receipt() {
        let mut op = lost_ack_operation();
        assert_eq!(
            op.reconcile(&[remote()], true, false, Some(&trusted_receipt())),
            Ok(State::ManualHold)
        );
    }

    #[test]
    fn c17_manual_hold_is_monotonic_and_distinct_new_operation_is_required() {
        let mut op = lost_ack_operation();
        op.reconcile(&[remote()], true, true, None).unwrap();
        assert_eq!(
            op.reconcile(&[remote()], true, true, Some(&trusted_receipt())),
            Err(ModelError::InvalidState)
        );
        assert_eq!(op.dispatch(true, true), Err(ModelError::InvalidState));
        let mut other = identity();
        other.operation_id = "human-authorized-fresh-op2".into();
        let fresh = CreateOperation::new(other).unwrap();
        assert_eq!(fresh.state(), State::Prepared);
        assert_eq!(op.state(), State::ManualHold);
    }

    #[test]
    fn disallowed_or_repeated_dispatch_does_not_emit_an_attempt() {
        let mut op = CreateOperation::new(identity()).unwrap();
        assert_eq!(op.dispatch(false, true), Err(ModelError::NotAuthorized));
        assert_eq!(op.dispatch(true, false), Err(ModelError::NotAuthorized));
        assert_eq!(op.transmitted_calls(), 0);
        op.dispatch(true, true).unwrap();
        assert_eq!(op.dispatch(true, true), Err(ModelError::InvalidState));
        assert_eq!(op.transmitted_calls(), 1);
    }
    #[test]
    fn m1_manifest_rejects_unregistered_transport_and_effect_pairs() {
        let row = "1\tcreate_comment\tgithub-rest\tUNKNOWN\tUNSUPPORTED\tUNSUPPORTED\tUNKNOWN\tnone";
        let unknown_transport = row.replace("github-rest", "unknown-provider");
        assert_eq!(
            parse_manifest(&format!("{MANIFEST_HEADER}\\n{unknown_transport}\\n")),
            Err(ManifestError::InvalidRow(2))
        );
        let unregistered_pair = row.replace("create_comment", "contents_update")
            .replace("github-rest", "github-connected-chat");
        assert_eq!(
            parse_manifest(&format!("{MANIFEST_HEADER}\\n{unregistered_pair}\\n")),
            Err(ManifestError::InvalidRow(2))
        );
    }

    #[test]
    fn m1_manifest_rejects_unverified_proof_roots_even_without_support_claim() {
        let row = "1\tcreate_comment\tgithub-rest\tUNKNOWN\tUNSUPPORTED\tUNSUPPORTED\tUNKNOWN\tsearch-matched-id";
        assert_eq!(
            parse_manifest(&format!("{MANIFEST_HEADER}\\n{row}\\n")),
            Err(ManifestError::UnverifiedProofRoot(2))
        );
        let unsafe_positive = row.replace("UNSUPPORTED", "SUPPORTED_CONDITIONALLY");
        assert_eq!(
            parse_manifest(&format!("{MANIFEST_HEADER}\\n{unsafe_positive}\\n")),
            Err(ManifestError::UnverifiedProofRoot(2))
        );
    }

    #[test]
    fn m1_manifest_requires_all_ten_registered_effect_transport_pairs() {
        assert_eq!(
            parse_manifest(EMBEDDED_MANIFEST).unwrap().len(),
            10
        );
        let omitted = EMBEDDED_MANIFEST
            .lines()
            .filter(|line| !line.starts_with("1\tcreate_issue\t"))
            .collect::<Vec<_>>()
            .join("\\n");
        assert_eq!(
            parse_manifest(&omitted),
            Err(ManifestError::IncompleteMatrix)
        );
    }

    #[test]
    fn m1_manifest_rejects_blank_data_rows_instead_of_skipping_them() {
        let malformed = EMBEDDED_MANIFEST.replacen(
            "\\n1\tcreate_issue\t",
            "\\n\\n1\tcreate_issue\t",
            1,
        );
        assert_eq!(
            parse_manifest(&malformed),
            Err(ManifestError::InvalidRow(2))
        );
    }

}
