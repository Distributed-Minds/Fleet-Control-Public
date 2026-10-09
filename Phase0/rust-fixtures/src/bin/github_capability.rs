//! Partial typed Rust migration of the GitHub capability spec-5 fixture checks.
//!
//! This is a fixture *safety-invariant validator*, not a full semantic oracle,
//! not a capability probe, and not permission to mutate GitHub. The historical
//! 28-case fixture omits some causal inputs needed to infer all expected labels
//! independently (e.g. several identical inputs have different expected states).
//! Do not report Python/Rust full semantic equivalence until those inputs exist.

use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Lineage {
    Current,
    Unknown,
    Conflicting,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Authority {
    Current,
    None,
    Stale,
    Forged,
    Incompatible,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Resource {
    Exact,
    None,
    Ambiguous,
    AbsentWithReceipt,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Action {
    ScheduledWrite,
    DefaultBranchWrite,
    Cleanup,
    Reconcile,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Approval {
    Required,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Verdict {
    CapabilityAbsent,
    ActionPaused,
    ActionBlocked,
    Revalidate,
    CleanupRequired,
    ReuseAttempt,
    RecoveryRequired,
    Ambiguous,
    PassedClean,
    AllowFenced,
    AuthorityStale,
    AuthorityUnavailable,
    AllowRecovery,
    ReuseLineage,
    LineageUnknown,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    spec: u8,
    issue: u8,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: u8,
    name: String,
    lineage: Lineage,
    authority: Authority,
    resource: Resource,
    action: Action,
    #[serde(default)]
    approval: Option<Approval>,
    #[serde(default)]
    recovery: bool,
    #[serde(default)]
    fenced: bool,
    #[serde(default)]
    successor_evidence: bool,
    expected: Verdict,
}

// Preserve the original Python fixture's 28 immutable historical labels.
// These are integrity baselines, not a computed semantic verdict oracle.
const CANONICAL_NAMES: [&str; 28] = [
    "interactive-write-scheduled-read-only",
    "interactive-action-scheduled-missing",
    "scheduled-stricter-approval",
    "repository-scope-differs",
    "capability-evidence-drift",
    "connector-without-mutation-action",
    "repository-policy-rejects-action",
    "probe-would-touch-default-branch",
    "cutoff-after-create",
    "cutoff-before-cleanup",
    "duplicate-retry",
    "cleanup-permission-lost",
    "locator-reused",
    "cleanup-ack-lost",
    "repeated-cutoff-bounded",
    "concurrent-generations-one-namespace",
    "old-probe-after-successor",
    "stale-probe-cleanup-without-transfer",
    "authority-lost-before-cleanup",
    "forged-authority-metadata",
    "bounded-recovery-transfer",
    "authority-adapter-incompatible",
    "capability-current-authority-expired",
    "context-recreated-successor",
    "context-id-reused-different-installation",
    "product-migration-access-only",
    "successor-orphan-cleanup-needs-transfer",
    "lineage-map-version-skew",
];

const CANONICAL_VERDICTS: [Verdict; 28] = [
    Verdict::CapabilityAbsent,
    Verdict::CapabilityAbsent,
    Verdict::ActionPaused,
    Verdict::ActionBlocked,
    Verdict::Revalidate,
    Verdict::CapabilityAbsent,
    Verdict::ActionBlocked,
    Verdict::ActionBlocked,
    Verdict::CleanupRequired,
    Verdict::CleanupRequired,
    Verdict::ReuseAttempt,
    Verdict::RecoveryRequired,
    Verdict::Ambiguous,
    Verdict::PassedClean,
    Verdict::CleanupRequired,
    Verdict::AllowFenced,
    Verdict::AuthorityStale,
    Verdict::RecoveryRequired,
    Verdict::RecoveryRequired,
    Verdict::AuthorityUnavailable,
    Verdict::AllowRecovery,
    Verdict::AuthorityUnavailable,
    Verdict::AuthorityStale,
    Verdict::ReuseLineage,
    Verdict::LineageUnknown,
    Verdict::LineageUnknown,
    Verdict::RecoveryRequired,
    Verdict::LineageUnknown,
];

fn check_case(c: &Case) -> Vec<String> {
    use Action::*;
    use Verdict::*;

    let mut errors = Vec::new();
    let expected = c.expected;
    let mut reject = |reason: &str| errors.push(format!("{}: {reason}", c.name));

    // Independently inspect semantic guard inputs before trusting the fixture's
    // expected output. These checks are necessary, not a sufficient result oracle.
    if c.action == DefaultBranchWrite && expected != ActionBlocked {
        reject("a default-branch probe must be ACTION_BLOCKED");
    }

    if c.approval == Some(Approval::Required) && expected != ActionPaused {
        reject("approval-required execution cannot claim autonomous success");
    }

    if c.lineage != Lineage::Current
        && matches!(c.action, ScheduledWrite | Cleanup)
        && expected != LineageUnknown
    {
        reject("mutation with unknown/conflicting lineage must fail closed");
    }

    if c.lineage == Lineage::Current && c.action == ScheduledWrite {
        match c.authority {
            Authority::Stale if expected != AuthorityStale => {
                reject("stale scheduled authority must be denied")
            }
            Authority::Forged | Authority::Incompatible if expected != AuthorityUnavailable => {
                reject("unverifiable scheduled authority cannot permit a write")
            }
            _ => {}
        }
    }

    if c.action == Cleanup && c.lineage == Lineage::Current && c.authority == Authority::Stale {
        let required = if c.recovery {
            AllowRecovery
        } else {
            RecoveryRequired
        };
        if expected != required {
            reject("stale cleanup requires exact bounded recovery transfer");
        }
    }

    if c.action == Cleanup && c.resource == Resource::Ambiguous && expected != Ambiguous {
        reject("ambiguous resource incarnation is not cleanup success");
    }

    // Historical labels must also have their necessary *input witnesses*.
    // Guarding only a changed expected label misses forged source facts that
    // retain the original case ID, name, and hardcoded expected disposition.
    match expected {
        ActionPaused => {
            if c.action != ScheduledWrite
                || c.lineage != Lineage::Current
                || c.authority != Authority::Current
                || c.resource != Resource::Exact
                || c.approval != Some(Approval::Required)
            {
                reject("ACTION_PAUSED requires the original approval-required scheduled write");
            }
        }
        AuthorityStale => {
            if c.action != ScheduledWrite
                || c.lineage != Lineage::Current
                || c.authority != Authority::Stale
                || c.resource != Resource::Exact
            {
                reject("AUTHORITY_STALE requires a scheduled write with stale authority");
            }
        }
        AuthorityUnavailable => {
            if c.action != ScheduledWrite
                || c.lineage != Lineage::Current
                || !matches!(c.authority, Authority::Forged | Authority::Incompatible)
                || c.resource != Resource::Exact
            {
                reject("AUTHORITY_UNAVAILABLE requires forged or incompatible write authority");
            }
        }
        AllowFenced => {
            if c.action != ScheduledWrite
                || c.lineage != Lineage::Current
                || c.authority != Authority::Current
                || c.resource != Resource::Exact
                || !c.fenced
                || c.approval.is_some()
            {
                reject(
                    "ALLOW_FENCED requires a fenced current scheduled write on an exact resource",
                );
            }
        }
        AllowRecovery => {
            if c.action != Cleanup
                || c.lineage != Lineage::Current
                || c.authority != Authority::Stale
                || c.resource != Resource::Exact
                || !c.recovery
                || c.approval.is_some()
            {
                reject("ALLOW_RECOVERY requires stale cleanup with explicit bounded transfer");
            }
        }
        PassedClean => {
            if c.action != Cleanup
                || c.lineage != Lineage::Current
                || c.authority != Authority::Current
                || c.resource != Resource::AbsentWithReceipt
                || c.approval.is_some()
            {
                reject("PASSED_CLEAN requires current authority and an exact cleanup receipt");
            }
        }
        ReuseLineage => {
            if c.action != Reconcile
                || c.lineage != Lineage::Current
                || c.authority != Authority::Current
                || c.resource != Resource::Exact
                || !c.successor_evidence
            {
                reject("REUSE_LINEAGE requires current authority, exact resource and successor evidence");
            }
        }
        _ => {}
    }

    errors
}

fn validate(f: &Fixture) -> Result<usize, Vec<String>> {
    let mut errors = Vec::new();
    if f.spec != 5 || f.issue != 11 {
        errors.push(format!("unsupported issue/spec: {}/{}", f.issue, f.spec));
    }

    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for c in &f.cases {
        if !(1..=28).contains(&c.id) || !ids.insert(c.id) {
            errors.push(format!("duplicate or invalid case id: {}", c.id));
        }
        if c.name.trim().is_empty() || !names.insert(c.name.as_str()) {
            errors.push(format!("duplicate or empty case name: {}", c.name));
        }
        errors.extend(check_case(c));
        if (1..=28).contains(&c.id) {
            let index = usize::from(c.id - 1);
            if c.name != CANONICAL_NAMES[index] {
                errors.push(format!("case {}: canonical scenario name mismatch", c.id));
            }
            if c.expected != CANONICAL_VERDICTS[index] {
                errors.push(format!(
                    "case {}: canonical expected outcome mismatch",
                    c.id
                ));
            }
        }
    }
    for id in 1..=28 {
        if !ids.contains(&id) {
            errors.push(format!("missing required case id: {id}"));
        }
    }
    errors.sort();

    if errors.is_empty() {
        Ok(f.cases.len())
    } else {
        Err(errors)
    }
}

const MAX_EXTERNAL_FIXTURE_BYTES: u64 = 8 * 1024 * 1024;

// Model-only fixture CLI: bound caller-controlled IO; this is not an
// atomic directory confinement protocol or a live provider authority check.
fn read_fixture_bounded(input: &Path) -> Result<String, String> {
    let before = fs::symlink_metadata(input)
        .map_err(|e| format!("cannot inspect {input:?}: {e}"))?;
    if !before.file_type().is_file() {
        return Err(format!("fixture {input:?} must be a regular file (no symlinks)"));
    }
    if before.len() > MAX_EXTERNAL_FIXTURE_BYTES {
        return Err(format!("fixture {input:?} exceeds {MAX_EXTERNAL_FIXTURE_BYTES} bytes"));
    }

    let file = File::open(input).map_err(|e| format!("cannot open {input:?}: {e}"))?;
    let opened = file
        .metadata()
        .map_err(|e| format!("cannot inspect opened fixture {input:?}: {e}"))?;
    if !opened.is_file() {
        return Err(format!("opened fixture {input:?} is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if opened.dev() != before.dev() || opened.ino() != before.ino() {
            return Err(format!("fixture identity changed before opening {input:?}"));
        }
    }
    if opened.len() > MAX_EXTERNAL_FIXTURE_BYTES {
        return Err(format!("opened fixture {input:?} exceeds {MAX_EXTERNAL_FIXTURE_BYTES} bytes"));
    }

    // The read cap also rejects data added between metadata checks and read.
    let mut json = String::new();
    file.take(MAX_EXTERNAL_FIXTURE_BYTES + 1)
        .read_to_string(&mut json)
        .map_err(|e| format!("cannot read {input:?}: {e}"))?;
    if json.len() as u64 > MAX_EXTERNAL_FIXTURE_BYTES {
        return Err(format!("fixture {input:?} exceeds {MAX_EXTERNAL_FIXTURE_BYTES} bytes"));
    }
    Ok(json)
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Phase0/fixtures/github-capability-spec5.json"));
    if args.next().is_some() {
        return Err("usage: github_capability [fixture.json]".to_owned());
    }
    let json = read_fixture_bounded(&input)?;
    let fixture: Fixture = serde_json::from_str(&json)
        .map_err(|e| format!("invalid fixture {input:?}: {e}"))?;
    match validate(&fixture) {
        Ok(count) => {
            println!("GitHub capability invariant fixtures (Rust): {count} checked");
            Ok(())
        }
        Err(errors) => Err(errors.join("\nFAIL: ")),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FAIL: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const BASELINE: &str = include_str!("../../../fixtures/github-capability-spec5.json");

    fn fixture() -> Value {
        serde_json::from_str(BASELINE).expect("original JSON fixture")
    }

    fn checked(v: Value) -> Result<usize, Vec<String>> {
        validate(&serde_json::from_value(v).expect("typed fixture mutation"))
    }

    fn mutate(id: u8, field: &str, value: Value) -> Value {
        let mut f = fixture();
        let case = f["cases"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["id"] == json!(id))
            .expect("case ID");
        case[field] = value;
        f
    }

    #[test]
    fn failure_labels_require_their_original_semantic_witnesses() {
        // A label/name-pinned fixture previously accepted these changed facts:
        // the hardcoded verdict never had to explain the changed inputs.
        for (id, field, changed) in [
            (3, "approval", Value::Null),
            (3, "action", json!("reconcile")),
            (17, "authority", json!("current")),
            (17, "resource", json!("none")),
            (23, "authority", json!("current")),
            (20, "authority", json!("current")),
            (22, "authority", json!("current")),
            (22, "action", json!("reconcile")),
        ] {
            assert!(
                checked(mutate(id, field, changed)).is_err(),
                "case {id}: changed {field} retained an unsupported historical verdict"
            );
        }
    }

    #[test]
    fn positive_receipt_and_lineage_reuse_need_current_authority_and_identity() {
        // Previously these mutated witnesses kept canonical names and expected
        // labels, so the invariant checker accepted forged "success" evidence.
        for (id, field, changed) in [
            (14, "authority", json!("none")),
            (14, "authority", json!("forged")),
            (14, "authority", json!("incompatible")),
            (24, "authority", json!("none")),
            (24, "authority", json!("forged")),
            (24, "authority", json!("incompatible")),
            (24, "resource", json!("none")),
            (24, "resource", json!("ambiguous")),
        ] {
            assert!(
                checked(mutate(id, field, changed)).is_err(),
                "case {id}: changed {field} kept a success-like verdict without its witness"
            );
        }
        // Both unchanged historical positives remain admitted.
        assert_eq!(checked(fixture()), Ok(28));
    }

    #[test]
    fn original_twenty_eight_cases_pass_invariant_checks() {
        assert_eq!(checked(fixture()), Ok(28));
    }

    #[test]
    fn deny_recovery_for_unprivileged_scheduled_write() {
        assert!(checked(mutate(1, "expected", json!("ALLOW_RECOVERY"))).is_err());
    }

    #[test]
    fn invented_success_is_not_a_valid_verdict() {
        let changed = mutate(3, "expected", json!("INVENTED_SUCCESS"));
        assert!(serde_json::from_value::<Fixture>(changed).is_err());
    }

    #[test]
    fn reuse_lineage_without_successor_proof_is_rejected() {
        assert!(checked(mutate(24, "successor_evidence", json!(false))).is_err());
    }

    #[test]
    fn missing_fence_cannot_be_allow_fenced() {
        assert!(checked(mutate(16, "fenced", json!(false))).is_err());
    }

    #[test]
    fn clean_receipt_cannot_be_replaced_with_absence_without_proof() {
        assert!(checked(mutate(14, "resource", json!("none"))).is_err());
    }

    #[test]
    fn stale_cleanup_without_recovery_transfer_is_rejected() {
        assert!(checked(mutate(21, "recovery", json!(false))).is_err());
    }

    #[test]
    fn stale_scheduled_write_cannot_be_allow_fenced() {
        assert!(checked(mutate(16, "authority", json!("stale"))).is_err());
    }

    #[test]
    fn deleted_case_rejects_incomplete_family() {
        let mut changed = fixture();
        changed["cases"].as_array_mut().unwrap().pop();
        assert!(checked(changed).is_err());
    }

    #[test]
    fn duplicate_case_identity_is_rejected() {
        let mut changed = fixture();
        changed["cases"][1]["id"] = json!(1);
        assert!(checked(changed).is_err());
    }

    #[test]
    fn unknown_input_type_is_rejected() {
        let changed = mutate(16, "fenced", json!("true"));
        assert!(serde_json::from_value::<Fixture>(changed).is_err());
    }

    #[test]
    fn approval_required_cannot_be_treated_as_success() {
        assert!(checked(mutate(3, "expected", json!("REVALIDATE"))).is_err());
    }

    #[test]
    fn historical_names_and_outcomes_cannot_be_forged() {
        let renamed = mutate(1, "name", json!("forged-scenario"));
        assert!(checked(renamed).is_err());

        // Case 1 previously had no input rule pinning this false outcome.
        let changed_outcome = mutate(1, "expected", json!("ACTION_BLOCKED"));
        assert!(checked(changed_outcome).is_err());
    }
}
