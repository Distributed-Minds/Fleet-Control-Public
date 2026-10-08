#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = json.loads((ROOT / "fixtures/containment-capacity-spec2.json").read_text(encoding="utf-8"))


def fail(message):
    raise SystemExit(f"FAIL: {message}")


def decide(case):
    if not case.get("policy_compatible", True):
        return "REQUIRE_MIGRATION"
    ids = case.get("duplicate_case_ids", [])
    if len(ids) != len(set(ids)):
        return "RECONCILE_DUPLICATE"
    if case.get("renewal_attempt", False) and not case.get("renewal_evidence_current", False):
        return "REJECT_RENEWAL"
    if not case.get("authority_current", True) and case.get("effect_active", False):
        if not case.get("restoration_debt", False):
            return "INVALID_MISSING_DEBT"
    return "OK"


def capacities(case, key, ticks, default=0):
    value = case.get(key)
    if value is None:
        return [default] * ticks
    if isinstance(value, int):
        return [value] * ticks
    if len(value) != ticks:
        fail(f"{case['id']}: {key} length must match arrivals")
    return list(value)


def simulate(case):
    arrivals = list(case["arrivals"])
    ticks = len(arrivals)
    if ticks == 0:
        fail(f"{case['id']}: workload trace must contain at least one tick")

    backlog = int(case.get("initial_adjudication_backlog", 0))
    restoration = int(case.get("initial_restoration_work", 0))
    authority_current = bool(case.get("authority_current", False))
    effect_active = bool(case.get("effect_active", False))
    debt = bool(case.get("restoration_debt", False))
    bound = int(case.get("progress_bound", 0))
    if bound <= 0:
        fail(f"{case['id']}: progress_bound must be positive")
    if restoration > 0 and (authority_current or not effect_active or not debt):
        fail(f"{case['id']}: restoration work requires ended authority, active effect, and explicit debt")
    if not authority_current and effect_active and not debt:
        return {"disposition": "INVALID_MISSING_DEBT"}

    if not authority_current and not effect_active:
        shared = capacities(case, "service_capacity", ticks, 0)
        for i, incoming in enumerate(arrivals):
            backlog += incoming
            backlog = max(0, backlog - shared[i])
        return {"disposition": "RESTORED", "final_adjudication_backlog": backlog}

    mode = case.get("capacity_mode", "shared")
    scheduler = case.get("scheduler", "adjudication_first")
    first_progress = None
    restored_by = None
    since_progress = 0
    overload_seen = False

    if mode == "shared":
        service = capacities(case, "service_capacity", ticks, 0)
    elif mode == "separate":
        adjudication_service = capacities(case, "adjudication_capacity", ticks, 0)
        restoration_service = capacities(case, "restoration_capacity", ticks, 0)
    else:
        fail(f"{case['id']}: unsupported capacity_mode {mode}")

    for tick, incoming in enumerate(arrivals, start=1):
        backlog += int(incoming)
        if restoration > 0:
            since_progress += 1

        restored_units = 0
        if mode == "separate":
            cap_r = restoration_service[tick - 1]
            cap_a = adjudication_service[tick - 1]
            if restoration > 0 and cap_r > 0:
                restored_units = min(restoration, cap_r)
                restoration -= restored_units
            backlog = max(0, backlog - cap_a)
            if incoming > cap_a:
                overload_seen = True
        else:
            cap = service[tick - 1]
            remaining = cap
            if restoration > 0 and remaining > 0:
                due = since_progress >= bound
                if scheduler == "restoration_first" or (scheduler == "deadline_safe" and due):
                    restored_units = 1
                    restoration -= 1
                    remaining -= 1
            if remaining > 0:
                served = min(backlog, remaining)
                backlog -= served
                remaining -= served
            if restoration > 0 and remaining > 0 and scheduler == "restoration_first":
                extra = min(restoration, remaining)
                restoration -= extra
                restored_units += extra
                remaining -= extra
            if incoming > cap:
                overload_seen = True

        if restored_units:
            if first_progress is None:
                first_progress = tick
            since_progress = 0
            if restoration == 0:
                restored_by = tick
                effect_active = False
                debt = False

        if restoration > 0 and since_progress >= bound:
            if case.get("fail_safe_escalates", False):
                return {
                    "disposition": "DEGRADED_ESCALATION",
                    "escalation_tick": tick,
                    "first_progress_tick": first_progress,
                    "final_adjudication_backlog": backlog,
                }
            return {
                "disposition": "INVALID_STARVATION",
                "escalation_tick": tick,
                "first_progress_tick": first_progress,
                "final_adjudication_backlog": backlog,
            }

    if restored_by is not None:
        return {
            "disposition": "RESTORED",
            "first_progress_tick": first_progress,
            "restored_by_tick": restored_by,
            "final_adjudication_backlog": backlog,
        }
    if restoration > 0 and first_progress is not None:
        return {
            "disposition": "RESTORE_PROGRESS",
            "first_progress_tick": first_progress,
            "final_adjudication_backlog": backlog,
        }
    if authority_current:
        return {
            "disposition": "OVERLOAD_VISIBLE" if overload_seen or backlog > 0 else "OK",
            "final_adjudication_backlog": backlog,
        }
    if restoration == 0 and not effect_active:
        return {"disposition": "RESTORED", "final_adjudication_backlog": backlog}
    return {"disposition": "INVALID_STARVATION", "final_adjudication_backlog": backlog}


def alert_volume(case):
    events = int(case["event_volume"])
    true_events = int(case["true_events"])
    benign = events - true_events
    if not (0 <= true_events <= events):
        fail(f"{case['id']}: true_events out of range")
    sn, sd = int(case["sensitivity_num"]), int(case["sensitivity_den"])
    fn, fd = int(case["false_positive_num"]), int(case["false_positive_den"])
    if sd <= 0 or fd <= 0:
        fail(f"{case['id']}: rate denominators must be positive")
    if (true_events * sn) % sd or (benign * fn) % fd:
        fail(f"{case['id']}: fixture rates must yield integral deterministic counts")
    true_alerts = true_events * sn // sd
    false_alerts = benign * fn // fd
    return true_alerts + false_alerts, false_alerts


def assert_expected(case, actual):
    expected = case["expected"]
    if actual.get("disposition") != expected.get("disposition"):
        fail(f"{case['id']}: expected {expected['disposition']} got {actual.get('disposition')}")
    for key, value in expected.items():
        if key == "disposition":
            continue
        if actual.get(key) != value:
            fail(f"{case['id']}: expected {key}={value} got {actual.get(key)}")


def main():
    if DATA.get("spec_version") != 2:
        fail("fixture must identify spec version 2")

    for case in DATA["decision_cases"]:
        got = decide(case)
        if got != case["expected"]:
            fail(f"{case['id']}: expected {case['expected']} got {got}")

    for case in DATA["workload_cases"]:
        assert_expected(case, simulate(case))

    planning_results = {}
    for case in DATA["planning_cases"]:
        alerts, false_alerts = alert_volume(case)
        if alerts != case["expected_alerts"] or false_alerts != case["expected_false_alerts"]:
            fail(
                f"{case['id']}: expected alerts={case['expected_alerts']}/false={case['expected_false_alerts']} "
                f"got alerts={alerts}/false={false_alerts}"
            )
        planning_results[case["id"]] = (alerts, case["worker_count"], case["event_volume"])

    low = planning_results["low-action-volume"]
    high = planning_results["high-action-volume"]
    if low[1] != high[1] or high[2] <= low[2] or high[0] <= low[0]:
        fail("action-volume property: equal worker count must not hide higher event-driven workload")

    rare = next(c for c in DATA["planning_cases"] if c["id"] == "rare-event-false-positive-heavy")
    _, false_alerts = alert_volume(rare)
    true_alerts = rare["expected_alerts"] - false_alerts
    if false_alerts <= true_alerts:
        fail("rare-event property: fixture must actually be false-positive-heavy")

    total = len(DATA["decision_cases"]) + len(DATA["workload_cases"]) + len(DATA["planning_cases"])
    print(f"PASS {total}/{total} containment-capacity cases")


if __name__ == "__main__":
    main()
