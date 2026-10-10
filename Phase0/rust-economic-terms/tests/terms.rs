use free_energy_economic_terms_sim::{
    prepare, Binding, CommitFence, Decision, Evidence, Exposure, FakeProvider, Mandate, Prepared,
    Reason, Replay, Request, SimulationJournal, Terms, Valuation, MAX_SIMULATED_ATTEMPTS,
};

fn terms() -> Terms {
    Terms {
        offer_incarnation: "quote-1".into(),
        provider_incarnation: "provider-1".into(),
        counterparty: "worker-a".into(),
        work_scope: "translate-manual".into(),
        currency: "USD".into(),
        quantity: 1,
        amount: 100,
        fee_ceiling: 20,
        recurrence: "once".into(),
        cancellation: "refundable".into(),
        liability: "bounded".into(),
        substitution: "none".into(),
    }
}

fn request() -> Request {
    Request {
        operation: "intent-001".into(),
        mandate: Mandate {
            provenance: Evidence::Synthetic,
            root_generation: 7,
            current_root_generation: 7,
            authorized_terms: terms(),
            maximum_worst_case: 130,
        },
        valuation: Valuation {
            provenance: Evidence::Synthetic,
            generation: 5,
            current_generation: 5,
            conservative_worst_case: 125,
        },
        exposure: Exposure {
            provenance: Evidence::Synthetic,
            canonical_component: "risk-domain-1".into(),
            offered_component: "risk-domain-1".into(),
            source_allocation_basis: "allocation-v6".into(),
            selected_allocation_basis: "allocation-v6".into(),
            allocation_receipt: "synthetic-receipt-1".into(),
            generation: 11,
            current_generation: 11,
            remaining_headroom: 200,
        },
        observed_offer: terms(),
    }
}

fn prepared() -> Prepared {
    prepare(&request()).expect("valid synthetic fixture")
}

fn fence(p: &Prepared) -> CommitFence {
    p.synthetic_fence()
}

#[test]
fn positive_is_simulation_reviewable_only() {
    let p = prepared();
    let mut provider = FakeProvider::new(Binding::ProviderConditional, terms());
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::First(Decision::ReviewableInSimulation)
    );
    assert_eq!(provider.simulated_accepted_count(), 1);
    assert_eq!(journal.receipt_count(), 1);
}

#[test]
fn malformed_operation_and_exact_terms_identifiers_fail_closed() {
    for bad in ["", "padded op", "evil\nnew", "bidi\u{202e}", "x/y"] {
        let mut x = request();
        x.operation = bad.into();
        assert_eq!(
            prepare(&x),
            Err(Decision::Unknown(Reason::MalformedIdentity))
        );
        let mut x = request();
        x.observed_offer.offer_incarnation = bad.into();
        assert_eq!(
            prepare(&x),
            Err(Decision::Unknown(Reason::MalformedIdentity))
        );
    }
}

#[test]
fn synthetic_authority_cannot_be_replaced_by_a_caller_claim() {
    for bad in [Evidence::Absent, Evidence::CallerClaim] {
        let mut x = request();
        x.mandate.provenance = bad;
        assert_eq!(
            prepare(&x),
            Err(Decision::Blocked(Reason::MissingAuthority))
        );
    }
}

#[test]
fn generation_zero_and_revocation_prevent_preparation() {
    for (generation, current) in [(0, 0), (7, 8), (8, 7)] {
        let mut x = request();
        x.mandate.root_generation = generation;
        x.mandate.current_root_generation = current;
        assert_eq!(prepare(&x), Err(Decision::Blocked(Reason::StaleAuthority)));
    }
}

#[test]
fn valuation_requires_synthetic_source_and_current_generation() {
    for bad in [Evidence::Absent, Evidence::CallerClaim] {
        let mut x = request();
        x.valuation.provenance = bad;
        assert_eq!(
            prepare(&x),
            Err(Decision::Unknown(Reason::MissingValuation))
        );
    }
    for (generation, current) in [(0, 0), (5, 6)] {
        let mut x = request();
        x.valuation.generation = generation;
        x.valuation.current_generation = current;
        assert_eq!(prepare(&x), Err(Decision::Unknown(Reason::StaleValuation)));
    }
}

#[test]
fn source_credit_basis_and_canonical_component_cannot_be_swapped() {
    let mut x = request();
    x.exposure.selected_allocation_basis = "allocation-v7".into();
    assert_eq!(prepare(&x), Err(Decision::Unknown(Reason::StaleExposure)));
    let mut x = request();
    x.exposure.offered_component = "risk-domain-other".into();
    assert_eq!(prepare(&x), Err(Decision::Unknown(Reason::MissingExposure)));
}

#[test]
fn exposure_requires_current_provenance_and_generation() {
    let mut x = request();
    x.exposure.provenance = Evidence::CallerClaim;
    assert_eq!(prepare(&x), Err(Decision::Unknown(Reason::MissingExposure)));
    let mut x = request();
    x.exposure.current_generation += 1;
    assert_eq!(prepare(&x), Err(Decision::Unknown(Reason::StaleExposure)));
    let mut x = request();
    x.exposure.allocation_receipt = "".into();
    assert_eq!(
        prepare(&x),
        Err(Decision::Unknown(Reason::MalformedIdentity))
    );
}

#[test]
fn capacity_and_maximum_worst_case_cannot_be_exceeded() {
    let mut x = request();
    x.valuation.conservative_worst_case = 131;
    assert_eq!(prepare(&x), Err(Decision::Blocked(Reason::MandateExceeded)));
    let mut x = request();
    x.exposure.remaining_headroom = 124;
    assert_eq!(
        prepare(&x),
        Err(Decision::Blocked(Reason::UnavailableHeadroom))
    );
    let mut x = request();
    x.valuation.conservative_worst_case = 119;
    assert_eq!(prepare(&x), Err(Decision::Blocked(Reason::MandateExceeded)));
}

#[test]
fn checked_arithmetic_rejects_amount_fee_overflow() {
    let mut x = request();
    x.observed_offer.amount = u64::MAX;
    x.mandate.authorized_terms.amount = u64::MAX;
    assert_eq!(prepare(&x), Err(Decision::Blocked(Reason::MandateExceeded)));
}

fn drifted_quotes() -> Vec<Terms> {
    let base = terms();
    let mut all = Vec::new();
    let mut x = base.clone();
    x.amount = 110;
    all.push(x);
    let mut x = base.clone();
    x.offer_incarnation = "quote-2".into();
    all.push(x);
    let mut x = base.clone();
    x.provider_incarnation = "provider-2".into();
    all.push(x);
    let mut x = base.clone();
    x.counterparty = "worker-b".into();
    all.push(x);
    let mut x = base.clone();
    x.work_scope = "surveillance-task".into();
    all.push(x);
    let mut x = base.clone();
    x.quantity = 3;
    all.push(x);
    let mut x = base.clone();
    x.fee_ceiling = 25;
    all.push(x);
    let mut x = base.clone();
    x.recurrence = "monthly".into();
    all.push(x);
    let mut x = base.clone();
    x.cancellation = "nonrefundable".into();
    all.push(x);
    let mut x = base.clone();
    x.liability = "unlimited".into();
    all.push(x);
    let mut x = base.clone();
    x.substitution = "any-worker".into();
    all.push(x);
    let mut x = base;
    x.currency = "EUR".into();
    all.push(x);
    all
}

#[test]
fn all_twelve_material_preparation_term_changes_fail_closed() {
    for candidate in drifted_quotes() {
        let mut x = request();
        x.observed_offer = candidate;
        assert_eq!(
            prepare(&x),
            Err(Decision::Blocked(Reason::IncompatibleTerms))
        );
    }
}

#[test]
fn all_twelve_postcheck_changes_fail_at_provider_conditional_boundary() {
    let p = prepared();
    for candidate in drifted_quotes() {
        let mut provider = FakeProvider::new(Binding::ProviderConditional, candidate);
        let mut journal = SimulationJournal::default();
        assert_eq!(
            journal.attempt(p.clone(), &fence(&p), &mut provider),
            Replay::First(Decision::Blocked(Reason::ProviderTermsMoved))
        );
        assert_eq!(provider.simulated_accepted_count(), 0);
    }
}

#[test]
fn client_precheck_and_absent_capability_are_not_provider_cas() {
    let p = prepared();
    for binding in [Binding::Unknown, Binding::ClientPrecheckOnly] {
        let mut provider = FakeProvider::new(binding, terms());
        let mut journal = SimulationJournal::default();
        assert_eq!(
            journal.attempt(p.clone(), &fence(&p), &mut provider),
            Replay::First(Decision::Blocked(Reason::NoProviderConditionalBinding))
        );
        assert_eq!(provider.simulated_accepted_count(), 0);
    }
}

#[test]
fn every_changed_effect_fence_field_blocks_before_fake_provider_acceptance() {
    let p = prepared();
    for index in 0..5 {
        let mut bad = fence(&p);
        match index {
            0 => bad.root_generation += 1,
            1 => bad.valuation_generation += 1,
            2 => bad.exposure_generation += 1,
            3 => bad.source_allocation_basis = "allocation-v7".into(),
            _ => bad.allocation_receipt = "different-receipt".into(),
        }
        let mut provider = FakeProvider::new(Binding::ProviderConditional, terms());
        let mut journal = SimulationJournal::default();
        assert_eq!(
            journal.attempt(p.clone(), &bad, &mut provider),
            Replay::First(Decision::Blocked(Reason::CommitFenceMoved))
        );
        assert_eq!(provider.simulated_accepted_count(), 0);
    }
}

#[test]
fn lost_ack_keeps_original_operation_under_reconciliation() {
    let p = prepared();
    let mut provider = FakeProvider::new(Binding::ProviderConditional, terms());
    provider.acknowledgement_lost = true;
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::First(Decision::ReconcileOriginalAttempt)
    );
    assert_eq!(provider.simulated_accepted_count(), 1);
    provider.actual_terms_at_commit.offer_incarnation = "quote-2".into();
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::Identical(Decision::ReconcileOriginalAttempt)
    );
    assert_eq!(provider.simulated_accepted_count(), 1);
}

#[test]
fn identical_retry_does_not_duplicate_mock_acceptance() {
    let p = prepared();
    let mut provider = FakeProvider::new(Binding::ProviderConditional, terms());
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::First(Decision::ReviewableInSimulation)
    );
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(provider.simulated_accepted_count(), 1);
}

#[test]
fn same_operation_new_terms_or_generation_conflicts_without_provider_effect() {
    let p = prepared();
    let mut provider = FakeProvider::new(Binding::ProviderConditional, terms());
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::First(Decision::ReviewableInSimulation)
    );
    let mut changed_request = request();
    changed_request.observed_offer.offer_incarnation = "quote-2".into();
    changed_request.mandate.authorized_terms.offer_incarnation = "quote-2".into();
    let changed = prepare(&changed_request).expect("current synthetic offer");
    assert_eq!(
        journal.attempt(changed, &fence(&p), &mut provider),
        Replay::Conflict
    );
    let mut changed_request = request();
    changed_request.mandate.root_generation += 1;
    changed_request.mandate.current_root_generation += 1;
    let changed = prepare(&changed_request).expect("current synthetic root generation");
    assert_eq!(
        journal.attempt(changed, &fence(&p), &mut provider),
        Replay::Conflict
    );
    assert_eq!(provider.simulated_accepted_count(), 1);
}

#[test]
fn full_ledger_never_evicts_old_attempts_for_new_operation() {
    let mut journal = SimulationJournal::default();
    let mut provider = FakeProvider::new(Binding::ProviderConditional, terms());
    for i in 0..MAX_SIMULATED_ATTEMPTS {
        let mut req = request();
        req.operation = format!("intent-{i}");
        let p = prepare(&req).expect("distinct synthetic operation");
        assert_eq!(
            journal.attempt(p.clone(), &fence(&p), &mut provider),
            Replay::First(Decision::ReviewableInSimulation)
        );
    }
    let mut req = request();
    req.operation = "one-more".into();
    let extra = prepare(&req).expect("valid extra synthetic operation");
    assert_eq!(
        journal.attempt(extra.clone(), &fence(&extra), &mut provider),
        Replay::CapacityExhausted
    );
    assert_eq!(journal.receipt_count(), MAX_SIMULATED_ATTEMPTS);
    let mut req = request();
    req.operation = "intent-0".into();
    let original = prepare(&req).expect("original synthetic operation");
    assert_eq!(
        journal.attempt(original.clone(), &fence(&original), &mut provider),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(provider.simulated_accepted_count(), MAX_SIMULATED_ATTEMPTS);
}

#[test]
fn stale_shared_exposure_receipt_blocks_second_root_only() {
    // This fixture does NOT demonstrate #27's cross-root serialized CAS.
    let mut first = request();
    first.mandate.root_generation = 8;
    first.mandate.current_root_generation = 8;
    assert!(prepare(&first).is_ok());
    let mut second = request();
    second.mandate.root_generation = 9;
    second.mandate.current_root_generation = 9;
    second.exposure.current_generation += 1; // stale same-component receipt
    assert_eq!(
        prepare(&second),
        Err(Decision::Unknown(Reason::StaleExposure))
    );
}

#[test]
fn no_retry_of_denied_operation_under_changed_snapshot() {
    let p = prepared();
    let mut provider = FakeProvider::new(Binding::ClientPrecheckOnly, terms());
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::First(Decision::Blocked(Reason::NoProviderConditionalBinding))
    );
    provider.binding = Binding::ProviderConditional;
    assert_eq!(
        journal.attempt(p.clone(), &fence(&p), &mut provider),
        Replay::Identical(Decision::Blocked(Reason::NoProviderConditionalBinding))
    );
    assert_eq!(provider.simulated_accepted_count(), 0);
}
