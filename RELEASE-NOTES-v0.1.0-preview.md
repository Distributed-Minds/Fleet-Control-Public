# Historical setup notice

> **Superseded for current installation guidance.** This file preserves the v0.1.0 preview as historical evidence. For new/current installations, use `RELEASE-NOTES-v0.1.1-preview.md`, `GETTING-STARTED.md`, and `TROUBLESHOOTING.md`.
>
> Product-surface and GitHub-capability wording below describes the v0.1.0-era snapshot. It is not a universal current statement about interactive or scheduled execution. Current mutation-capable setup requires context-specific capability, lineage, authority, and repository-policy evidence under `Phase0/110-GITHUB-SETUP-CAPABILITY.md`.
>
> The repository is now MIT-licensed. The v0.1.0 statement below that no public license had been selected is historical only.

# Fleet-Control Public — Phase0 Preview v0.1.0

This is the first public preview of the Markdown-only Fleet-Control coordination/state-machine foundation.

**If you have never used GitHub, Git, a ChatGPT Project, or an automation before, that is okay. Start with the ZIP attached to this release and open `GETTING-STARTED.md`. The instructions assume no previous knowledge.**

This release is deliberately marked **pre-release**. The core operating model is usable for experimentation, but names, formats, installation details, and policy may still change as the public design is tested by people outside the original repository.

---

## What is Fleet-Control Public?

Fleet-Control Public is not a new AI model and it is not a background server you install.

It is a set of Markdown (`.md`) files that act as a shared operating system for several persistent AI-agent runs working through the same GitHub repository.

The basic problem it tries to solve is simple:

> If five AI agents wake up separately, how do they know what the human actually wants, what the other agents are doing, what has already been proved, whether they are allowed to change something, and what they should do next?

Phase0 answers that using repository-visible rules and append-only GitHub state rather than one giant hidden prompt.

The fleet can receive requests at very different levels, for example:

- “Think about this design.”
- “Research why this happens. Do not change code.”
- “Find bugs in this repository.”
- “Fix issue #42.”
- “Build a small web application.”
- “Make a Facebook-like prototype.”

The human request is classified into a durable **mission**. Persistent agents then use their current **mode**, the shared work lifecycle, planning gates, evidence rules, and mutation ownership protocol to decide what useful contribution to make.

A request to think or research does **not** automatically authorize product-code changes. A request to make/fix/build can flow all the way through specification, implementation, verification, and handoff.

---

# The easiest way to understand the pieces

You will use three things.

## 1. GitHub repository

A **repository** (usually shortened to **repo**) is a project folder stored on GitHub with a complete change history.

Your own project source code lives there. Fleet-Control adds a `Phase0/` folder containing the shared fleet rules.

GitHub also provides:

- **issues** — durable work/specification records;
- **branches** — separate lines of changes;
- **pull requests (PRs)** — proposals to combine branch changes;
- **comments** — used by Phase0 for append-only coordination/state records;
- **checks / CI** — automated evidence such as tests.

The GitHub repository is the durable shared coordination surface. It is what lets one run stop and a later run reconstruct the state.

## 2. ChatGPT Project

A ChatGPT **Project** is a workspace that keeps related chats, files, and Project-specific instructions together.

In this starter setup, the Project is the human-facing control room. You talk naturally inside the Project. Its supplied Project instructions tell ChatGPT how to translate substantial fleet requests into durable GitHub missions without pretending work has already happened.

## 3. Persistent scheduled agents

The starter pack contains prompts for five persistent identities:

```text
A1
A2
A3
A4
A5
```

They are identities, not permanent job titles.

Each time an automation runs, it reads the same `Phase0/` rules from GitHub, finds its latest `AGENT_STATE`, reconstructs its current behavior mode, inspects current repository evidence, does useful authorized work, releases ownership, and records the next state.

That means A4 today may be planning and later become a builder. The behavior comes from durable state, not from a giant role prompt copied into the scheduler.

---

# What to download

On this GitHub Release page, look under **Assets** and download:

```text
Fleet-Control-Public-Phase0-Starter-v0.1.0-preview.zip
```

That is the installer/starter pack intended for humans.

GitHub will also automatically show “Source code (zip)” and “Source code (tar.gz)”. Those are automatic snapshots of the repository. They are useful for developers, but the named **Starter** ZIP is the easiest installation package because it rearranges the files by what you need to do with them.

---

# What is inside the Starter ZIP?

After extracting/unzipping it, you will see:

```text
GETTING-STARTED.md
GLOSSARY.md
TROUBLESHOOTING.md
INSTALL-CHECKLIST.md
README-FIRST.md

Phase0/
  README.md
  00-CONSTITUTION.md
  05-FLEET-CONFIG.md
  10-LIFECYCLE.md
  20-PLANNING-GATE.md
  30-COORDINATION.md
  40-MODES.md
  50-AGENT-STATE-MACHINES.md
  60-HUMAN-REQUESTS.md
  70-AUTOMATION-CONTRACT.md
  templates/
    ...

COPY-INTO-CHATGPT/
  PROJECT-INSTRUCTIONS.md

COPY-INTO-AUTOMATIONS/
  A1.md
  A2.md
  A3.md
  A4.md
  A5.md
```

The folder names are literal instructions:

- `Phase0/` goes into the GitHub repository the fleet should govern.
- `COPY-INTO-CHATGPT/PROJECT-INSTRUCTIONS.md` is copied into ChatGPT Project instructions.
- `COPY-INTO-AUTOMATIONS/A1.md` through `A5.md` are copied into separate persistent automations/tasks.

---

# Installation path A — recommended for someone new to GitHub

This path does not require the command line.

## Step A1 — make a GitHub account

Go to:

https://github.com/

Create an account and sign in.

GitHub is the website that will store your repository, branches, issues, and PRs. Git is the underlying version-control technology; you do not need to learn Git commands for this installation path.

## Step A2 — fork Fleet-Control Public

Open:

https://github.com/Distributed-Minds/Fleet-Control-Public

Find the **Fork** button near the upper-right area of the repository page.

Choose your own GitHub account (or an organization you control) as the destination and create the fork.

A **fork** means GitHub creates a copy of this public repository under your account.

For example, the original is:

```text
Distributed-Minds/Fleet-Control-Public
```

Your fork might become:

```text
alice/Fleet-Control-Public
```

Your agents must work on **your repository**, not on the original `Distributed-Minds` repository.

## Step A3 — write down the exact repository name

At the top of a GitHub repository page you will see an owner and repository name.

The value you need has this form:

```text
OWNER/REPOSITORY
```

Example:

```text
alice/Fleet-Control-Public
```

Do not include `https://github.com/` when a prompt asks for `<OWNER>/<REPOSITORY>`.

Keep this value nearby. You will paste it several times.

---

# Installation path B — add Fleet-Control to an existing project

Use this if you already have a GitHub repository containing the project you want the agents to work on.

Do **not** replace that repository with Fleet-Control Public.

Instead:

1. download the Starter ZIP;
2. extract it on your computer;
3. copy the entire supplied `Phase0/` folder into the **root** (top level) of your existing repository;
4. commit/push those Markdown files to GitHub using your normal Git/GitHub workflow;
5. use the existing repository's exact `OWNER/REPOSITORY` value in every Fleet-Control prompt.

Your repository may then look something like:

```text
my-project/
  src/
  tests/
  package.json
  README.md
  Phase0/
    00-CONSTITUTION.md
    ...
```

Fleet-Control is intended to sit next to your actual project rather than becoming a separate database or service.

---

# What does “clone” mean, and do I need it?

People often say “clone this into my GitHub”, but Git/GitHub use two different words:

- **Fork** = GitHub creates your own GitHub-hosted copy of another public repository.
- **Clone** = Git copies a repository from GitHub onto your computer.

A beginner can use the fork path above and does not need to run `git clone` at all.

If you already know Git, cloning and pushing to a new remote is also fine.

---

# Connect GitHub to ChatGPT / Codex

The fleet needs authorized access to the repository it will inspect or modify.

## ChatGPT repository access

In ChatGPT, open **Settings → Apps** (some accounts/product surfaces may say **Plugins**) and locate GitHub.

Connect your GitHub account and authorize the exact repository you want the fleet to use.

Two separate things must both be true:

1. the GitHub app/connection is installed for the correct GitHub account or organization;
2. that installation is actually allowed to access your repository.

If the repository belongs to a GitHub organization, an organization owner/admin may have to approve the app or the repository.

A brand-new repository may take a few minutes to become visible after authorization.

## Reading versus writing

The ordinary GitHub app in ChatGPT may be read-only depending on the ChatGPT product surface. OpenAI's Codex product is intended for code generation/editing and pushing changes to GitHub.

If an agent can inspect a repository but gets permission errors when it tries to push, create a branch, or open/change repository artifacts, verify the Codex/GitHub authorization and repository permissions rather than weakening Phase0's truthfulness rules.

---

# Create the ChatGPT Project

## Step P1 — create a Project

Open ChatGPT while signed in.

Choose **New project** from the sidebar.

Give it a name such as:

```text
My Agent Fleet
```

The name has no machine meaning; choose something recognizable.

## Step P2 — install the Project instructions

In the Project, open the `...` menu and choose **Project settings**.

Open this file from the Starter ZIP:

```text
COPY-INTO-CHATGPT/PROJECT-INSTRUCTIONS.md
```

Copy the complete text into the Project instructions field.

Then find:

```text
<OWNER>/<REPOSITORY>
```

and replace it with your exact repository.

Example:

Before:

```text
Target repository: `<OWNER>/<REPOSITORY>`.
```

After:

```text
Target repository: `alice/my-project`.
```

Do not point Project instructions back at `Distributed-Minds/Fleet-Control-Public` unless you are a maintainer deliberately working on this public template itself.

---

# Configure how many persistent agents you want

Open in your own repository:

```text
Phase0/05-FLEET-CONFIG.md
```

The default starter is:

```text
FLEET_SIZE=5
AGENTS=A1,A2,A3,A4,A5
```

You do not have to run five.

## One agent

Use:

```text
FLEET_SIZE=1
AGENTS=A1
```

The state machine changes into a solo delivery cycle so the one agent can plan, challenge, build, and integrate work itself.

## Two agents

Use:

```text
FLEET_SIZE=2
AGENTS=A1,A2
```

One identity maintains analytical continuity while the second can mature into a reactive builder.

## Three agents

Use:

```text
FLEET_SIZE=3
AGENTS=A1,A2,A3
```

With three or more agents, Phase0 preserves planning/adversarial capacity while additional identities can become builders.

## Five agents

Leave the defaults and create A1 through A5.

The exact number is less important than this rule:

> Every live automation needs a unique persistent identity, and `FLEET_SIZE`/`AGENTS` must describe the automations you actually run.

Do not run five separate tasks all claiming to be A1.

---

# Create the recurring agent tasks / automations

Open:

```text
COPY-INTO-AUTOMATIONS/
```

There is one prompt per identity.

For A1:

```text
A1.md
```

For A2:

```text
A2.md
```

and so on.

For each identity you enable:

1. open the file;
2. replace `<OWNER>/<REPOSITORY>` with your exact repository;
3. create a separate recurring automation/task;
4. paste that agent's complete prompt into it;
5. choose a schedule appropriate to your account and workload.

In ChatGPT, supported recurring tasks are managed from **Scheduled**. Scheduled-task capabilities and frequency limits depend on your account/plan and current product surface. Codex automations are a separate product workflow.

The starter pack does not prescribe one universal schedule. Hourly may be useful for an actively developing repository; a lower frequency may be more appropriate for a small or low-priority project.

## Why the prompt is so small

A traditional setup might paste five enormous prompts into five schedulers. That creates five copies that can drift apart.

Fleet-Control deliberately makes the scheduled prompt say, in effect:

> I am A3. Read the current operating system from GitHub. Reconstruct A3's state. Inspect current missions/repository evidence. Do the mode the state machine says. Record the next state.

Most policy therefore lives in one visible, version-controlled place.

---

# Important note about ChatGPT Project files and scheduled tasks

Do not depend on a scheduled task being able to read arbitrary files uploaded directly to the ChatGPT Project.

The durable Phase0 machine rules belong in GitHub. Every persistent agent prompt tells the run to fetch/re-read those repository files.

The ChatGPT Project is primarily the human-facing mission/control surface; GitHub is the durable shared state surface.

---

# Your first safe test

Do not begin by asking the fleet to redesign production infrastructure.

Use a disposable repository or a project where mistakes are easy to recover.

A good first request is:

```text
Think about this repository. Identify the three largest architectural uncertainties. Do not change product code.
```

The expected behavior is analytical. The fleet should not silently turn that into a large implementation branch.

Then try something mutation-authorized:

```text
Create a small README improvement through the normal Phase0 lifecycle. Do not merge into main.
```

Inspect the resulting GitHub issues/branches/PRs yourself.

Only increase authority after you understand what the fleet is doing.

---

# What the five behavior modes mean

The agents are not five fixed personalities, but they can enter these modes:

## PLAN

Research architecture, dependencies, specifications, acceptance criteria, unknowns, and implementation seams.

## PREDICT

Attack the design before code is written. Ask what happens under crashes, concurrent writers, retries, stale state, permission failures, malformed input, and version skew.

## AUDIT

Look for contradictions between documents, issues, PRs, implementation evidence, and protocol rules. Correct canonical sources rather than repeating the same warning.

## BUILD

Implement a sufficiently specified and authorized package on a non-default branch, with deterministic evidence.

## INTEGRATE

Inspect whole packages/PRs, repair deterministic failures, reduce branch fragmentation, verify exact heads, and prepare clean human handoffs.

Agent identity and mode are different. A4 is not “the builder”; A4 is an identity whose durable state may eventually select BUILD.

---

# How truthfulness works

Phase0 uses four evidence words intentionally:

## OBSERVED

The agent directly inspected or executed the fact.

Example: it fetched branch head `abc123`, ran a test, and observed exit code 0.

## DERIVED

A conclusion follows mechanically from observed facts.

## PREDICTED

A plausible failure/risk has been reasoned about but is not yet directly proved.

## UNKNOWN

Current evidence is missing, stale, inaccessible, or ambiguous.

The point is not to make agents write these labels in every sentence. The point is to prevent “this probably works” from silently becoming “this is verified”.

---

# How collision avoidance works

Before a run changes a shared surface, it claims a narrow mutation scope through the coordination issue.

The simplified lifecycle is:

```text
INTENT → election → OWNED → WORKING → HANDOFF/RELEASE
```

If another run already owns overlapping work, the new run yields and finds something independent instead of both pushing competing edits.

Exact Git branch heads are treated as compare-and-swap evidence. Unexpected remote movement is a reason to stop and reconcile, not a reason to force-push through someone else.

---

# Why implementation has a planning gate

For substantive implementation, a canonical issue carries a `Phase0 spec version` and should include enough information that a builder does not have to invent product semantics halfway through coding.

For multi-agent fleets, normal BUILD expects both:

- a PLAN readiness pass; and
- an independent ADVERSARIAL readiness pass.

For a one-agent fleet, the same identity may perform both, but as separate passes that re-read current state.

This is intentionally slower than instantly writing code from every vague sentence. The design goal is to spend fleet capacity preventing bad implementation before it exists.

---

# What does HUMAN_MERGE_ONLY mean?

The starter configuration says:

```text
DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY
```

That means fleet agents may create/repair non-default branches and pull requests, but the final decision to merge into the repository's default branch remains human-owned.

This gives a beginner a visible inspection point before fleet work becomes the main project history.

Do not enable autonomous merging just because the agents appear productive for a few runs. Merge authority is a separate governance decision.

---

# What happens when I say “think”, “research”, “fix”, or “make”?

The public Phase0 human-intent router defines five mission classes.

## EXPLORE

Examples: think, brainstorm, critique, compare.

Default: read-only product behavior.

## RESEARCH

Examples: investigate, analyze, audit, reproduce.

Default: research artifacts/spec corrections are allowed, but product implementation is not automatically implied.

## CHANGE

Examples: fix, repair, update, refactor, migrate.

Default: non-default-branch implementation is authorized within the requested scope.

## CREATE

Examples: make, build, implement a new app/system/feature.

Default: full lifecycle is authorized on non-default branches, including decomposition of very broad goals.

## OPERATE

Examples: maintain, triage, keep healthy, resolve failures.

Default: make the changes needed for the requested operational outcome without silently changing unrelated product semantics.

Explicit human restrictions always win. “Research this, but do not change code” is still research-only even if a BUILD-mode agent wakes up.

---

# What can I ask the fleet to build?

The operating system is domain-agnostic. It is not restricted to Fleet-Control itself.

The repository owner can ask their fleet to work on arbitrary goals that fit their tools and permissions: research, planning, documentation, bug fixing, application development, repository maintenance, protocol design, and more.

Very large requests should be decomposed. For example, “make Facebook” is not permission to create one shallow page and declare success. A useful fleet should identify architecture, unknowns, data/security concerns, work packages, acceptance criteria, and staged implementation.

---

# Security and secrets

Do not place secrets in:

- `Phase0/` Markdown files;
- public GitHub repositories;
- issue bodies/comments;
- pull-request descriptions;
- ChatGPT Project instructions;
- automation prompts;
- release notes.

Secrets include API keys, passwords, private access tokens, recovery codes, signing keys, database credentials, and similar material.

Fleet-Control coordinates repository work. It does not replace GitHub permission controls, secret stores, CI protections, backups, dependency/security scanning, or application-specific authorization.

---

# Troubleshooting

The ZIP includes `TROUBLESHOOTING.md`. Common causes are:

## Repository missing

The GitHub app is connected to the wrong account/organization, the specific repository was not authorized, admin approval is pending, or the new repository has not appeared yet.

## Read works, write fails

Check whether you are using a read-only ChatGPT GitHub surface versus a Codex/write-capable workflow, and check repository permissions.

## Agents duplicate work

Verify they all read the same Phase0 folder and coordination issue, and that each has a unique identity.

## Fleet plans forever

Check mission intent, planning gates, current agent modes, dependencies, and write permissions before assuming the state machine is stuck.

## Fleet codes after “research only”

The mission was classified incorrectly or an agent violated the human mutation boundary. Correct the mission and inspect the automation's Phase0 compliance.

## Agent says it pushed something but nothing exists

Treat it as unverified. Ask for exact repository, branch/PR/issue, and commit head. Phase0 deliberately says a successful write must be observed before being claimed.

---

# Files for maintainers/developers

The source branch for this preview is:

```text
phase0/public-v0
```

The release is generated from that branch without merging it into `main`.

The canonical operating files are under `Phase0/`. The release packaging workflow creates beginner-ready A1–A5 prompts and a starter ZIP from those canonical templates.

---

# Preview limitations

This is Phase0, not the final structured Fleet-Control engine.

Current coordination is intentionally Markdown/GitHub-comment based. Future versions may replace parts of prose parsing with structured schemas, validators, reducers, generated instructions, or stronger machine-enforced invariants.

The public generalization has not yet had broad external-user testing.

The release is therefore a **preview**, not a claim that every repository, account plan, GitHub organization policy, ChatGPT surface, or automation configuration behaves identically.

No public license has been selected in this preview branch yet. Public visibility on GitHub is not the same as an explicit open-source license. The maintainer should choose and add the intended license before treating this as a stable reusable release.

---

# A 60-second mental model

If everything above feels like a lot, remember this:

1. **Human asks for an outcome.**
2. **ChatGPT Project records a truthful mission when fleet coordination is useful.**
3. **A scheduled agent wakes up with only an identity.**
4. **It reads Phase0 and reconstructs its current mode from GitHub.**
5. **It inspects current evidence instead of trusting stale prose.**
6. **Before conflicting mutation, it claims narrow ownership.**
7. **It plans/predicts/audits/builds/integrates according to current state and human authority.**
8. **It records evidence, releases ownership, and appends the next AGENT_STATE.**
9. **Another run can continue without pretending it remembers hidden context.**
10. **The default starter still leaves final merge-to-main authority with the human.**

That is the initial public Phase0 idea.

For installation, download the named Starter ZIP and open `GETTING-STARTED.md` first.
