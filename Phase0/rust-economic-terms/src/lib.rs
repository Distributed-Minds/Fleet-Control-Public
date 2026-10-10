//! A deliberately OFFLINE, SYNTHETIC provider-term admission model for public #19 spec 2.
//!
//! No provider transport, credentials, payments, real human assent, durable
//! reservation ledger, trusted quote oracle, or publication endpoint exists.
//! `Synthetic` is an injected test-fixture assertion, never production authority.
//! ProviderConditional is also a *fake* adapter capability, not verified evidence
//! of any real provider implementing a server-side compare-and-commit operation.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    Absent,
    CallerClaim,
    Synthetic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    Unknown,
    ClientPrecheckOnly,
    ProviderConditional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    MalformedIdentity,
    MissingAuthority,
    StaleAuthority,
    MissingValuation,
    StaleValuation,
    MandateExceeded,
    MissingExposure,
    StaleExposure,
    UnavailableHeadroom,
    IncompatibleTerms,
    NoProviderConditionalBinding,
    ProviderTermsMoved,
    CommitFenceMoved,
    ConflictingOperation,
    LedgerCapacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    ReviewableInSimulation,
    Blocked(Reason),
    Unknown(Reason),
    ReconcileOriginalAttempt,
}

/// No omitted field may be silently treated as equivalent to the approved
/// economic/paid-human commitment. The exact quote incarnation is material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    pub offer_incarnation: String,
    pub provider_incarnation: String,
    pub counterparty: String,
    pub work_scope: String,
    pub currency: String,
    pub quantity: u32,
    pub amount: u64,
    pub fee_ceiling: u64,
    pub recurrence: String,
    pub cancellation: String,
    pub liability: String,
    pub substitution: String,
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}

impl Terms {
    fn valid(&self) -> bool {
        [
            &self.offer_incarnation,
            &self.provider_incarnation,
            &self.counterparty,
            &self.work_scope,
            &self.currency,
            &self.recurrence,
            &self.cancellation,
            &self.liability,
            &self.substitution,
        ]
        .into_iter()
        .all(|field| token(field))
            && self.quantity > 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mandate {
    pub provenance: Evidence,
    pub root_generation: u64,
    pub current_root_generation: u64,
    pub authorized_terms: Terms,
    pub maximum_worst_case: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Valuation {
    pub provenance: Evidence,
    pub generation: u64,
    pub current_generation: u64,
    pub conservative_worst_case: u64,
}

/// This is an *injected proof obligation*, not a substitute for #27 spec 6's
/// actual source-slice receipt, shared-accounting CAS, or release/debt ledger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exposure {
    pub provenance: Evidence,
    pub canonical_component: String,
    pub offered_component: String,
    pub source_allocation_basis: String,
    pub selected_allocation_basis: String,
    pub allocation_receipt: String,
    pub generation: u64,
    pub current_generation: u64,
    pub remaining_headroom: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub operation: String,
    pub mandate: Mandate,
    pub valuation: Valuation,
    pub exposure: Exposure,
    pub observed_offer: Terms,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prepared {
    pub operation: String,
    pub exact_terms: Terms,
    pub root_generation: u64,
    pub valuation_generation: u64,
    pub exposure_generation: u64,
    pub source_allocation_basis: String,
    pub allocation_receipt: String,
}

/// Stops *before* even a fake external effect when evidence is missing or
/// mismatched. Only a synthetic, typed `Prepared` may proceed to the fake cut.
pub fn prepare(request: &Request) -> Result<Prepared, Decision> {
    let mandate = &request.mandate;
    let quote = &request.observed_offer;
    let valuation = &request.valuation;
    let exposure = &request.exposure;
    if !token(&request.operation)
        || !quote.valid()
        || !mandate.authorized_terms.valid()
        || !token(&exposure.canonical_component)
        || !token(&exposure.offered_component)
        || !token(&exposure.source_allocation_basis)
        || !token(&exposure.selected_allocation_basis)
        || !token(&exposure.allocation_receipt)
    {
        return Err(Decision::Unknown(Reason::MalformedIdentity));
    }
    if mandate.provenance != Evidence::Synthetic {
        return Err(Decision::Blocked(Reason::MissingAuthority));
    }
    if mandate.root_generation == 0 || mandate.root_generation != mandate.current_root_generation {
        return Err(Decision::Blocked(Reason::StaleAuthority));
    }
    if valuation.provenance != Evidence::Synthetic {
        return Err(Decision::Unknown(Reason::MissingValuation));
    }
    if valuation.generation == 0 || valuation.generation != valuation.current_generation {
        return Err(Decision::Unknown(Reason::StaleValuation));
    }
    // Exact authorized terms are the primary admission identity: a changed
    // amount must not be reclassified as only a valuation/ceiling failure.
    if quote != &mandate.authorized_terms {
        return Err(Decision::Blocked(Reason::IncompatibleTerms));
    }
    let Some(minimum_cost) = quote.amount.checked_add(quote.fee_ceiling) else {
        return Err(Decision::Blocked(Reason::MandateExceeded));
    };
    if valuation.conservative_worst_case < minimum_cost
        || valuation.conservative_worst_case > mandate.maximum_worst_case
    {
        return Err(Decision::Blocked(Reason::MandateExceeded));
    }
    if exposure.provenance != Evidence::Synthetic
        || exposure.canonical_component != exposure.offered_component
    {
        return Err(Decision::Unknown(Reason::MissingExposure));
    }
    if exposure.generation == 0
        || exposure.generation != exposure.current_generation
        || exposure.source_allocation_basis != exposure.selected_allocation_basis
    {
        return Err(Decision::Unknown(Reason::StaleExposure));
    }
    if exposure.remaining_headroom < valuation.conservative_worst_case {
        return Err(Decision::Blocked(Reason::UnavailableHeadroom));
    }
    Ok(Prepared {
        operation: request.operation.clone(),
        exact_terms: quote.clone(),
        root_generation: mandate.root_generation,
        valuation_generation: valuation.generation,
        exposure_generation: exposure.generation,
        source_allocation_basis: exposure.source_allocation_basis.clone(),
        allocation_receipt: exposure.allocation_receipt.clone(),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitFence {
    pub root_generation: u64,
    pub valuation_generation: u64,
    pub exposure_generation: u64,
    pub source_allocation_basis: String,
    pub allocation_receipt: String,
}

#[derive(Clone, Debug)]
pub struct FakeProvider {
    pub binding: Binding,
    pub actual_terms_at_commit: Terms,
    pub acknowledgement_lost: bool,
    simulated_accepted_count: usize,
}

impl FakeProvider {
    pub fn new(binding: Binding, terms: Terms) -> Self {
        Self {
            binding,
            actual_terms_at_commit: terms,
            acknowledgement_lost: false,
            simulated_accepted_count: 0,
        }
    }

    /// Counts purely simulated events. Never calls a real provider.
    pub fn simulated_accepted_count(&self) -> usize {
        self.simulated_accepted_count
    }

    fn compare_and_commit(&mut self, prepared: &Prepared) -> Decision {
        // A client-side last-second GET, hash header or local lock is NOT a
        // server-enforced condition. Missing capability is an absolute stop.
        if self.binding != Binding::ProviderConditional {
            return Decision::Blocked(Reason::NoProviderConditionalBinding);
        }
        // One indivisible fake-provider step models provider-side comparison
        // with exact *accepted* material terms, not merely the price or URL.
        if self.actual_terms_at_commit != prepared.exact_terms {
            return Decision::Blocked(Reason::ProviderTermsMoved);
        }
        self.simulated_accepted_count += 1;
        if self.acknowledgement_lost {
            Decision::ReconcileOriginalAttempt
        } else {
            Decision::ReviewableInSimulation
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Replay {
    First(Decision),
    Identical(Decision),
    Conflict,
    CapacityExhausted,
}

/// In-memory *test* receipt ledger only. Never evicts or treats restart as
/// proof of provider no-effect. Production #10/#27 semantics are not supplied.
#[derive(Default)]
pub struct SimulationJournal {
    attempts: BTreeMap<String, (Prepared, Decision)>,
}

pub const MAX_SIMULATED_ATTEMPTS: usize = 1024;

impl SimulationJournal {
    pub fn attempt(
        &mut self,
        prepared: Prepared,
        fence: &CommitFence,
        provider: &mut FakeProvider,
    ) -> Replay {
        if let Some((old_prepared, old_result)) = self.attempts.get(&prepared.operation) {
            return if old_prepared == &prepared {
                Replay::Identical(*old_result)
            } else {
                Replay::Conflict
            };
        }
        if self.attempts.len() >= MAX_SIMULATED_ATTEMPTS {
            return Replay::CapacityExhausted;
        }
        let verdict = if fence.root_generation != prepared.root_generation
            || fence.valuation_generation != prepared.valuation_generation
            || fence.exposure_generation != prepared.exposure_generation
            || fence.source_allocation_basis != prepared.source_allocation_basis
            || fence.allocation_receipt != prepared.allocation_receipt
        {
            Decision::Blocked(Reason::CommitFenceMoved)
        } else {
            provider.compare_and_commit(&prepared)
        };
        self.attempts
            .insert(prepared.operation.clone(), (prepared, verdict));
        Replay::First(verdict)
    }

    pub fn receipt_count(&self) -> usize {
        self.attempts.len()
    }
}
