# Phase0 Human Requests and Missions

Human requests are converted into durable **missions** when shared fleet coordination is useful.

A mission says what the fleet is trying to accomplish and how much mutation is authorized. A behavior mode says how one agent contributes. Do not confuse them.

## EXPLORE

Typical language: think about, brainstorm, compare, explain possible approaches, critique an idea.

Default mutation budget: `READ_ONLY`.

The fleet may persist a concise research artifact or mission issue when durable coordination materially helps, but must not mutate product behavior unless the human expands scope.

## RESEARCH

Typical language: research, investigate, audit, analyze, reproduce, find the cause.

Default mutation budget: `ARTIFACTS`.

Allowed by default: reading, external research, issue/specification correction, durable findings, and diagnostic fixtures on non-default branches when they are the clearest way to prove/falsify a claim and do not silently become product behavior.

Product implementation is not implied.

## CHANGE

Typical language: fix, repair, update, refactor, migrate, resolve.

Default mutation budget: `NONDEFAULT_CODE`.

The fleet may use the full lifecycle on requested scope.

## CREATE

Typical language: make, build, implement, create an app/system/feature.

Default mutation budget: `NONDEFAULT_CODE`.

For large goals, decompose into architecture and coherent work packages. Never pretend a broad product is complete because one shallow slice exists.

## OPERATE

Typical language: maintain, clean up, triage, keep healthy, resolve failures, reduce backlog.

Default mutation budget: `NONDEFAULT_CODE` when changes are required for the requested operational outcome. Preserve unrelated product semantics.

## Explicit human restrictions override inference

Examples:

- “Research this, no code changes” => RESEARCH, product mutation forbidden.
- “Think about this and write conclusions into issues” => EXPLORE/RESEARCH, issue mutation allowed, product mutation forbidden.
- “Fix this but don't touch main” => CHANGE, non-default branches/PRs allowed, default branch forbidden.
- “Make Facebook” => CREATE; research/decompose/specify/build/verify rather than collapsing the request into one run.

When intent is genuinely ambiguous, use the least-mutating interpretation that still makes useful progress, record the ambiguity, and avoid irreversible product changes.

## Mission issue

A substantial shared request should have one canonical mission issue.

Suggested title:

```text
[fleet-mission] <short human outcome>
```

Required marker block:

```text
FLEET_MISSION
id=<stable-short-id>
status=ACTIVE|PAUSED|DONE|CANCELLED
intent=EXPLORE|RESEARCH|CHANGE|CREATE|OPERATE
mutation=READ_ONLY|ARTIFACTS|NONDEFAULT_CODE|CUSTOM
priority=<integer-or-human-order>
```

## Mission body

Preserve:

### Human request
A faithful concise statement of desired outcome.

### Explicit restrictions
Anything the human forbade or constrained.

### Current interpretation
Intent class and mutation budget, with ambiguity called out.

### Success
Observable mission-level outcomes when known.

### Child work packages
Links to canonical issues created for substantial implementation or research packages.

### Mission evidence / status
Compact current durable facts. Do not use this as the package-level design document.

## Mission selection

Agents inspect ACTIVE missions before inventing unrelated work.

Priority order:

1. explicit newest human priority;
2. blockers threatening repository integrity or active delivery;
3. dependency-unblocking work;
4. highest-value runnable mission/package;
5. useful read-only research.

Do not broaden missions merely to keep an agent busy.

## Mission completion

A mission is DONE only when the requested outcome is actually satisfied at the evidence level appropriate to the request.

For CREATE/CHANGE, an unmerged PR may be a valid fleet handoff but is not automatically equivalent to “deployed”, “released”, or “live”.

State exactly what is finished and what remains human-owned.
