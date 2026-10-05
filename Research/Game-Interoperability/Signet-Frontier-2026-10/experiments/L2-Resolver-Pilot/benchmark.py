#!/usr/bin/env python3
from __future__ import annotations
import json
import re
from pathlib import Path

NO_MATCH = "__NO_MATCH__"

def tokens(value: str) -> set[str]:
    return set(re.findall(r"[a-z0-9]+", value.lower().replace("_", " ").replace(":", " ")))

def lookup(case: dict, candidates: list[str]) -> str:
    return case["gold"] if case["gold"] in candidates else NO_MATCH

def lexical(case: dict, candidates: list[str]) -> str:
    query = tokens(case["archetype"]) | tokens(case["game"])
    ranked = []
    for index, candidate in enumerate(candidates):
        score = len(query & tokens(candidate))
        ranked.append((score, -index, candidate))
    ranked.sort(reverse=True)
    best = ranked[0]
    return best[2] if best[0] > 0 else NO_MATCH

def first(case: dict, candidates: list[str]) -> str:
    return candidates[0] if candidates else NO_MATCH

BACKENDS = {"lookup": lookup, "lexical": lexical, "first": first}

def rotate(values: list[str], count: int) -> list[str]:
    return values[count:] + values[:count]

def evaluate(name: str, resolver, cases: list[dict]) -> dict:
    base_ok = 0
    abstain_ok = 0
    stable = 0
    details = []

    for case in cases:
        candidates = list(case["candidates"])
        prediction = resolver(case, candidates)
        base_ok += prediction == case["gold"]

        variants = [rotate(candidates, index) for index in range(len(candidates))]
        permutation_predictions = [resolver(case, variant) for variant in variants]
        stable += len(set(permutation_predictions)) == 1

        without_gold = [candidate for candidate in candidates if candidate != case["gold"]]
        no_match_prediction = resolver(case, without_gold)
        abstain_ok += no_match_prediction == NO_MATCH

        details.append({
            "id": case["id"],
            "base_pred": prediction,
            "permutation_predictions": permutation_predictions,
            "no_match_pred": no_match_prediction,
        })

    count = len(cases)
    return {
        "backend": name,
        "base_accuracy": base_ok / count,
        "no_match_abstention_accuracy": abstain_ok / count,
        "candidate_order_stability": stable / count,
        "details": details,
    }

def main() -> None:
    data = json.loads(Path("cases.json").read_text(encoding="utf-8"))
    cases = data["cases"]
    output = {
        "schema": "signet-resolver-pilot-results@0",
        "case_count": len(cases),
        "results": [evaluate(name, resolver, cases) for name, resolver in BACKENDS.items()],
    }
    Path("results.json").write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")

    for result in output["results"]:
        print(
            f"{result['backend']}: "
            f"base_accuracy={result['base_accuracy']:.3f} "
            f"no_match={result['no_match_abstention_accuracy']:.3f} "
            f"order_stability={result['candidate_order_stability']:.3f}"
        )

if __name__ == "__main__":
    main()
