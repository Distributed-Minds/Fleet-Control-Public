//! Offline, synthetic spec-4 causal-closure cut oracle for public issue #16.
//!
//! This is deliberately not a provider client or a production revocation engine.
//! TrustedSelection represents a separate externally authorized registry selector.
//! Supplying it in a test does not authenticate a live issuer.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Outcome {
    CompleteForDeclaredSurfaces,
    Partial,
    Unknown,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Composition {
    AllRequired,
    AnyOfDeclared,
    Unsupported,
}

#[derive(Clone, Debug)]
struct TrustedSelection {
    issuer: &'static str,
    selected_head: u64,
    predecessor: u64,
    generation: u64,
    cutoff: u64,
    surfaces: BTreeSet<&'static str>,
    accepted_causes: BTreeSet<&'static str>,
    independently_rooted_causes: BTreeSet<&'static str>,
}

#[derive(Clone, Debug)]
struct Surface {
    id: &'static str,
    generation: u64,
    acknowledged_through: u64,
    drained: bool,
}

#[derive(Clone, Debug)]
struct Cause {
    id: &'static str,
    from: &'static str,
    to: &'static str,
    enqueue_sequence: u64,
    terminal_or_independently_transferred: bool,
    accepted_but_unmaterialized: bool,
}

#[derive(Clone, Debug)]
struct ProposedCut {
    issuer: &'static str,
    selected_head: u64,
    predecessor: u64,
    generation: u64,
    cutoff: u64,
    authorized_transition: bool,
    discovery_fenced: bool,
    inventory_complete: bool,
    causal_ledger_complete: bool,
    provider_error: bool,
    unreconciled_ack: bool,
    resource_incarnation_changed: bool,
    composition: Composition,
    global_cycle_barrier: bool,
    surfaces: Vec<Surface>,
    causes: Vec<Cause>,
    residual_settled_obligations: u32,
}

// Every positive authority/currentness assertion is checked against a separate
// trusted selector; a candidate's self-supplied head is not proof of currentness.
fn evaluate(selection: &TrustedSelection, cut: &ProposedCut) -> Outcome {
    if cut.provider_error {
        return Outcome::Error;
    }
    if cut.issuer != selection.issuer
        || cut.selected_head != selection.selected_head
        || cut.predecessor != selection.predecessor
        || cut.generation != selection.generation
        || cut.cutoff != selection.cutoff
        || !cut.authorized_transition
        || !cut.discovery_fenced
        || !cut.inventory_complete
        || !cut.causal_ledger_complete
        || cut.unreconciled_ack
        || cut.resource_incarnation_changed
        || cut.composition == Composition::Unsupported
    {
        return Outcome::Unknown;
    }

    let mut frontiers = BTreeMap::new();
    for surface in &cut.surfaces {
        if !selection.surfaces.contains(surface.id)
            || surface.generation != selection.generation
            || frontiers.insert(surface.id, surface).is_some()
        {
            return Outcome::Unknown;
        }
    }
    if frontiers.keys().copied().collect::<BTreeSet<_>>() != selection.surfaces {
        return Outcome::Unknown;
    }
    if frontiers.values().any(|frontier| !frontier.drained) {
        return Outcome::Partial;
    }

    let mut cause_ids = BTreeSet::new();
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for cause in &cut.causes {
        if !cause_ids.insert(cause.id) {
            return Outcome::Unknown;
        }
        if selection.independently_rooted_causes.contains(cause.id) {
            // Only externally selected provenance exempts an independent root.
            continue;
        }
        if !selection.accepted_causes.contains(cause.id) {
            return Outcome::Unknown;
        }
        let Some(source) = frontiers.get(cause.from) else {
            return Outcome::Unknown;
        };
        let Some(target) = frontiers.get(cause.to) else {
            return Outcome::Unknown;
        };
        if !source.drained
            || !target.drained
            || cause.accepted_but_unmaterialized
            || !cause.terminal_or_independently_transferred
            // Synthetic shared sequence must be covered by *both* causal ends.
            || cause.enqueue_sequence > source.acknowledged_through
            || cause.enqueue_sequence > target.acknowledged_through
        {
            return Outcome::Partial;
        }
        adjacency.entry(cause.from).or_default().push(cause.to);
    }
    let expected = selection.accepted_causes.union(&selection.independently_rooted_causes)
        .copied().collect::<BTreeSet<_>>();
    if cause_ids != expected {
        return Outcome::Unknown;
    }

    // Local queue snapshots do not establish a coherent cut around a cycle.
    for start in &selection.surfaces {
        let mut pending = vec![*start];
        let mut seen = BTreeSet::new();
        while let Some(current) = pending.pop() {
            if let Some(next) = adjacency.get(current) {
                for destination in next {
                    if destination == start && !cut.global_cycle_barrier {
                        return Outcome::Partial;
                    }
                    if seen.insert(*destination) {
                        pending.push(*destination);
                    }
                }
            }
        }
    }

    // Committed obligations remain recorded; they are not future authority.
    let _ = cut.residual_settled_obligations;
    Outcome::CompleteForDeclaredSurfaces
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> (TrustedSelection, ProposedCut) {
        let selection = TrustedSelection {
            issuer: "trusted-registry", selected_head: 9, predecessor: 8,
            generation: 3, cutoff: 17,
            surfaces: ["A", "B", "C"].into_iter().collect(),
            accepted_causes: ["ab", "bc"].into_iter().collect(),
            independently_rooted_causes: BTreeSet::new(),
        };
        let cut = ProposedCut {
            issuer: "trusted-registry", selected_head: 9, predecessor: 8,
            generation: 3, cutoff: 17, authorized_transition: true,
            discovery_fenced: true, inventory_complete: true,
            causal_ledger_complete: true, provider_error: false,
            unreconciled_ack: false, resource_incarnation_changed: false,
            composition: Composition::AllRequired, global_cycle_barrier: false,
            surfaces: ["A", "B", "C"].into_iter().map(|id| Surface {
                id, generation: 3, acknowledged_through: 10, drained: true,
            }).collect(),
            causes: vec![
                Cause { id: "ab", from: "A", to: "B", enqueue_sequence: 4,
                    terminal_or_independently_transferred: true, accepted_but_unmaterialized: false },
                Cause { id: "bc", from: "B", to: "C", enqueue_sequence: 5,
                    terminal_or_independently_transferred: true, accepted_but_unmaterialized: false },
            ],
            residual_settled_obligations: 0,
        };
        (selection, cut)
    }

    #[test]
    fn c4_01_through_18_closure_matrix() {
        use Outcome::{CompleteForDeclaredSurfaces as Complete, Partial, Unknown};
        let (selection, mut cut) = baseline();
        cut.surfaces[1].acknowledged_through = 3;
        assert_eq!(evaluate(&selection, &cut), Partial, "C4-01");
        let (selection, mut cut) = baseline();
        assert_eq!(evaluate(&selection, &cut), Complete, "C4-02");
        cut.surfaces[2].acknowledged_through = 4;
        assert_eq!(evaluate(&selection, &cut), Partial, "C4-03");
        let (mut selection, mut cut) = baseline();
        selection.accepted_causes.insert("ac");
        cut.causes.push(Cause { id: "ac", from: "A", to: "C", enqueue_sequence: 9,
            terminal_or_independently_transferred: true, accepted_but_unmaterialized: false });
        cut.surfaces[2].drained = false;
        assert_eq!(evaluate(&selection, &cut), Partial, "C4-04");
        let (mut selection, mut cut) = baseline();
        selection.accepted_causes.insert("ba");
        cut.causes.push(Cause { id: "ba", from: "B", to: "A", enqueue_sequence: 6,
            terminal_or_independently_transferred: true, accepted_but_unmaterialized: false });
        assert_eq!(evaluate(&selection, &cut), Partial, "C4-05");
        cut.global_cycle_barrier = true;
        assert_eq!(evaluate(&selection, &cut), Complete, "C4-06");
        let (selection, mut cut) = baseline();
        cut.causes[0].accepted_but_unmaterialized = true;
        assert_eq!(evaluate(&selection, &cut), Partial, "C4-07");
        cut.causes[0].accepted_but_unmaterialized = false;
        cut.causes[0].terminal_or_independently_transferred = false;
        assert_eq!(evaluate(&selection, &cut), Partial, "C4-08");
        let (selection, mut cut) = baseline();
        cut.surfaces.push(Surface { id: "D", generation: 3, acknowledged_through: 10, drained: true });
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-09");
        let (selection, mut cut) = baseline();
        cut.discovery_fenced = false;
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-10");
        let (mut selection, mut cut) = baseline();
        selection.independently_rooted_causes.insert("independent");
        cut.causes.push(Cause { id: "independent", from: "OUTSIDE", to: "C", enqueue_sequence: 999,
            terminal_or_independently_transferred: false, accepted_but_unmaterialized: true });
        assert_eq!(evaluate(&selection, &cut), Complete, "C4-11");
        let (selection, mut cut) = baseline();
        cut.selected_head = 8;
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-12");
        let (selection, mut cut) = baseline();
        cut.causal_ledger_complete = false;
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-13");
        let (selection, mut cut) = baseline();
        cut.surfaces[1].generation = 2;
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-14");
        let (selection, mut cut) = baseline();
        cut.unreconciled_ack = true;
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-15a");
        cut.unreconciled_ack = false;
        cut.resource_incarnation_changed = true;
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-15b");
        let (selection, mut cut) = baseline();
        cut.composition = Composition::Unsupported;
        assert_eq!(evaluate(&selection, &cut), Unknown, "C4-16");
        let (selection, mut cut) = baseline();
        cut.residual_settled_obligations = 42;
        assert_eq!(evaluate(&selection, &cut), Complete, "C4-17");
        let (selection, cut) = baseline();
        assert_eq!(evaluate(&selection, &cut), Complete, "C4-18");
    }

    #[test]
    fn positive_mutants_fail_closed() {
        let (selection, mut cut) = baseline();
        cut.selected_head += 1;
        assert_ne!(evaluate(&selection, &cut), Outcome::CompleteForDeclaredSurfaces);
        let (selection, mut cut) = baseline();
        cut.surfaces.remove(1);
        assert_ne!(evaluate(&selection, &cut), Outcome::CompleteForDeclaredSurfaces);
        let (selection, mut cut) = baseline();
        cut.causes.remove(0);
        assert_ne!(evaluate(&selection, &cut), Outcome::CompleteForDeclaredSurfaces);
        let (selection, mut cut) = baseline();
        cut.causes[0].enqueue_sequence = 11;
        assert_ne!(evaluate(&selection, &cut), Outcome::CompleteForDeclaredSurfaces);
        let (selection, mut cut) = baseline();
        cut.authorized_transition = false;
        assert_ne!(evaluate(&selection, &cut), Outcome::CompleteForDeclaredSurfaces);
        let (selection, mut cut) = baseline();
        cut.inventory_complete = false;
        assert_ne!(evaluate(&selection, &cut), Outcome::CompleteForDeclaredSurfaces);
        let (selection, mut cut) = baseline();
        cut.provider_error = true;
        assert_eq!(evaluate(&selection, &cut), Outcome::Error);
        let (selection, mut cut) = baseline();
        cut.composition = Composition::AnyOfDeclared;
        assert_eq!(evaluate(&selection, &cut), Outcome::CompleteForDeclaredSurfaces);
    }

    #[test]
    fn source_frontier_cannot_lag_accepted_cross_provider_emission() {
        // The baseline has A -> B at logical emission seq 4 and B -> C at 5.
        // Destination-only checking falsely reports COMPLETE for A ack < 4.
        // This models shared logical ordering, not trusted provider attestations.
        let (selection, baseline_cut) = baseline();
        let mut cases = 0;
        for emission_sequence in 1..=16_u64 {
            for source_ack in 0..=16_u64 {
                let mut cut = baseline_cut.clone();
                cut.causes[0].enqueue_sequence = emission_sequence;
                cut.surfaces[0].acknowledged_through = source_ack;
                let expected = if emission_sequence <= source_ack && emission_sequence <= 10 {
                    Outcome::CompleteForDeclaredSurfaces
                } else {
                    Outcome::Partial
                };
                assert_eq!(
                    evaluate(&selection, &cut),
                    expected,
                    "emission_sequence={emission_sequence} source_ack={source_ack}"
                );
                cases += 1;
            }
        }
        assert_eq!(cases, 272);
    }
}
