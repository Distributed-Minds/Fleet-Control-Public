#!/usr/bin/env python3
"""Pure deterministic authority-closure behavior model for spec 2 fixtures.

This module intentionally computes dispositions from semantic inputs only. Fixture
expected fields are not accepted by the reducer and therefore cannot make a case
pass by declaration alone.
"""

from __future__ import annotations

from typing import Any


class UnsupportedAuthorityClosureCase(ValueError):
    """Raised when a synthetic case does not describe a supported semantic path."""


def _evaluate_composition(case: dict[str, Any]) -> dict[str, Any] | None:
    """Evaluate a declared multi-root composition rule when one is present."""
    composition = case.get("composition")
    if composition == "ALL_REQUIRED":
        surviving = case.get("surviving_roots", 0)
        required = case.get("required_roots", 0)
        return {"expected": "BOUNDED_AUTHORITY" if surviving >= required else "NO_AUTHORITY"}
    if composition == "ANY_OF_DECLARED":
        surviving = case.get("surviving_roots", 0)
        return {"expected": "BOUNDED_AUTHORITY" if surviving > 0 else "NO_AUTHORITY"}
    return None


def evaluate_authority_closure(case: dict[str, Any]) -> dict[str, Any]:
    """Compute the spec-2 disposition for one synthetic authority state."""

    # Compatibility, visibility, and completeness fail closed before mutation.
    if case.get("lineage_protocol_compatible") is False and case.get("mutation_requested"):
        return {"expected": "FAIL_CLOSED_UNTIL_COMPATIBLE"}
    if case.get("provider_access") == "ERROR":
        return {"expected_closure": "ERROR"}
    if case.get("inventory_complete") is False:
        return {"expected_closure": "UNKNOWN"}
    if case.get("failed_descendants", 0) > 0:
        return {"expected_closure": "PARTIAL"}
    if case.get("declared_surfaces_complete") and case.get("undeclared_external_surfaces_unknown"):
        return {"expected_closure": "COMPLETE_FOR_DECLARED_SURFACES"}

    # Reconciliation and exact-incarnation safety precede destructive retry.
    if case.get("cancel_ack_lost") and case.get("authoritative_state_known") is False:
        return {"expected": "READBACK_FIRST"}
    if case.get("locator_reused") and case.get("same_incarnation") is False:
        return {"expected": "DO_NOT_CANCEL_REPLACEMENT"}
    if case.get("same_operation_id") and case.get("same_incarnation"):
        return {"expected_logical_cancellations": 1}
    if case.get("external_deletion") and case.get("closure_caused_deletion") is False:
        return {"expected": "NO_CAUSAL_CLAIM"}

    # Historical/committed world state is not fresh authority.
    if case.get("state_class") in {"COMMITTED_OBLIGATION", "HISTORICAL_EFFECT"}:
        return {"expected": "RESIDUAL_STATE"}
    if case.get("cleanup_consequential") and case.get("recovery_authority") is False:
        return {"expected": "DENY"}

    # Handoff survives only through independent current authority and exact scope.
    if "handoff_independent" in case:
        if case.get("handoff_independent") and case.get("retained_scope_exact"):
            return {"expected": "PRESERVE_BOUNDED"}
        return {"expected": "REJECT"}

    # Cycles cannot manufacture authority. A surviving external root is only the
    # seed for the SCC; any declared multi-root composition still constrains the
    # effective capability of that cyclic component.
    if case.get("cycle"):
        if not case.get("external_root"):
            return {"expected": "NO_AUTHORITY"}
        composed = _evaluate_composition(case)
        if composed is not None:
            return composed
        return {"expected": "ROOT_BOUNDED_ONLY"}

    composed = _evaluate_composition(case)
    if composed is not None:
        return composed

    # Runtime/deferred authority remains derivative even after the initiator stops.
    if case.get("runtime_minted") and case.get("initiator_stopped"):
        return {"expected": "TRACK_DERIVATIVE"}
    if case.get("grandchild_created_after_fence"):
        return {"expected": "NO_AUTHORITY"}
    if case.get("job_started_after_cutoff") and case.get("current_authority") is False:
        return {"expected": "DENY_OR_UNRESOLVED"}
    if case.get("provider_credential_valid") and case.get("fleet_authority_revoked"):
        if case.get("external_invalidation_complete") is False:
            return {"expected": "DENY_AND_RECORD_REVOCATION_DEBT"}
        return {"expected": "DENY"}
    if case.get("ancestor_cutoff") and case.get("child_effect_after_cutoff"):
        return {"expected": "DENY"}

    # Shared topology does not imply shared authority provenance.
    if case.get("shared_infrastructure") and case.get("authority_dependency") is False:
        return {"expected": "PRESERVE"}

    raise UnsupportedAuthorityClosureCase(f"unsupported authority-closure state: {case!r}")
