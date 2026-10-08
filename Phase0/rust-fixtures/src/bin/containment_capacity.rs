//! Independent typed semantic oracle for the historical containment-capacity spec v2.
//! Offline fixture verification only: this does not enforce live runtime policy.

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    spec_version: u32,
    decision_cases: Vec<Decision>,
    workload_cases: Vec<Workload>,
    planning_cases: Vec<Planning>,
}

#[derive(Debug, Deserialize)]
struct Decision {
    id: String,
    #[serde(default = "yes")]
    policy_compatible: bool,
    #[serde(default)]
    duplicate_case_ids: Vec<String>,
    #[serde(default)]
    renewal_attempt: bool,
    #[serde(default)]
    renewal_evidence_current: bool,
    #[serde(default = "yes")]
    authority_current: bool,
    #[serde(default)]
    effect_active: bool,
    #[serde(default)]
    restoration_debt: bool,
    expected: String,
}

fn decide(case: &Decision) -> &'static str {
    if !case.policy_compatible {
        return "REQUIRE_MIGRATION";
    }
    let mut ids = HashSet::new();
    if case.duplicate_case_ids.iter().any(|id| !ids.insert(id)) {
        return "RECONCILE_DUPLICATE";
    }
    if case.renewal_attempt && !case.renewal_evidence_current {
        return "REJECT_RENEWAL";
    }
    if !case.authority_current && case.effect_active && !case.restoration_debt {
        return "INVALID_MISSING_DEBT";
    }
    "OK"
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Capacity {
    Constant(i64),
    PerTick(Vec<i64>),
}

fn capacities(input: &Option<Capacity>, ticks: usize) -> Result<Vec<i64>, String> {
    let values = match input {
        None => vec![0; ticks],
        Some(Capacity::Constant(cap)) => vec![*cap; ticks],
        Some(Capacity::PerTick(values)) if values.len() == ticks => values.clone(),
        Some(Capacity::PerTick(values)) => {
            return Err(format!(
                "capacity trace length {} != arrival count {ticks}",
                values.len()
            ));
        }
    };
    if values.iter().any(|v| *v < 0) {
        return Err("negative service capacity".to_owned());
    }
    Ok(values)
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum CapacityMode {
    #[default]
    Shared,
    Separate,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Scheduler {
    #[default]
    AdjudicationFirst,
    RestorationFirst,
    DeadlineSafe,
}

#[derive(Debug, Deserialize)]
struct Workload {
    id: String,
    #[serde(default)]
    authority_current: bool,
    #[serde(default)]
    effect_active: bool,
    #[serde(default)]
    restoration_debt: bool,
    #[serde(default)]
    initial_adjudication_backlog: i64,
    #[serde(default)]
    initial_restoration_work: i64,
    arrivals: Vec<i64>,
    service_capacity: Option<Capacity>,
    adjudication_capacity: Option<Capacity>,
    restoration_capacity: Option<Capacity>,
    #[serde(default)]
    capacity_mode: CapacityMode,
    #[serde(default)]
    scheduler: Scheduler,
    progress_bound: usize,
    #[serde(default)]
    fail_safe_escalates: bool,
    expected: Value,
}

fn simulate(c: &Workload) -> Result<Value, String> {
    let ticks = c.arrivals.len();
    if ticks == 0 || c.progress_bound == 0 {
        return Err("arrival trace and progress bound must be nonempty/positive".to_owned());
    }
    if c.initial_adjudication_backlog < 0
        || c.initial_restoration_work < 0
        || c.arrivals.iter().any(|n| *n < 0)
    {
        return Err("negative arrivals, backlog, or restoration work".to_owned());
    }

    let mut backlog = c.initial_adjudication_backlog;
    let mut restoration = c.initial_restoration_work;
    let mut effect_active = c.effect_active;
    if restoration > 0 && (c.authority_current || !effect_active || !c.restoration_debt) {
        return Err(
            "restoration work requires expired authority, active effect and debt".to_owned(),
        );
    }
    if !c.authority_current && effect_active && !c.restoration_debt {
        return Ok(json!({"disposition": "INVALID_MISSING_DEBT"}));
    }
    if !c.authority_current && !effect_active {
        let caps = capacities(&c.service_capacity, ticks)?;
        for (arrival, cap) in c.arrivals.iter().zip(caps.iter()) {
            backlog = backlog
                .checked_add(*arrival)
                .ok_or("backlog overflow")?
                .saturating_sub(*cap)
                .max(0);
        }
        return Ok(json!({"disposition": "RESTORED", "final_adjudication_backlog": backlog}));
    }

    let shared = if c.capacity_mode == CapacityMode::Shared {
        capacities(&c.service_capacity, ticks)?
    } else {
        Vec::new()
    };
    let adjudication = if c.capacity_mode == CapacityMode::Separate {
        capacities(&c.adjudication_capacity, ticks)?
    } else {
        Vec::new()
    };
    let restorative = if c.capacity_mode == CapacityMode::Separate {
        capacities(&c.restoration_capacity, ticks)?
    } else {
        Vec::new()
    };

    let mut first_progress: Option<usize> = None;
    let mut restored_by: Option<usize> = None;
    let mut since_progress: usize = 0;
    let mut overload_seen = false;
    for (offset, arrival) in c.arrivals.iter().enumerate() {
        let tick = offset + 1;
        backlog = backlog.checked_add(*arrival).ok_or("backlog overflow")?;
        if restoration > 0 {
            since_progress += 1;
        }
        let mut restored_units = 0i64;
        if c.capacity_mode == CapacityMode::Separate {
            let cap_r = restorative[offset];
            let cap_a = adjudication[offset];
            if restoration > 0 && cap_r > 0 {
                restored_units = restoration.min(cap_r);
                restoration -= restored_units;
            }
            backlog = backlog.saturating_sub(cap_a).max(0);
            overload_seen |= *arrival > cap_a;
        } else {
            let cap = shared[offset];
            let mut remaining = cap;
            if restoration > 0 && remaining > 0 {
                let due = since_progress >= c.progress_bound;
                if c.scheduler == Scheduler::RestorationFirst
                    || (c.scheduler == Scheduler::DeadlineSafe && due)
                {
                    restored_units = 1;
                    restoration -= 1;
                    remaining -= 1;
                }
            }
            if remaining > 0 {
                let served = backlog.min(remaining);
                backlog -= served;
                remaining -= served;
            }
            if restoration > 0 && remaining > 0 && c.scheduler == Scheduler::RestorationFirst {
                let extra = restoration.min(remaining);
                restoration -= extra;
                restored_units += extra;
            }
            overload_seen |= *arrival > cap;
        }
        if restored_units > 0 {
            first_progress.get_or_insert(tick);
            since_progress = 0;
            if restoration == 0 {
                restored_by = Some(tick);
                effect_active = false;
            }
        }
        if restoration > 0 && since_progress >= c.progress_bound {
            return Ok(json!({
                "disposition": if c.fail_safe_escalates {
                    "DEGRADED_ESCALATION"
                } else {
                    "INVALID_STARVATION"
                },
                "escalation_tick": tick,
                "first_progress_tick": first_progress,
                "final_adjudication_backlog": backlog
            }));
        }
    }

    if let Some(tick) = restored_by {
        Ok(json!({
            "disposition": "RESTORED",
            "first_progress_tick": first_progress,
            "restored_by_tick": tick,
            "final_adjudication_backlog": backlog
        }))
    } else if restoration > 0 && first_progress.is_some() {
        Ok(json!({
            "disposition": "RESTORE_PROGRESS",
            "first_progress_tick": first_progress,
            "final_adjudication_backlog": backlog
        }))
    } else if c.authority_current {
        Ok(json!({
            "disposition": if overload_seen || backlog > 0 {"OVERLOAD_VISIBLE"} else {"OK"},
            "final_adjudication_backlog": backlog
        }))
    } else if restoration == 0 && !effect_active {
        Ok(json!({"disposition": "RESTORED", "final_adjudication_backlog": backlog}))
    } else {
        Ok(json!({"disposition": "INVALID_STARVATION", "final_adjudication_backlog": backlog}))
    }
}

#[derive(Debug, Deserialize)]
struct Planning {
    id: String,
    worker_count: u64,
    event_volume: u64,
    true_events: u64,
    sensitivity_num: u64,
    sensitivity_den: u64,
    false_positive_num: u64,
    false_positive_den: u64,
    expected_alerts: u64,
    expected_false_alerts: u64,
}

fn alert_volume(c: &Planning) -> Result<(u64, u64), String> {
    if c.true_events > c.event_volume
        || c.sensitivity_den == 0
        || c.false_positive_den == 0
        || c.sensitivity_num > c.sensitivity_den
        || c.false_positive_num > c.false_positive_den
    {
        return Err("invalid event count or rate".to_owned());
    }
    let true_product = u128::from(c.true_events) * u128::from(c.sensitivity_num);
    let false_product =
        u128::from(c.event_volume - c.true_events) * u128::from(c.false_positive_num);
    if true_product % u128::from(c.sensitivity_den) != 0
        || false_product % u128::from(c.false_positive_den) != 0
    {
        return Err("fractional deterministic alert count".to_owned());
    }
    let true_alerts = true_product / u128::from(c.sensitivity_den);
    let false_alerts = false_product / u128::from(c.false_positive_den);
    let total = u64::try_from(true_alerts + false_alerts).map_err(|_| "alert count overflow")?;
    let false_alerts = u64::try_from(false_alerts).map_err(|_| "false-alert overflow")?;
    Ok((total, false_alerts))
}

fn validate(f: &Fixture) -> Result<usize, Vec<String>> {
    let mut failures = Vec::new();
    if f.schema_version != 2 || f.spec_version != 2 {
        failures.push("unsupported containment-capacity schema/spec".to_owned());
    }
    for (name, found, minimum) in [
        ("decision", f.decision_cases.len(), 4),
        ("workload", f.workload_cases.len(), 9),
        ("planning", f.planning_cases.len(), 4),
    ] {
        if found < minimum {
            failures.push(format!("incomplete {name} family: {found} < {minimum}"));
        }
    }
    let mut seen = HashSet::new();
    for id in f
        .decision_cases
        .iter()
        .map(|c| &c.id)
        .chain(f.workload_cases.iter().map(|c| &c.id))
        .chain(f.planning_cases.iter().map(|c| &c.id))
    {
        if id.trim().is_empty() || !seen.insert(id) {
            failures.push(format!("duplicate or empty case id: {id}"));
        }
    }

    for c in &f.decision_cases {
        let actual = decide(c);
        if actual != c.expected {
            failures.push(format!(
                "{}: expected {}, computed {actual}",
                c.id, c.expected
            ));
        }
    }
    for c in &f.workload_cases {
        match simulate(c) {
            Ok(actual) => {
                let Some(expected) = c.expected.as_object() else {
                    failures.push(format!("{}: expected must be a JSON object", c.id));
                    continue;
                };
                // Sparse historical fixture expectations are intentional, but
                // a disposition-only assertion must not certify restoration timing,
                // starvation deadlines, or backlog accounting. Require the
                // minimum observable metrics for the computed state.
                let required: &[&str] = match actual.get("disposition").and_then(Value::as_str) {
                    Some("RESTORED") if c.initial_restoration_work > 0 => {
                        &["disposition", "first_progress_tick", "restored_by_tick"]
                    }
                    Some("INVALID_STARVATION" | "DEGRADED_ESCALATION") => {
                        &["disposition", "escalation_tick"]
                    }
                    Some("RESTORE_PROGRESS") => &["disposition", "first_progress_tick"],
                    Some("OK" | "OVERLOAD_VISIBLE" | "RESTORED") => {
                        &["disposition", "final_adjudication_backlog"]
                    }
                    _ => &["disposition"],
                };
                for key in required {
                    if !expected.contains_key(*key) {
                        failures.push(format!("{}: missing expected {key}", c.id));
                    }
                }
                for (key, value) in expected {
                    if actual.get(key) != Some(value) {
                        failures.push(format!(
                            "{}: expected {key}={value}, computed {:?}",
                            c.id,
                            actual.get(key)
                        ));
                    }
                }
            }
            Err(reason) => failures.push(format!("{}: {reason}", c.id)),
        }
    }
    for c in &f.planning_cases {
        match alert_volume(c) {
            Ok((alerts, false_alerts)) => {
                if alerts != c.expected_alerts || false_alerts != c.expected_false_alerts {
                    failures.push(format!(
                        "{}: expected {}/{}, computed {alerts}/{false_alerts}",
                        c.id, c.expected_alerts, c.expected_false_alerts
                    ));
                }
            }
            Err(reason) => failures.push(format!("{}: {reason}", c.id)),
        }
    }

    let low = f
        .planning_cases
        .iter()
        .find(|c| c.id == "low-action-volume");
    let high = f
        .planning_cases
        .iter()
        .find(|c| c.id == "high-action-volume");
    match (low, high) {
        (Some(low), Some(high)) => match (alert_volume(low), alert_volume(high)) {
            (Ok((low_alerts, _)), Ok((high_alerts, _))) => {
                if low.worker_count != high.worker_count
                    || high.event_volume <= low.event_volume
                    || high_alerts <= low_alerts
                {
                    failures.push("action-volume monotonicity violated".to_owned());
                }
            }
            _ => failures.push("cannot verify action-volume property".to_owned()),
        },
        _ => failures.push("missing action-volume property cases".to_owned()),
    }
    match f
        .planning_cases
        .iter()
        .find(|c| c.id == "rare-event-false-positive-heavy")
    {
        Some(rare) => match alert_volume(rare) {
            Ok((alerts, false_alerts)) if false_alerts > alerts - false_alerts => {}
            _ => failures.push("rare-event false-positive-heavy property violated".to_owned()),
        },
        None => failures.push("missing rare-event planning property case".to_owned()),
    }

    failures.sort();
    if failures.is_empty() {
        Ok(f.decision_cases.len() + f.workload_cases.len() + f.planning_cases.len())
    } else {
        Err(failures)
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Phase0/fixtures/containment-capacity-spec2.json"));
    if args.next().is_some() {
        return Err("usage: containment_capacity [fixture.json]".to_owned());
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let fixture: Fixture = serde_json::from_str(&source)
        .map_err(|error| format!("invalid fixture {}: {error}", path.display()))?;
    match validate(&fixture) {
        Ok(count) => {
            println!("containment-capacity fixtures (Rust): {count} passed");
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
    const BASELINE: &str = include_str!("../../../fixtures/containment-capacity-spec2.json");

    fn original() -> Value {
        serde_json::from_str(BASELINE).expect("valid legacy fixture")
    }

    fn mutated(value: Value) -> Result<usize, Vec<String>> {
        let typed: Fixture = serde_json::from_value(value).expect("typed mutation");
        validate(&typed)
    }

    #[test]
    fn original_seventeen_semantic_cases_pass() {
        let fixture: Fixture = serde_json::from_str(BASELINE).expect("typed baseline");
        assert_eq!(validate(&fixture), Ok(17));
    }

    #[test]
    fn sparse_expectations_cannot_skip_material_capacity_semantics() {
        // Each omission previously retained a matching disposition while
        // silently bypassing an essential measured property.
        for (case_index, key) in [
            (0, "final_adjudication_backlog"),
            (1, "final_adjudication_backlog"),
            (2, "first_progress_tick"),
            (2, "restored_by_tick"),
            (3, "escalation_tick"),
            (4, "escalation_tick"),
            (6, "restored_by_tick"),
            (7, "final_adjudication_backlog"),
            (8, "first_progress_tick"),
        ] {
            let mut suite = original();
            suite["workload_cases"][case_index]["expected"]
                .as_object_mut()
                .expect("expected workload metrics")
                .remove(key);
            let errors = mutated(suite)
                .expect_err("missing semantic assertion must fail closed")
                .join(" ");
            assert!(errors.contains(key), "missing {key}: {errors}");
        }
    }

    #[test]
    fn changed_restoration_capacity_is_detected() {
        let mut changed = original();
        changed["workload_cases"][8]["restoration_capacity"][0] = json!(0);
        assert!(mutated(changed).is_err());
    }

    #[test]
    fn changed_alert_rate_is_detected() {
        let mut changed = original();
        changed["planning_cases"][0]["false_positive_num"] = json!(0);
        assert!(mutated(changed).is_err());
    }

    #[test]
    fn forged_decision_outcome_is_rejected() {
        let mut changed = original();
        changed["decision_cases"][0]["expected"] = json!("OK");
        assert!(mutated(changed).is_err());
    }

    #[test]
    fn missing_family_cannot_false_pass() {
        let mut changed = original();
        changed["decision_cases"] = json!([]);
        assert!(mutated(changed).is_err());
    }

    #[test]
    fn missing_duplicate_or_negative_capacity_cannot_pass() {
        let mut duplicate = original();
        duplicate["workload_cases"][0]["id"] = duplicate["decision_cases"][0]["id"].clone();
        assert!(mutated(duplicate).is_err());
        let mut negative = original();
        negative["workload_cases"][0]["service_capacity"][0] = json!(-1);
        assert!(mutated(negative).is_err());
    }

    #[test]
    fn missing_workload_input_or_zero_bound_is_rejected() {
        let mut missing = original();
        missing["workload_cases"][0]
            .as_object_mut()
            .unwrap()
            .remove("arrivals");
        assert!(serde_json::from_value::<Fixture>(missing).is_err());
        let mut zero = original();
        zero["workload_cases"][0]["progress_bound"] = json!(0);
        assert!(mutated(zero).is_err());
    }

    #[test]
    fn invalid_rate_and_negative_arrival_are_rejected() {
        let mut rate = original();
        rate["planning_cases"][0]["sensitivity_den"] = json!(0);
        assert!(mutated(rate).is_err());
        let mut arrival = original();
        arrival["workload_cases"][0]["arrivals"][0] = json!(-2);
        assert!(mutated(arrival).is_err());
    }
}
