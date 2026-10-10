//! Offline, synthetic-only contribution-admission reducer for issue #159, spec 3.
//!
//! There is intentionally no provider adapter, credential handling, signing,
//! human-assent capture, publication method or production trust root.
//! SyntheticFixture represents a test harness assertion, NOT human consent.
use std::collections::BTreeMap;

mod sha256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Claim {
    Absent,
    CallerClaim,
    SyntheticFixture,
    Disputed,
    Revoked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    OfflineSimulation,
    InteractiveOnly,
    Unverified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderObservation {
    NoEffectAttempted,
    AcknowledgementLost,
    RateLimited,
    MatchingObjectUnattributed,
    ConflictingObject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    MalformedIdentity,
    MissingHumanAct,
    MissingDco,
    StaleTerms,
    UnclearRights,
    StaleAuthority,
    MovedTarget,
    ChangedReviewedContent,
    UnreviewedTransformation,
    MissingReview,
    MissingTaskAuthorization,
    ConflictingRemoteEffect,
    AmbiguousCredential,
    UnverifiedHistory,
    UnverifiedContext,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    ReviewableInSimulation,
    Blocked(Reason),
    HumanReviewRequired(Reason),
    ReconcileOriginalAttempt,
    Unknown(Reason),
}

/// The caller chooses these *synthetic* proof values. None has production
/// authority, even when an otherwise positive test reaches ReviewableInSimulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub human_act: Claim,
    pub dco_declaration: Claim,
    pub terms_assent: Claim,
    pub code_rights: Claim,
    pub asset_rights: Claim,
    pub employer_rights: Claim,
    pub reviewer_decision: Claim,
    pub task_authority: Claim,
    pub transformation_map: Claim,
    pub historical_lineage: Claim,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    pub schema_version: u32,
    pub repository_id: u64,
    pub repository_incarnation: String,
    pub source_commit: String,
    pub intended_target_head: String,
    pub observed_target_head: String,
    pub input_digest: String,
    pub output_digest: String,
    pub reviewed_output_digest: String,
    pub terms_version: u64,
    pub current_terms_version: u64,
    pub authority_generation: u64,
    pub current_authority_generation: u64,
    pub transformation_occurred: bool,
    pub credential_identity_ambiguous: bool,
    pub context: Context,
    pub provider_observation: ProviderObservation,
    pub evidence: Evidence,
    // These visible bits deliberately cannot become proof of human assent.
    pub github_signature_verified: bool,
    pub dco_bot_exempt: bool,
}

fn simulated(value: Claim) -> bool {
    value == Claim::SyntheticFixture
}

/// Fail-closed, nonpublishing reduction. Never returns production permission.
pub fn evaluate(input: &Admission) -> Decision {
    use Decision::{Blocked, HumanReviewRequired, ReconcileOriginalAttempt, Unknown};
    if input.schema_version != 1
        || input.repository_id == 0
        || input.repository_incarnation.is_empty()
        || input.source_commit.is_empty()
        || input.intended_target_head.is_empty()
        || input.input_digest.is_empty()
        || input.output_digest.is_empty()
    {
        return Unknown(Reason::MalformedIdentity);
    }
    match input.provider_observation {
        ProviderObservation::AcknowledgementLost
        | ProviderObservation::RateLimited
        | ProviderObservation::MatchingObjectUnattributed => {
            return ReconcileOriginalAttempt;
        }
        ProviderObservation::ConflictingObject => {
            return Blocked(Reason::ConflictingRemoteEffect);
        }
        ProviderObservation::NoEffectAttempted => {}
    }
    if input.context != Context::OfflineSimulation {
        return Unknown(Reason::UnverifiedContext);
    }
    if input.credential_identity_ambiguous {
        return Unknown(Reason::AmbiguousCredential);
    }
    if input.authority_generation != input.current_authority_generation
        || input.evidence.task_authority == Claim::Revoked
    {
        return Blocked(Reason::StaleAuthority);
    }
    if input.observed_target_head != input.intended_target_head {
        return Blocked(Reason::MovedTarget);
    }
    if input.terms_version != input.current_terms_version || !simulated(input.evidence.terms_assent)
    {
        return HumanReviewRequired(Reason::StaleTerms);
    }
    if !simulated(input.evidence.human_act) {
        return Blocked(Reason::MissingHumanAct);
    }
    if !simulated(input.evidence.dco_declaration) {
        return Blocked(Reason::MissingDco);
    }
    if !simulated(input.evidence.code_rights)
        || !simulated(input.evidence.asset_rights)
        || !simulated(input.evidence.employer_rights)
    {
        return HumanReviewRequired(Reason::UnclearRights);
    }
    if !simulated(input.evidence.historical_lineage) {
        return Unknown(Reason::UnverifiedHistory);
    }
    if !simulated(input.evidence.task_authority) {
        return Blocked(Reason::MissingTaskAuthorization);
    }
    if !simulated(input.evidence.reviewer_decision) {
        return HumanReviewRequired(Reason::MissingReview);
    }
    if input.reviewed_output_digest != input.output_digest {
        return Blocked(Reason::ChangedReviewedContent);
    }
    if input.transformation_occurred && !simulated(input.evidence.transformation_map) {
        return HumanReviewRequired(Reason::UnreviewedTransformation);
    }
    Decision::ReviewableInSimulation
}

/// Canonical v1 encoding: domain prefix + fixed-order length-prefixed UTF-8
/// fields and fixed-width little-endian numbers; field boundaries cannot alias.
/// Changing field order/meaning/encoding requires a new schema version.
pub fn canonical_bytes(record: &Admission) -> Vec<u8> {
    fn field(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    fn claim(c: Claim) -> u8 {
        match c {
            Claim::Absent => 0,
            Claim::CallerClaim => 1,
            Claim::SyntheticFixture => 2,
            Claim::Disputed => 3,
            Claim::Revoked => 4,
        }
    }
    let mut out = b"free-energy/synthetic-admission/v1\0".to_vec();
    out.extend_from_slice(&record.schema_version.to_le_bytes());
    out.extend_from_slice(&record.repository_id.to_le_bytes());
    field(&mut out, &record.repository_incarnation);
    field(&mut out, &record.source_commit);
    field(&mut out, &record.intended_target_head);
    field(&mut out, &record.observed_target_head);
    field(&mut out, &record.input_digest);
    field(&mut out, &record.output_digest);
    field(&mut out, &record.reviewed_output_digest);
    for number in [
        record.terms_version,
        record.current_terms_version,
        record.authority_generation,
        record.current_authority_generation,
    ] {
        out.extend_from_slice(&number.to_le_bytes());
    }
    out.push(u8::from(record.transformation_occurred));
    out.push(u8::from(record.credential_identity_ambiguous));
    out.push(match record.context {
        Context::OfflineSimulation => 0,
        Context::InteractiveOnly => 1,
        Context::Unverified => 2,
    });
    out.push(match record.provider_observation {
        ProviderObservation::NoEffectAttempted => 0,
        ProviderObservation::AcknowledgementLost => 1,
        ProviderObservation::RateLimited => 2,
        ProviderObservation::MatchingObjectUnattributed => 3,
        ProviderObservation::ConflictingObject => 4,
    });
    for c in [
        record.evidence.human_act,
        record.evidence.dco_declaration,
        record.evidence.terms_assent,
        record.evidence.code_rights,
        record.evidence.asset_rights,
        record.evidence.employer_rights,
        record.evidence.reviewer_decision,
        record.evidence.task_authority,
        record.evidence.transformation_map,
        record.evidence.historical_lineage,
    ] {
        out.push(claim(c));
    }
    out.push(u8::from(record.github_signature_verified));
    out.push(u8::from(record.dco_bot_exempt));
    out
}

/// SHA-256 of the versioned synthetic envelope; digest is a content identity,
/// never a signature, assent proof, provider receipt, or authority to publish.
pub fn canonical_digest(record: &Admission) -> String {
    sha256::digest_hex(&canonical_bytes(record))
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationKey {
    pub repository_id: u64,
    pub repository_incarnation: String,
    pub operation_id: String,
    pub authority_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Replay {
    First(Decision),
    Identical(Decision),
    Conflict,
}

/// In-memory synthetic idempotency reducer; DOES NOT claim crash durability.
/// No transport calls exist: provider_effects_emitted is always zero.
#[derive(Default)]
pub struct SimulationJournal {
    records: BTreeMap<OperationKey, (String, Decision)>,
}

impl SimulationJournal {
    /// Bind one operation to the *actual* synthetic admission payload.
    ///
    /// The previous API accepted a caller-selected digest and result; a caller
    /// could replay different bytes under the same label or inject a positive
    /// disposition. Both the fingerprint and result are now derived here.
    /// This in-memory journal remains simulation-only, not a durable broker.
    pub fn record(&mut self, key: OperationKey, admission: &Admission) -> Replay {
        if key.repository_id == 0
            || key.repository_id != admission.repository_id
            || key.repository_incarnation.is_empty()
            || key.repository_incarnation != admission.repository_incarnation
            || key.operation_id.is_empty()
            || key.authority_generation == 0
            || key.authority_generation != admission.authority_generation
        {
            return Replay::Conflict;
        }

        // A changed repository incarnation or authority generation cannot
        // reuse the *same* operation ID as a fresh effect attempt. Preserve
        // the original receipt for reconciliation, even across recreation.
        if self.records.keys().any(|old| {
            old.repository_id == key.repository_id
                && old.operation_id == key.operation_id
                && (old.repository_incarnation != key.repository_incarnation
                    || old.authority_generation != key.authority_generation)
        }) {
            return Replay::Conflict;
        }

        let payload_digest = canonical_digest(admission);
        if let Some((old_digest, original)) = self.records.get(&key) {
            if old_digest == &payload_digest {
                Replay::Identical(*original)
            } else {
                Replay::Conflict
            }
        } else {
            let result = evaluate(admission);
            self.records.insert(key, (payload_digest, result));
            Replay::First(result)
        }
    }

    pub fn provider_effects_emitted(&self) -> usize {
        0
    }
}
