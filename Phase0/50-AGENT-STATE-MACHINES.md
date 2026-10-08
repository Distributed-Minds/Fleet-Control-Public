# Phase0 Persistent Agent State Machines

Scheduled agents are persistent identities, not permanently fixed roles.

The scheduler supplies `AGENT_ID`. This file determines the mode from durable `AGENT_STATE` history plus fleet size.

`40-MODES.md` defines what modes do. This file defines deterministic transitions.

## State record

At the end of every run, append exactly one state record for this agent:

```text
AGENT_STATE | agent=<id> | mode=<PLAN|PREDICT|AUDIT|BUILD|INTEGRATE> | phase=<SOLO|ANALYTIC|BOOTSTRAP|FREE> | noop_streak=<n> | last_result=<MATERIAL|NOOP|BOOTSTRAP> | last_build=<PROGRESS|NO-PROGRESS|none> | ts=<ISO8601> | note=<short>
```

The latest valid record for an agent is authoritative. Agents append only their own state. Package ownership is separate from agent state.

## Result classification

### MATERIAL

The run added a concrete durable fact or authorized change not already represented on the same unchanged state.

Examples include materially improving a mission/issue specification; adding a supported failure prediction, invariant, or fixture; correcting a real contradiction; discovering and recording a new concrete blocker; creating non-duplicate required scope; changing implementation/tests/tooling; repairing deterministic failure; or materially integrating/verifying a package.

### NOOP

The agent performed the search appropriate to its mode and produced no new durable fact or authorized change.

Repeated observation, duplicate finding, collision loss by itself, or rerunning unchanged checks without a new conclusion is NOOP.

### Two-NOOP exhaustion

Where a transition says `2 NOOP`, two consecutive NOOP runs in the same mode trigger it.

MATERIAL resets `noop_streak=0`. Mode changes reset `noop_streak=0`.

## Determine fleet topology

Read `FLEET_SIZE` and ordered `AGENTS` from `05-FLEET-CONFIG.md`.

An agent's index is its 1-based position in `AGENTS`.

# Fleet size = 1: SOLO delivery cycle

Bootstrap:

```text
mode=PLAN
phase=SOLO
noop_streak=0
```

The single agent must be able to deliver work; it is never trapped permanently in planning.

Mission-aware progression:

```text
PLAN
  material planning still needed -> PLAN
  analysis-only mission satisfied -> PLAN
  current package sufficiently specified -> PREDICT

PREDICT
  design changed by findings -> PLAN
  analysis-only mission satisfied -> PLAN
  current package sufficiently attacked and mission permits implementation -> BUILD

BUILD
  implementation PROGRESS -> INTEGRATE
  implementation NO-PROGRESS -> AUDIT
  no BUILD authorized by mission -> PLAN

AUDIT
  -> PLAN if semantics/specification changed
  -> BUILD if current gated implementation remains valid and mutation is authorized

INTEGRATE
  unfinished deterministic implementation work -> BUILD
  package handed off / no runnable package -> PLAN
```

The single agent records PLAN and ADVERSARIAL gate passes as separate passes.

# Fleet size = 2

## Agent index 1 — analytic continuity

Bootstrap:

```text
mode=PLAN
phase=ANALYTIC
noop_streak=0
```

Cycle on two-NOOP exhaustion:

```text
PLAN -> PREDICT -> AUDIT -> PLAN
```

MATERIAL keeps the agent in its current mode.

## Agent index 2 — bootstrap analyst then reactive builder

Bootstrap:

```text
mode=PLAN
phase=BOOTSTRAP
noop_streak=0
```

Two-NOOP exhaustion transitions:

```text
PLAN -> PREDICT -> AUDIT -> BUILD(FREE)
```

Once FREE:

```text
BUILD --PROGRESS-----> INTEGRATE
BUILD --NO-PROGRESS--> AUDIT
AUDIT ----------------> BUILD
INTEGRATE ------------> BUILD
```

If active missions do not authorize BUILD, BUILD mode performs read-only discovery/gate inspection and must not violate the mission merely because its mode is BUILD.

# Fleet size >= 3

## Agent index 1 — permanent planner

Bootstrap:

```text
mode=PLAN
phase=ANALYTIC
```

Remain PLAN. NOOP does not change mode.

## Agent index 2 — adversarial/analytic oscillator

Bootstrap:

```text
mode=PREDICT
phase=ANALYTIC
noop_streak=0
```

On two-NOOP exhaustion:

```text
PREDICT -> AUDIT -> PLAN -> PREDICT
```

MATERIAL keeps the current mode.

## Agent index 3..N — bootstrap analysts then reactive builders

Bootstrap:

```text
mode=PLAN
phase=BOOTSTRAP
noop_streak=0
```

On two-NOOP exhaustion:

```text
PLAN -> PREDICT -> AUDIT -> BUILD(FREE)
```

FREE phase:

```text
BUILD --PROGRESS-----> INTEGRATE
BUILD --NO-PROGRESS--> AUDIT
AUDIT ----------------> BUILD
INTEGRATE ------------> BUILD
```

This scales implementation capacity without removing continuous planning and adversarial coverage.

## End-of-run state procedure

Before ending:

1. reconstruct the starting agent state;
2. classify the run result truthfully;
3. calculate the deterministic next state;
4. finish/release package ownership;
5. append exactly one new AGENT_STATE record for this agent;
6. never rely on an unrecorded future transition.

If state history is malformed or ambiguous, record the ambiguity as a blocker and choose the safest bootstrap state rather than inventing history.
