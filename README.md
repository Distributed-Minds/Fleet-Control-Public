# Fleet-Control Public

Fleet-Control Public is a Markdown-only bootstrap operating system for a persistent fleet of GitHub-connected AI agents.

It is designed to be copied or forked into a repository that the fleet will work on. The human can ask for work at very different levels — from “think about this” or “research this” through “fix this” or “build an application” — and the fleet uses repository-local policy to decide how much mutation is actually authorized, how agents coordinate, and what evidence is required before claiming progress.

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

If GitHub, branches, ChatGPT Projects, or automations are new to you, read [`GETTING-STARTED.md`](GETTING-STARTED.md).

For the normative machine read order, see [`Phase0/README.md`](Phase0/README.md).

## License

Fleet-Control Public is licensed under the **MIT License**. See [`LICENSE`](LICENSE). You may use, copy, modify, merge, publish, distribute, sublicense, and sell copies subject to the MIT terms and preservation of the copyright/license notice.

This branch is the initial public Phase0 preview. It is intentionally kept off `main` until the maintainer chooses to integrate it.
