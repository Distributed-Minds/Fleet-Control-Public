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
A Git operation that copies a repository to a computer.

## Coordination issue
One append-only GitHub issue used for ownership transitions and persistent agent state.

## Default branch
The repository's main line, commonly named `main`. The starter policy keeps final merges human-owned.

## Derived
A conclusion mechanically inferred from directly observed facts.

## Fleet
The set of persistent scheduled agent identities working under the same Phase0 rules.

## Fork
A GitHub feature that creates your own GitHub-hosted copy of another public repository.

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

## Predicted
A plausible failure or conclusion not yet directly proved.

## Pull request (PR)
A GitHub proposal to combine one branch into another.

## Repository / repo
A version-controlled project and its history stored in Git/GitHub.

## Unknown
A fact for which current evidence is missing, stale, inaccessible, or ambiguous.
