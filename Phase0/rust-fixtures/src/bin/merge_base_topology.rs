//! Read-only, deterministic fixture oracle for Phase0 merge-base topology spec 2.
//!
//! These are *model fixtures*, not proof of actual Git DAG traversal or a
//! reproducible virtual-base merge. A real integration planner must separately
//! establish complete best-base discovery under its effective Git history view.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const HISTORICAL_CASE_COUNT: usize = 12;
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

fn nonblank(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(format!("{field} must be nonblank without control bytes"));
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

fn main() {
    let cases: Vec<Case> = match serde_json::from_str(FIXTURES) {
        Ok(cases) => cases,
        Err(error) => {
            eprintln!("merge-base-topology fixture parse failed: {error}");
            std::process::exit(1);
        }
    };
    match check(&cases) {
        Ok(count) => println!("merge-base-topology: {count} read-only model fixtures PASS"),
        Err(errors) => {
            for error in errors {
                eprintln!("merge-base-topology: {error}");
            }
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
    fn duplicate_case_name_is_rejected() {
        let mut cases = baseline();
        cases[1].name = cases[0].name.clone();
        assert!(check(&cases).is_err());
    }
}
