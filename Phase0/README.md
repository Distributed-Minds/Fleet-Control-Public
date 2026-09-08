# Phase0 — public fleet operating system

Phase0 is the repository-local source of truth for the installed agent fleet.

The scheduler should carry very little policy. It supplies a persistent agent identity. The agent reconstructs behavior from these files plus durable GitHub coordination records.

## Normative read order

At the start of every persistent-agent run, read in this order:

1. `00-CONSTITUTION.md`
2. `05-FLEET-CONFIG.md`
3. `10-LIFECYCLE.md`
4. `20-PLANNING-GATE.md`
5. `30-COORDINATION.md`
6. `40-MODES.md`
7. `50-AGENT-STATE-MACHINES.md`
8. `60-HUMAN-REQUESTS.md`
9. `80-INTEGRATION-CANDIDATES.md` when the run predicts, validates, or executes Git integration candidates
10. `90-CONTAINMENT.md` when the run predicts, validates, or executes automated containment or false-positive recovery
11. `100-AD-HOC-RESEARCH.md` when a temporary researcher discovers non-default evidence, publishes an additive research packet, or hands evidence to persistent agents
12. reconstruct this agent's latest valid `AGENT_STATE`
13. inspect active mission(s), issues, PRs, branches, checks, and relevant repository evidence
14. execute the behavior mode selected by `50-AGENT-STATE-MACHINES.md`, bounded by human intent and ownership rules
15. release/handoff ownership and append the deterministic next `AGENT_STATE`

When normative files disagree, authority rules in `00-CONSTITUTION.md` apply.

## Important separation of concepts

These are deliberately separate:

- **human mission** — what outcome is wanted and what mutation is authorized;
- **agent mode** — how an agent searches for useful work this run;
- **package ownership** — which mutation scope a run currently controls;
- **lifecycle state** — where a specific work package is in delivery;
- **evidence state** — what is observed, derived, predicted, or unknown.

Changing one does not silently change the others.

## Default lifecycle

`DISCOVER → RESEARCH → PREDICT → CORRECT → SPECIFY → GATE → BUILD → VERIFY → INTEGRATE → HANDOFF`

Not every mission reaches BUILD. Analysis-only requests should normally stop upstream.

## Bootstrap bias

Phase0 is conservative about unsupported claims and conflicting mutation, but aggressive about making authorized deterministic progress.

Do not create work merely to keep every agent busy.
