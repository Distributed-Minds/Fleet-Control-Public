# FREE ENERGY

**Federated Remastering Ecosystem for Everyone — Engine for Networked Entertainment, Remixing, Games & You**

> **FREE ENERGY Remasters Everything.**

**Preview installation:** These files currently live on [`phase0/public-v0`](https://github.com/Distributed-Minds/Fleet-Control-Public/tree/phase0/public-v0), **not** upstream `main` (which contains a landing README but no Phase0 files). A default-branch-only fork will miss Phase0. See [the corrected installation steps](GETTING-STARTED.md#2-install-the-actual-phase0-preview-in-a-repository-you-control) or the [FREE ENERGY starter release](https://github.com/Distributed-Minds/Fleet-Control-Public/releases) before creating automations.

FREE ENERGY is the public collaborative project built on the Fleet-Control orchestration model.

The long-term goal is intentionally excessive: give people and AI-agent fleets a shared system for finding, repairing, remastering, porting, extending, remixing, and eventually combining games, engines, assets, tools, mods, and abandoned experiments instead of repeatedly rebuilding the same pieces in isolation.

This repository currently contains the first public bootstrap layer: **Fleet-Control Phase0**, a Markdown-first operating system for persistent GitHub-connected AI agents.

## FREE ENERGY vs. Fleet-Control

- **FREE ENERGY** is the public project, community, future website, and user-facing ecosystem.
- **Fleet-Control** is the orchestration substrate: the agent coordination, authority, lifecycle, state-machine, and repository-control machinery underneath it.
- **Phase0** is the current public bootstrap implementation of that machinery.

The current GitHub repository slug may still say `Fleet-Control-Public` during the migration. The canonical public name is **FREE ENERGY**.

See [`BRANDING.md`](BRANDING.md) for the naming contract.

## What Phase0 provides

Phase0 gives a fleet:

- a shared authority order;
- truthfulness/evidence rules;
- a work lifecycle;
- an implementation planning gate;
- collision-safe ownership using append-only GitHub comments;
- behavior modes such as PLAN, PREDICT, AUDIT, BUILD, and INTEGRATE;
- persistent per-agent state machines;
- a human-request / mission protocol;
- a small scheduler/automation contract.

The bootstrap is intentionally Markdown-only. It does not require a custom service, database, queue, or coordinator.

## Why this exists

A large collaborative game-remastering ecosystem cannot work if every human and every model starts from scratch in a private chat.

FREE ENERGY is intended to turn useful work into durable shared artifacts: reusable research, source ports, compatibility fixes, build knowledge, assets, tools, remasters, mods, experiments, and eventually higher-level mashups across many game projects.

The current Phase0 release is infrastructure for that larger system, not the final product.

## Core philosophy

The fleet should optimize for the strongest durable outcome, not for visible activity.

A useful run may resolve an unknown, correct a bad specification, predict a real failure mode and turn it into a fixture, implement a coherent package, repair a broken branch, verify a claim at an exact head, or explicitly block on missing evidence.

Repeated observations and unsupported confidence are not progress.

## Default safety boundary

By default, fleet agents:

- mutate only the repository containing this installation;
- work on non-default branches for implementation;
- do not merge into the default branch;
- preserve explicit human restrictions;
- do not silently turn analysis requests into code changes.

The installing human may deliberately change those policies.

## Start here

Choose a path based on what you actually want to do:

- **PLAY — find and play an existing project:** use that project's own release and installation instructions. FREE ENERGY's [playable-project catalog](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/60) is planned, not yet an available download directory.
- **HELP — improve an existing project:** follow [Help an existing FREE ENERGY project](HELP-A-PROJECT.md) for a beginner-friendly contribution path and a copyable prompt for a local coding agent. Check the upstream project's contribution and AI-use policies before doing or publishing work. The guide does not enroll you in a fleet or assign issues automatically.
- **MAKE — create a mod, port, remaster, or bridge:** consult the [FREE ENERGY workflow guide map](WORKFLOW-GUIDES.md) for technical routes, independently maintained examples, and rights/testing considerations. These references are not a hosted building service.
- **RUN YOUR OWN FLEET — install the Phase0 preview:** follow [Getting Started](GETTING-STARTED.md) and the [starter releases](https://github.com/Distributed-Minds/Fleet-Control-Public/releases) to install the separate GitHub agent-coordination starter in a repository you control. The public `main` branch does not yet contain this preview.

For the normative Phase0 machine read order, see [`Phase0/README.md`](Phase0/README.md).

## Contact and contributing

For general questions or to express contributor interest, see [public GitHub Discussions](https://github.com/Distributed-Minds/Fleet-Control-Public/discussions). For specific reproducible work, use the [issue tracker](https://github.com/Distributed-Minds/Fleet-Control-Public/issues). These are public contact routes, **not** automatic worker enrollment, task assignment, or repository access.

Do not post credentials, private repository material, sensitive personal information, or security-vulnerability details in public issues or Discussions. A confidential reporting route has **not** been verified; [issue #38](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/38) tracks that separately.

## License

FREE ENERGY's current Fleet-Control Phase0 bootstrap is licensed under the **MIT License**. See [`LICENSE`](LICENSE).

This branch family is the initial public Phase0 preview. It is intentionally kept off `main` until the maintainer chooses to integrate it.
