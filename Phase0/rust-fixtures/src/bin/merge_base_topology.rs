//! Read-only, deterministic fixture oracle for Phase0 merge-base topology spec 2.
//!
//! These are *model fixtures*, not proof of actual Git DAG traversal or a
//! reproducible virtual-base merge. A real integration planner must separately
//! establish complete best-base discovery under its effective Git history view.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const HISTORICAL_CASE_COUNT: usize = 12;
// The historical Python migration must not silently replace one of the named
// semantic cases with an unrelated case while retaining the same count.
const HISTORICAL_CASE_NAMES: [&str; HISTORICAL_CASE_COUNT] = [
    "unique-complete",
    "multiple-complete",
    "multiple-reordered",
    "none-complete",
    "incomplete-view",
    "multi-no-virtual",
    "multi-virtual-a",
    "multi-virtual-a-reordered",
    "multi-virtual-version-drift",
    "multi-virtual-intermediate-drift",
    "same-heads-replaced-history",
    "best-base-moved",
];
const FIXTURES: &str = include_str!("../../../fixtures/merge-base-topology-spec2.json");

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    history_view: String,
    complete: bool,
    bases: Vec<String>,
    #[serde(rename = "virtual")]
    virtual_basis: Option<VirtualBasis>,
    expect: Option<String>,
    expect_virtual: Option<String>,
    same_set_as: Option<String>,
    same_computation_as: Option<String>,
    different_computation_from: Option<String>,
    stale_against: Option<StaleBasis>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VirtualBasis {
    supported: bool,
    algorithm: Option<String>,
    version: Option<u64>,
    options: Option<BTreeMap<String, String>>,
    intermediates: Option<Vec<String>>,
    result: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StaleBasis {
    history_view: String,
    bases: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TopologyIdentity {
    history_view: String,
    complete: bool,
    bases: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VirtualIdentity {
    topology: TopologyIdentity,
    algorithm: String,
    version: u64,
    options: BTreeMap<String, String>,
    intermediates: Vec<String>,
    result: String,
}

#[derive(Clone, Debug)]
struct Observed {
    topology: TopologyIdentity,
    disposition: &'static str,
    virtual_status: Option<&'static str>,
    virtual_identity: Option<VirtualIdentity>,
    mutation_authority: bool,
}

// History-view, best-base and virtual-computation identities are compared
// byte-for-byte. Invisible formatting must not make distinct witnesses appear
// identical in human-reviewed fixture receipts. Preserve readable Unicode.
fn deceptive_identity_scalar(ch: char) -> bool {
    ch.is_control()
        || (!ch.is_ascii() && ch.is_whitespace())
        || matches!(
            ch,
            '\u{00ad}'
                | '\u{034f}'
                | '\u{061c}'
                | '\u{180e}'
                | '\u{115f}'
                | '\u{1160}'
                | '\u{3164}'
                | '\u{ffa0}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{206f}'
                | '\u{fe00}'..='\u{fe0f}'
                | '\u{feff}'
                | '\u{e0001}'
                | '\u{e0020}'..='\u{e007f}'
                | '\u{e0100}'..='\u{e01ef}'
        )
}

fn nonblank(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().any(deceptive_identity_scalar) {
        return Err(format!(
            "{field} must be nonblank without control or invisible formatting"
        ));
    }
    Ok(())
}

fn canonical_bases(values: &[String]) -> Result<Vec<String>, String> {
    let mut seen = BTreeSet::new();
    for value in values {
        nonblank(value, "merge base")?;
        if !seen.insert(value.as_str()) {
            return Err(format!("duplicate best merge base: {value}"));
        }
    }
    Ok(seen.into_iter().map(str::to_owned).collect())
}

fn topology(
    history_view: &str,
    complete: bool,
    bases: &[String],
) -> Result<TopologyIdentity, String> {
    nonblank(history_view, "history_view")?;
    Ok(TopologyIdentity {
        history_view: history_view.to_owned(),
        complete,
        bases: canonical_bases(bases)?,
    })
}

fn compute(case: &Case) -> Result<Observed, String> {
    nonblank(&case.name, "case name")?;
    let basis = topology(&case.history_view, case.complete, &case.bases)?;
    let disposition = if !basis.complete {
        "UNPROVABLE"
    } else {
        match basis.bases.len() {
            0 => "NONE",
            1 => "UNIQUE",
            _ => "MULTIPLE",
        }
    };

    let (virtual_status, virtual_identity) = match &case.virtual_basis {
        None => (None, None),
        Some(_) if disposition != "MULTIPLE" => {
            return Err("virtual-base declaration requires complete MULTIPLE topology".into());
        }
        Some(v) if !v.supported => {
            if v.algorithm.is_some()
                || v.version.is_some()
                || v.options.is_some()
                || v.intermediates.is_some()
                || v.result.is_some()
            {
                return Err("unsupported virtual-base path cannot assert computed fields".into());
            }
            (Some("UNSUPPORTED"), None)
        }
        Some(v) => {
            let algorithm = v.algorithm.as_deref().ok_or("missing virtual algorithm")?;
            nonblank(algorithm, "virtual algorithm")?;
            let version = v
                .version
                .filter(|version| *version > 0)
                .ok_or("invalid virtual version")?;
            let options = v.options.clone().ok_or("missing virtual options")?;
            for (key, value) in &options {
                nonblank(key, "virtual option key")?;
                nonblank(value, "virtual option value")?;
            }
            let intermediates = v
                .intermediates
                .as_ref()
                .ok_or("missing virtual intermediates")?;
            if intermediates.is_empty() {
                return Err("empty virtual intermediate list".into());
            }
            let intermediates = canonical_bases(intermediates)?;
            let result = v.result.as_deref().ok_or("missing virtual result")?;
            nonblank(result, "virtual result")?;
            (
                Some("SUPPORTED"),
                Some(VirtualIdentity {
                    topology: basis.clone(),
                    algorithm: algorithm.to_owned(),
                    version,
                    options,
                    intermediates,
                    result: result.to_owned(),
                }),
            )
        }
    };

    Ok(Observed {
        topology: basis,
        disposition,
        virtual_status,
        virtual_identity,
        // Read-only fixture observations never authorize a Git ref mutation.
        mutation_authority: false,
    })
}

fn check(cases: &[Case]) -> Result<usize, Vec<String>> {
    let mut errors = Vec::new();
    if cases.len() != HISTORICAL_CASE_COUNT {
        errors.push(format!(
            "expected {HISTORICAL_CASE_COUNT} historical cases, found {}",
            cases.len()
        ));
    }
    let mut observed = BTreeMap::<String, Observed>::new();

    for case in cases {
        if !HISTORICAL_CASE_NAMES.contains(&case.name.as_str()) {
            errors.push(format!("unknown historical merge-base case: {}", case.name));
            continue;
        }
        if observed.contains_key(&case.name) {
            errors.push(format!("duplicate case name: {}", case.name));
            continue;
        }
        match compute(case) {
            Err(error) => errors.push(format!("{}: {error}", case.name)),
            Ok(actual) => {
                if case.expect.is_none() && case.expect_virtual.is_none() {
                    errors.push(format!("{}: case has no verdict assertion", case.name));
                }
                if let Some(expected) = case.expect.as_deref() {
                    if expected != actual.disposition {
                        errors.push(format!(
                            "{}: expected topology {expected}, computed {}",
                            case.name, actual.disposition
                        ));
                    }
                }
                if let Some(expected) = case.expect_virtual.as_deref() {
                    if Some(expected) != actual.virtual_status {
                        errors.push(format!(
                            "{}: expected virtual {expected}, computed {:?}",
                            case.name, actual.virtual_status
                        ));
                    }
                }
                if actual.mutation_authority {
                    errors.push(format!(
                        "{}: topology granted mutation authority",
                        case.name
                    ));
                }
                observed.insert(case.name.clone(), actual);
            }
        }
    }

    for case in cases {
        let Some(actual) = observed.get(&case.name) else {
            continue;
        };
        // Equality to the case itself is not an independent cross-case witness.
        // Reject it even when all computed topology and virtual-base fields match.
        for (kind, reference) in [
            ("same_set_as", case.same_set_as.as_deref()),
            ("same_computation_as", case.same_computation_as.as_deref()),
            (
                "different_computation_from",
                case.different_computation_from.as_deref(),
            ),
        ] {
            if reference == Some(case.name.as_str()) {
                errors.push(format!(
                    "{}: {kind} requires a distinct witness case",
                    case.name
                ));
            }
        }
        if let Some(other) = case.same_set_as.as_deref() {
            match observed.get(other) {
                Some(basis) if actual.topology == basis.topology => {}
                _ => errors.push(format!(
                    "{}: same_set_as {other} identity mismatch",
                    case.name
                )),
            }
        }
        if let Some(other) = case.same_computation_as.as_deref() {
            match (actual.virtual_identity.as_ref(), observed.get(other)) {
                (Some(left), Some(right)) if Some(left) == right.virtual_identity.as_ref() => {}
                _ => errors.push(format!(
                    "{}: same_computation_as {other} identity mismatch",
                    case.name
                )),
            }
        }
        if let Some(other) = case.different_computation_from.as_deref() {
            match (actual.virtual_identity.as_ref(), observed.get(other)) {
                (Some(left), Some(right))
                    if right.virtual_identity.as_ref().is_some_and(|r| r != left) => {}
                _ => errors.push(format!(
                    "{}: different_computation_from {other} was not established",
                    case.name
                )),
            }
        }
        if let Some(stale) = case.stale_against.as_ref() {
            match topology(&stale.history_view, true, &stale.bases) {
                Ok(previous) if previous != actual.topology => {}
                Ok(_) => errors.push(format!(
                    "{}: stale_against reused the exact current history/base identity",
                    case.name
                )),
                Err(error) => errors.push(format!("{}: bad stale basis: {error}", case.name)),
            }
        }
    }

    if errors.is_empty() {
        Ok(observed.len())
    } else {
        Err(errors)
    }
}

// Preserve a zero-argument checked-in baseline, but also admit caller-owned
// fixtures for actual process-level semantic regression and input-boundary
// tests. This oracle remains read-only and grants no Git mutation authority.
fn run() -> Result<usize, String> {
    let mut args = std::env::args_os().skip(1);
    let input = args.next();
    if args.next().is_some() {
        return Err("usage: merge_base_topology [fixture.json]".to_owned());
    }
    let content = match input {
        Some(path) => std::fs::read_to_string(&path).map_err(|error| {
            format!("cannot read merge-base fixture {path:?}: {error}")
        })?,
        None => FIXTURES.to_owned(),
    };
    let cases: Vec<Case> = serde_json::from_str(&content)
        .map_err(|error| format!("merge-base-topology fixture parse failed: {error}"))?;
    check(&cases).map_err(|errors| errors.join("\n"))
}

fn main() {
    match run() {
        Ok(count) => println!("merge-base-topology: {count} read-only model fixtures PASS"),
        Err(error) => {
            eprintln!("merge-base-topology: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Vec<Case> {
        serde_json::from_str(FIXTURES).expect("checked-in fixtures parse")
    }

    fn case_mut<'a>(cases: &'a mut [Case], name: &str) -> &'a mut Case {
        cases
            .iter_mut()
            .find(|c| c.name == name)
            .expect("fixture present")
    }

    #[test]
    fn invisible_identity_scalars_cannot_spoof_merge_base_or_virtual_witnesses() {
        let cases = baseline();
        for marker in [
            '\u{00ad}',
            '\u{034f}',
            '\u{061c}',
            '\u{180e}',
            '\u{115f}',
            '\u{1160}',
            '\u{3164}',
            '\u{ffa0}',
            '\u{200b}',
            '\u{202e}',
            '\u{2060}',
            '\u{fe0f}',
            '\u{feff}',
            '\u{e0001}',
            '\u{e0020}',
            '\u{e007f}',
            '\u{e0100}',
            '\u{e01ef}',
        ] {
            let mut unique = cases
                .iter()
                .find(|case| case.name == "unique-complete")
                .expect("historical unique case")
                .clone();
            unique.history_view.push(marker);
            assert!(
                compute(&unique).is_err(),
                "invisible history-view marker passed: {marker:?}"
            );

            let mut unique = cases
                .iter()
                .find(|case| case.name == "unique-complete")
                .expect("historical unique case")
                .clone();
            unique.bases[0].push(marker);
            assert!(
                compute(&unique).is_err(),
                "invisible best-base marker passed: {marker:?}"
            );

            let mut multiple = cases
                .iter()
                .find(|case| case.name == "multi-virtual-a")
                .expect("historical virtual case")
                .clone();
            multiple
                .virtual_basis
                .as_mut()
                .expect("virtual basis")
                .algorithm
                .as_mut()
                .expect("virtual algorithm")
                .push(marker);
            assert!(
                compute(&multiple).is_err(),
                "invisible virtual algorithm marker passed: {marker:?}"
            );
        }

        // Whitespace that is visible in a Unicode scalar classification but
        // visually easy to overlook must not become a provenance token.
        let mut hidden_separator = cases
            .iter()
            .find(|case| case.name == "unique-complete")
            .expect("historical unique case")
            .clone();
        hidden_separator.history_view = "view\u{2007}name".to_owned();
        assert!(compute(&hidden_separator).is_err());

        // Reject spoofing without imposing ASCII-only identities.
        let mut readable = cases
            .iter()
            .find(|case| case.name == "unique-complete")
            .expect("historical unique case")
            .clone();
        readable.history_view = "café-λ".to_owned();
        assert!(compute(&readable).is_ok());
    }

    #[test]
    fn historical_twelve_cases_compute_correctly() {
        assert_eq!(check(&baseline()), Ok(12));
    }

    #[test]
    fn expected_labels_are_not_the_oracle() {
        let mut cases = baseline();
        case_mut(&mut cases, "unique-complete").expect = Some("NONE".into());
        assert!(check(&cases).is_err());
    }

    #[test]
    fn substituted_case_cannot_masquerade_as_complete_historical_suite() {
        let mut cases = baseline();
        case_mut(&mut cases, "unique-complete").name = "invented-equivalent-case".to_owned();
        assert_eq!(cases.len(), HISTORICAL_CASE_COUNT);
        assert!(
            check(&cases).is_err(),
            "same count must not permit replaced historical case"
        );
    }

    #[test]
    fn truncated_fixture_cannot_pass() {
        let mut cases = baseline();
        cases.pop();
        assert!(check(&cases).is_err());
    }

    #[test]
    fn reversed_discovery_keeps_canonical_identity() {
        let cases = baseline();
        let mut reordered = cases[1].clone();
        reordered.bases.reverse();
        assert_eq!(
            compute(&cases[1]).unwrap().topology,
            compute(&reordered).unwrap().topology
        );
    }

    #[test]
    fn duplicate_base_is_rejected_not_silently_deduplicated() {
        let mut cases = baseline();
        case_mut(&mut cases, "multiple-complete").bases = vec!["b1".into(), "b1".into()];
        assert!(check(&cases).is_err());
    }

    #[test]
    fn incomplete_history_cannot_inherit_unique_result() {
        let mut cases = baseline();
        case_mut(&mut cases, "unique-complete").complete = false;
        assert!(check(&cases).is_err());
    }

    #[test]
    fn unsupported_multi_base_cannot_assert_a_result_tree() {
        let mut cases = baseline();
        case_mut(&mut cases, "multi-no-virtual")
            .virtual_basis
            .as_mut()
            .unwrap()
            .result = Some("tree:forged".into());
        assert!(check(&cases).is_err());
    }

    #[test]
    fn supported_virtual_path_requires_reproducibility_fields() {
        let mut cases = baseline();
        case_mut(&mut cases, "multi-virtual-a")
            .virtual_basis
            .as_mut()
            .unwrap()
            .intermediates = None;
        assert!(check(&cases).is_err());
    }

    #[test]
    fn virtual_algorithm_version_change_invalidates_identity() {
        let cases = baseline();
        let mut moved = cases[6].clone();
        moved.virtual_basis.as_mut().unwrap().version = Some(2);
        assert_ne!(
            compute(&cases[6]).unwrap().virtual_identity,
            compute(&moved).unwrap().virtual_identity
        );
    }

    #[test]
    fn reversing_equivalent_intermediates_preserves_identity() {
        let cases = baseline();
        let mut reordered = cases[6].clone();
        reordered
            .virtual_basis
            .as_mut()
            .unwrap()
            .intermediates
            .as_mut()
            .unwrap()
            .reverse();
        assert_eq!(
            compute(&cases[6]).unwrap().virtual_identity,
            compute(&reordered).unwrap().virtual_identity
        );
    }

    #[test]
    fn identical_stale_basis_does_not_count_as_invalidation() {
        let mut cases = baseline();
        let current = case_mut(&mut cases, "best-base-moved");
        current.stale_against = Some(StaleBasis {
            history_view: current.history_view.clone(),
            bases: current.bases.clone(),
        });
        assert!(check(&cases).is_err());
    }

    #[test]
    fn no_outcome_grants_mutation_authority() {
        for case in baseline() {
            let observed = compute(&case).expect("valid modeled fixture");
            assert!(!observed.mutation_authority);
        }
    }

    #[test]
    fn cross_case_assertions_cannot_use_self_as_their_witness() {
        for (name, kind) in [
            ("multiple-reordered", "same_set_as"),
            ("multi-virtual-a-reordered", "same_computation_as"),
            ("multi-virtual-version-drift", "different_computation_from"),
        ] {
            let mut cases = baseline();
            let case = case_mut(&mut cases, name);
            match kind {
                "same_set_as" => case.same_set_as = Some(name.to_owned()),
                "same_computation_as" => case.same_computation_as = Some(name.to_owned()),
                "different_computation_from" => {
                    case.different_computation_from = Some(name.to_owned())
                }
                _ => unreachable!(),
            }
            let errors = check(&cases).expect_err("self-reference must fail closed");
            assert!(
                errors
                    .iter()
                    .any(|error| error.contains("requires a distinct witness case")),
                "{name}/{kind}: {errors:?}"
            );
        }
    }

    #[test]
    fn duplicate_case_name_is_rejected() {
        let mut cases = baseline();
        cases[1].name = cases[0].name.clone();
        assert!(check(&cases).is_err());
    }
}
