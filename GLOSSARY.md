# Glossary

## Agent
A persistent fleet identity such as `A1`. An agent is not permanently tied to one job. Its current behavior mode is reconstructed from durable state.

## AGENT_STATE
An append-only state record stored in the coordination issue. It tells a future run which behavior mode the same agent should enter next.

## Branch
A named line of Git history. Fleet implementation work normally happens on non-default branches.

## Build / BUILD
The lifecycle/mode where authorized implementation changes are made.

## Canonical issue
The GitHub issue treated as the current authoritative specification for a work package.

## Clone
A Git operation that copies a repository to a computer. A plain clone of this public repository currently checks out upstream `main`, which is only a landing page and does **not** contain the Phase0 starter. For a command-line installation, explicitly clone `--branch phase0/public-v0`, then copy its `Phase0/` folder into a repository you control, following [Getting Started](GETTING-STARTED.md#if-you-do-not-want-to-fork). Cloning the upstream repository alone does not install or enroll a fleet.

## Coordination issue
One append-only GitHub issue used for ownership transitions and persistent agent state.

## Default branch
The branch GitHub presents by default and which an installed Phase0 fleet's scheduled agents read. This is configured per repository, not universally named `main`. The public upstream currently uses `main` for a landing page and keeps the actual Phase0 preview on `phase0/public-v0`. An installation must have `Phase0/05-FLEET-CONFIG.md` on **its own repository's default branch** before enabling agents. The starter policy keeps final merges human-owned.

## Derived
A conclusion mechanically inferred from directly observed facts.

## Fleet
The set of persistent scheduled agent identities working under the same Phase0 rules.

## Fleet-Control
The underlying orchestration and coordination subsystem of FREE ENERGY. Its current public Phase0 preview is a Markdown-first starter, not the planned centrally dispatched volunteer-worker service.

## Fork
A GitHub feature that creates your own GitHub-hosted copy of another public repository. For this repository's current preview, copying **only the default branch** gives you the landing page without Phase0. Choose the all-branches fork route and set **your fork's** default branch to `phase0/public-v0`, or use the separately documented ZIP/command-line installation paths. See [Getting Started](GETTING-STARTED.md#2-install-the-actual-phase0-preview-in-a-repository-you-control).

## FREE ENERGY
The public collaboration ecosystem for repairing, porting, remastering, remixing, and eventually connecting games and reusable work. Those larger game-platform capabilities are a roadmap, not a shipped Phase0 feature.

## Gate
A readiness decision about whether a specific issue specification is ready for implementation.

## Git
The version-control system underneath GitHub.

## GitHub
A service that hosts Git repositories, issues, pull requests, checks, releases, and collaboration metadata.

## Human mission
The durable representation of what the human wants and how much mutation is authorized.

## Issue
A GitHub work/discussion item. Fleet-Control uses issues as durable specifications and mission records, not only bug reports.

## Lifecycle
The package progression: discover, research, predict, correct, specify, gate, build, verify, integrate, handoff.

## Mode
An agent's current search/quality responsibility: PLAN, PREDICT, AUDIT, BUILD, or INTEGRATE.

## Mutation
Any change to repository/GitHub state: files, branches, issues, PRs, comments, labels, etc.

## Observed
A fact directly inspected or executed with evidence.

## Phase0
The currently available Markdown-first, GitHub-backed Fleet-Control coordination preview. The installable source is on `phase0/public-v0`, with a separately published historical v0.1.2 starter ZIP; upstream `main` does not yet contain the installed preview. See [Getting Started](GETTING-STARTED.md) for verified installation choices and current archive caveats.

## Planned universal volunteer worker
A future contributor worker that receives specific authorized assignments from a trusted central control plane rather than choosing its own work. Required by [issue #70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70), but **not implemented** by Phase0's A1–A5 identities or today's temporary manual operators.

## Predicted
A plausible failure or conclusion not yet directly proved.

## Pull request (PR)
A GitHub proposal to combine one branch into another.

## Repository / repo
A version-controlled project and its history stored in Git/GitHub.

## Unknown
A fact for which current evidence is missing, stale, inaccessible, or ambiguous.
