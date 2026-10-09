//! Standalone, typed oracle for the Phase0 coordination-history-spec5 fixtures.
//!
//! This evaluates eligibility and reconciliation from input facts, not expected
//! fixture labels or case identifiers. It is NOT a safe archive/deletion engine.
//! Missing fields in scenario overrides inherit the all-current baseline; that
//! inheritance applies only to this historical test fixture, not live evidence.

use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;

#[cfg(test)]
const BASELINE: &str = include_str!("../../../fixtures/coordination-history-spec5.json");
const FIXTURE_SCHEMA: &str = "fleet-control/coordination-history-spec5-fixtures/v1";
const MINIMUM_CASES: usize = 18;
const REQUIRED_HISTORICAL_CASE_IDS: &[&str] = &[
    "positive-all-current",
    "opaque-id-not-order",
    "manifest-competing-successors",
    "manifest-rollback",
    "mixed-cut-omission",
    "mixed-cut-overlap-dedup",
    "source-version-moved",
    "authority-revoked",
    "retention-expires-before-horizon",
    "key-custody-lost",
    "caller-self-weakens-horizon",
    "authorized-future-only-downgrade",
    "authorized-bounded-retirement",
    "horizon-rollback-after-downgrade",
    "lost-ack-manifest-reconcile",
    "partial-delete-retry",
    "protected-active-owner",
    "incomplete-pagination",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Verdict {
    Eligible,
    Ineligible,
    Reconcile,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Facts {
    archive_exact: Option<bool>,
    order_current: Option<bool>,
    order_evidence: Option<String>,
    manifest_current: Option<bool>,
    manifest_state: Option<String>,
    manifest_transition_outcome: Option<String>,
    snapshot_coherent: Option<bool>,
    snapshot_state: Option<String>,
    stable_record_identity: Option<bool>,
    archive_live_overlap: Option<bool>,
    source_current: Option<bool>,
    source_state: Option<String>,
    authority_current: Option<bool>,
    horizon_current: Option<bool>,
    horizon_state: Option<String>,
    predecessor_record_retired: Option<bool>,
    durability_current: Option<bool>,
    durability_state: Option<String>,
    protected: Option<bool>,
    // Descriptive receipt metadata; not independent authorization evidence.
    #[allow(dead_code)]
    protected_reason: Option<String>,
    partial_effect: Option<bool>,
    #[allow(dead_code)]
    retry: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    id: String,
    expect: Verdict,
    facts: Facts,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    schema: String,
    issue: u32,
    cases: Vec<Scenario>,
}

fn evaluate(f: &Facts) -> Verdict {
    // Unknown commit acknowledgement and partially applied deletion must be
    // reconciled against authoritative source/ref state BEFORE any retry.
    if f.manifest_transition_outcome.as_deref() == Some("unknown") || f.partial_effect == Some(true)
    {
        return Verdict::Reconcile;
    }
    // A made-up acknowledgement value must not inherit an all-current
    // historical baseline. Only the explicitly modeled unknown outcome
    // triggers reconciliation; every other supplied value fails closed.
    if f.manifest_transition_outcome.is_some() {
        return Verdict::Ineligible;
    }
    // Baseline inheritance is for sparse historical fixture scenarios ONLY.
    let current = [
        f.archive_exact,
        f.order_current,
        f.manifest_current,
        f.snapshot_coherent,
        f.source_current,
        f.authority_current,
        f.horizon_current,
        f.durability_current,
    ]
    .iter()
    .all(|value| value.unwrap_or(true));
    if !current || f.protected.unwrap_or(false) {
        return Verdict::Ineligible;
    }
    // A caller cannot invent an order or claim coherent replay from an opaque
    // identifier; mixed archive/live overlap needs stable deduplication keys.
    if f.order_evidence.is_some()
        || (f.archive_live_overlap == Some(true) && f.stable_record_identity != Some(true))
    {
        return Verdict::Ineligible;
    }
    // These markers all denote stale/conflicting state, never proof of a safe cut.
    if f.manifest_state.is_some()
        || f.snapshot_state.is_some()
        || f.source_state.is_some()
        || f.durability_state.is_some()
    {
        return Verdict::Ineligible;
    }
    match f.horizon_state.as_deref() {
        None => {}
        Some("shorter_successor") if f.predecessor_record_retired == Some(true) => {}
        _ => return Verdict::Ineligible,
    }
    Verdict::Eligible
}

fn check_baseline(suite: &Suite, failures: &mut Vec<String>) {
    let Some(base) = suite
        .cases
        .iter()
        .find(|case| case.id == "positive-all-current")
    else {
        failures.push("missing all-current positive baseline".to_owned());
        return;
    };
    // Every inherited "true" MUST be explicitly asserted by the source
    // positive control; otherwise sparse scenarios could silently pass.
    let facts = &base.facts;
    if [
        facts.archive_exact,
        facts.order_current,
        facts.manifest_current,
        facts.snapshot_coherent,
        facts.source_current,
        facts.authority_current,
        facts.horizon_current,
        facts.durability_current,
    ]
    .iter()
    .any(|x| *x != Some(true))
        || facts.protected != Some(false)
    {
        failures.push("all-current baseline missing explicit safety prerequisites".to_owned());
    }
}

fn require_original_scenario_ids(suite: &Suite, failures: &mut Vec<String>) {
    // A same-count replacement must not erase a historical negative scenario.
    // Additive scenarios are permitted; original identity is not fixture-controlled.
    let present: HashSet<&str> = suite.cases.iter().map(|case| case.id.as_str()).collect();
    for required in REQUIRED_HISTORICAL_CASE_IDS {
        if !present.contains(required) {
            failures.push(format!(
                "missing required historical scenario id: {required}"
            ));
        }
    }
}

fn validate(suite: &Suite) -> Result<usize, Vec<String>> {
    let mut failures = Vec::new();
    if suite.schema != FIXTURE_SCHEMA || suite.issue != 22 {
        failures.push(format!(
            "unsupported fixture schema/issue: {}/{}",
            suite.schema, suite.issue
        ));
    }
    if suite.cases.len() < MINIMUM_CASES {
        failures.push(format!(
            "incomplete coordination-history fixture: expected at least {MINIMUM_CASES}, found {}",
            suite.cases.len()
        ));
    }
    check_baseline(suite, &mut failures);
    require_original_scenario_ids(suite, &mut failures);
    let mut ids = HashSet::new();
    for case in &suite.cases {
        if case.id.trim().is_empty() || !ids.insert(case.id.as_str()) {
            failures.push(format!("duplicate or blank scenario id: {}", case.id));
        }
        let actual = evaluate(&case.facts);
        if actual != case.expect {
            failures.push(format!(
                "{}: expected {:?}; calculated {:?}",
                case.id, case.expect, actual
            ));
        }
    }
    failures.sort();
    if failures.is_empty() {
        Ok(suite.cases.len())
    } else {
        Err(failures)
    }
}

const MAX_FIXTURE_BYTES: u64 = 8 * 1024 * 1024;

/// Advisory fixture input only. The on-disk path and its contents remain
/// caller-controlled and cannot establish provider history or mutation rights.
/// A bounded read must not hang on a substituted FIFO or trust a prior stat
/// after the pathname has been replaced.
fn read_fixture_with_observed_metadata(
    path: &Path,
    observed: &fs::Metadata,
) -> Result<String, String> {
    if !observed.file_type().is_file() {
        return Err("fixture must be a regular, non-symlink file".to_owned());
    }
    if observed.len() > MAX_FIXTURE_BYTES {
        return Err("fixture exceeds the 8 MiB input limit".to_owned());
    }

    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Prevent an open-time symlink substitution and avoid blocking on a
        // FIFO that replaces the regular file after the initial stat.
        const O_NONBLOCK: i32 = 0o4000;
        const O_NOFOLLOW: i32 = 0o400000;
        options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("cannot open fixture: {error}"))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("cannot stat opened fixture: {error}"))?;
    if !opened.file_type().is_file() {
        return Err("opened fixture is not a regular file".to_owned());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if opened.dev() != observed.dev() || opened.ino() != observed.ino() {
            return Err("fixture replaced between metadata check and open".to_owned());
        }
    }
    if opened.len() > MAX_FIXTURE_BYTES {
        return Err("opened fixture exceeds the 8 MiB input limit".to_owned());
    }

    // Check bytes actually read, not just pre-open metadata: a concurrent
    // writer could extend the file after both stat observations.
    let mut bytes = Vec::new();
    file.take(MAX_FIXTURE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read opened fixture: {error}"))?;
    if bytes.len() as u64 > MAX_FIXTURE_BYTES {
        return Err("fixture exceeds the 8 MiB input limit".to_owned());
    }
    String::from_utf8(bytes).map_err(|_| "fixture must be UTF-8".to_owned())
}

fn read_fixture_bounded(path: &Path) -> Result<String, String> {
    let observed = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot stat fixture: {error}"))?;
    read_fixture_with_observed_metadata(path, &observed)
}

fn execute() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Phase0/fixtures/coordination-history-spec5.json"));
    if args.next().is_some() {
        return Err("usage: coordination_history [fixtures.json]".to_owned());
    }
    let input = read_fixture_bounded(&path)?;
    let suite: Suite = serde_json::from_str(&input)
        .map_err(|e| format!("malformed coordination-history fixture: {e}"))?;
    match validate(&suite) {
        Ok(n) => {
            println!("PASS: {n} independently evaluated coordination-history cases");
            Ok(())
        }
        Err(failures) => Err(failures.join("\n")),
    }
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> Suite {
        serde_json::from_str(BASELINE).expect("checked-in fixture must parse")
    }

    #[test]
    fn all_historical_cases_have_independent_verdicts() {
        assert_eq!(validate(&source()), Ok(18));
    }

    #[test]
    fn wrong_expected_verdict_cannot_self_validate() {
        let mut suite = source();
        suite.cases[0].expect = Verdict::Ineligible;
        assert!(validate(&suite).is_err());
    }

    #[test]
    fn missing_family_or_baseline_is_not_a_pass() {
        let mut suite = source();
        suite.cases.clear();
        let errors = validate(&suite).unwrap_err().join(" ");
        assert!(errors.contains("incomplete"));
        assert!(errors.contains("missing all-current"));
    }

    #[test]
    fn each_same_count_replacement_of_original_scenario_is_rejected() {
        for old_id in REQUIRED_HISTORICAL_CASE_IDS {
            let mut suite = source();
            let case = suite
                .cases
                .iter_mut()
                .find(|case| case.id == *old_id)
                .expect("historical case must be present");
            case.id = format!("forged-replacement-for-{old_id}");
            let failures = validate(&suite).expect_err("renamed original must fail");
            let missing = format!("missing required historical scenario id: {old_id}");
            assert!(
                failures.iter().any(|failure| failure == &missing),
                "{old_id}: {failures:?}"
            );
        }
    }

    #[test]
    fn duplicate_scenario_identity_is_rejected() {
        let mut suite = source();
        suite.cases.push(suite.cases[0].clone());
        assert!(validate(&suite)
            .unwrap_err()
            .join(" ")
            .contains("duplicate"));
    }

    #[test]
    fn revoked_authority_invalidates_an_otherwise_eligible_cut() {
        let mut suite = source();
        suite.cases[0].facts.authority_current = Some(false);
        assert!(validate(&suite).is_err());
    }

    #[test]
    fn overlap_without_stable_record_identity_is_ineligible() {
        let mut suite = source();
        suite.cases[0].facts.archive_live_overlap = Some(true);
        suite.cases[0].facts.stable_record_identity = Some(false);
        assert!(validate(&suite).is_err());
    }

    #[test]
    fn shorter_horizon_requires_explicit_bounded_retirement() {
        let mut suite = source();
        suite.cases[0].facts.horizon_state = Some("shorter_successor".to_owned());
        suite.cases[0].facts.predecessor_record_retired = Some(false);
        assert!(validate(&suite).is_err());
        suite.cases[0].facts.predecessor_record_retired = Some(true);
        assert_eq!(validate(&suite), Ok(18));
    }

    #[test]
    fn acknowledgement_loss_requires_reconciliation() {
        let mut suite = source();
        suite.cases[0].facts.manifest_transition_outcome = Some("unknown".to_owned());
        assert_eq!(evaluate(&suite.cases[0].facts), Verdict::Reconcile);
        assert!(validate(&suite).is_err());
    }

    #[test]
    fn false_or_missing_positive_control_is_not_inherited() {
        let mut suite = source();
        suite.cases[0].facts.archive_exact = None;
        assert!(validate(&suite)
            .unwrap_err()
            .join(" ")
            .contains("baseline missing"));
    }

    #[test]
    fn invalid_inputs_fail_parsing() {
        let mut raw: serde_json::Value = serde_json::from_str(BASELINE).unwrap();
        raw["cases"][0]["facts"]["archive_exact"] = serde_json::json!("true");
        assert!(serde_json::from_value::<Suite>(raw).is_err());
        let mut raw: serde_json::Value = serde_json::from_str(BASELINE).unwrap();
        raw["cases"][0]["facts"]["not_a_real_fact"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Suite>(raw).is_err());
    }

    #[test]
    fn version_mismatch_rejected() {
        let mut suite = source();
        suite.issue = 23;
        assert!(validate(&suite)
            .unwrap_err()
            .join(" ")
            .contains("unsupported"));
    }

    #[test]
    fn fixture_reader_rejects_nonregular_oversized_and_non_utf8_inputs() {
        let root = env::temp_dir().join(format!(
            "free-energy-history-reader-{}",
            process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let valid = root.join("valid.json");
        fs::write(&valid, BASELINE).unwrap();
        assert_eq!(read_fixture_bounded(&valid).unwrap(), BASELINE);
        assert!(read_fixture_bounded(&root).is_err());

        let oversized = root.join("oversized.json");
        fs::File::create(&oversized)
            .unwrap()
            .set_len(MAX_FIXTURE_BYTES + 1)
            .unwrap();
        assert!(read_fixture_bounded(&oversized)
            .unwrap_err()
            .contains("8 MiB"));
        let non_utf8 = root.join("non-utf8.json");
        fs::write(&non_utf8, [0xff, 0xfe]).unwrap();
        assert!(read_fixture_bounded(&non_utf8)
            .unwrap_err()
            .contains("UTF-8"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn fixture_reader_rejects_same_path_inode_swap_and_symlink_replacement() {
        let root = env::temp_dir().join(format!(
            "free-energy-history-inode-{}",
            process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("history.json");
        let next = root.join("next.json");
        fs::write(&path, BASELINE).unwrap();
        fs::write(&next, BASELINE).unwrap();
        let observed = fs::symlink_metadata(&path).unwrap();
        fs::rename(&next, &path).unwrap();
        assert_eq!(
            read_fixture_with_observed_metadata(&path, &observed).unwrap_err(),
            "fixture replaced between metadata check and open"
        );

        let current = fs::symlink_metadata(&path).unwrap();
        let target = root.join("target.json");
        fs::write(&target, BASELINE).unwrap();
        fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(read_fixture_with_observed_metadata(&path, &current).is_err());
        assert!(read_fixture_bounded(&path).is_err());
        fs::remove_dir_all(root).unwrap();
    }

}
