# Getting Started — no Git or ChatGPT experience required

This guide assumes you are starting from zero. You do **not** need to understand Git, branches, pull requests, state machines, or multi-agent systems before you begin.

## What you are setting up

You will have three pieces:

1. **Your GitHub repository** — the place where your project files and the Fleet-Control `Phase0/` rules live.
2. **A ChatGPT Project** — the place where you talk to ChatGPT about what you want the fleet to do.
3. **Persistent scheduled agents** — recurring agents with stable identities such as `A1`, `A2`, and `A3`. Each run reads the same repository rules and reconstructs its own state from GitHub.

The GitHub repository is the durable shared memory for the fleet. The scheduler prompts are deliberately small.

## Five words worth knowing

- **Repository (repo):** a project folder stored on GitHub, with history.
- **Branch:** a separate line of changes inside a repository. Agents use non-default branches so they do not directly overwrite your main line of work.
- **Pull request (PR):** a proposal to combine a branch into another branch, usually the default branch.
- **Fork:** GitHub makes a copy of someone else's public repository inside your own GitHub account.
- **Clone:** Git copies a repository onto your computer. You do not need to clone anything for the easiest setup.

## Recommended beginner setup: fork first

### 1. Create a GitHub account

Create an account at https://github.com/ and sign in.

### 2. Make your own copy

Open https://github.com/Distributed-Minds/Fleet-Control-Public and choose **Fork** near the top-right.

Choose your own GitHub account or organization and create the fork. Your fleet should work on **your fork**, not on `Distributed-Minds/Fleet-Control-Public`.

If you already have a repository for an existing project, you can instead copy the supplied `Phase0/` folder into that repository. Do not replace your existing source code.

### 3. Write down your repository name

GitHub repository names look like:

```text
OWNER/REPOSITORY
```

Example:

```text
alice/my-project
```

You will paste this exact value into the Project instructions and automation prompts.

### 4. Connect GitHub

For ChatGPT repository reading/searching, open ChatGPT **Settings → Apps** (or **Plugins**, if that is what your account shows), choose GitHub, sign in, and grant access to **your repository**.

Installing the GitHub app and granting it access to a particular repository are separate steps. If the repository belongs to a GitHub organization, an organization owner may need to approve access.

For code editing/pushing, use Codex with access to the same repository. The ordinary ChatGPT GitHub app may be read-only in some product surfaces; Codex is the OpenAI product intended for generating, editing, and pushing code to GitHub.

A newly created or newly authorized repository may take a few minutes to appear.

### 5. Create a ChatGPT Project

In ChatGPT, choose **New project** in the sidebar. Name it anything you like, for example `My Agent Fleet`.

Open the project menu (`...`) → **Project settings**.

From the release ZIP, open:

```text
COPY-INTO-CHATGPT/PROJECT-INSTRUCTIONS.md
```

Copy the entire file into Project instructions. Replace every occurrence of:

```text
<OWNER>/<REPOSITORY>
```

with your exact repository name.

Project instructions apply only inside that ChatGPT Project.

### 6. Configure the fleet

In your repository, open:

```text
Phase0/05-FLEET-CONFIG.md
```

The starter defaults to five agents:

```text
FLEET_SIZE=5
AGENTS=A1,A2,A3,A4,A5
```

You can use fewer agents. Keep the count and list consistent.

For a first setup, leave:

```text
DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY
```

This means agents may prepare branches and PRs, but a human decides what enters the default branch.

### 7. Create persistent scheduled agents

The release ZIP contains one ready-made prompt per identity:

```text
COPY-INTO-AUTOMATIONS/A1.md
COPY-INTO-AUTOMATIONS/A2.md
COPY-INTO-AUTOMATIONS/A3.md
COPY-INTO-AUTOMATIONS/A4.md
COPY-INTO-AUTOMATIONS/A5.md
```

For each agent you enable:

1. open its file;
2. replace `<OWNER>/<REPOSITORY>` with your repository;
3. create a recurring task/automation in the ChatGPT/Codex surface you use;
4. paste the prompt as the task instruction;
5. choose a schedule appropriate for your plan and workload.

Do **not** give all automations the same identity. A1 must remain A1, A2 must remain A2, and so on.

In ChatGPT, supported recurring tasks are managed from **Scheduled**. Availability and frequency limits depend on account/plan. Codex automations are separate from ChatGPT scheduled tasks.

Important: scheduled ChatGPT tasks may not be able to access files uploaded directly to a ChatGPT Project. Fleet-Control therefore keeps the durable fleet operating system in GitHub and tells every persistent run to re-read it there.

### 8. Give the fleet real work

Open a normal chat inside your Fleet Project and ask for the outcome you want.

Examples:

```text
Think about whether we should redesign our permissions model.
```

```text
Research why our API sometimes duplicates jobs. Don't change code yet.
```

```text
Fix issue #42. Don't merge into main.
```

```text
Build a small recipe-sharing web app.
```

```text
Make a Facebook-like social network prototype. Start by decomposing the problem and creating implementation-ready work packages.
```

The Project instructions turn substantial requests into durable missions. Scheduled agents then read those missions and use the shared state machine to decide how to contribute without all doing the same thing.

### 9. Expect GitHub artifacts to appear

The fleet may create:

- one coordination issue;
- mission issues;
- implementation issues;
- non-default branches;
- pull requests;
- machine-ish state comments.

That is expected. Do not delete the coordination issue just because it looks repetitive; it is the fleet's durable collision/state log.

## What the fleet should not do by default

Unless you explicitly change the rules, it should not:

- merge into your default branch;
- mutate unrelated repositories;
- turn “think about this” into product code changes;
- claim tests passed without evidence;
- claim another executor completed work without observing the resulting repository state;
- create duplicate work just to keep every agent busy.

## If you do not want to fork

### Existing repository

Download the release ZIP. Copy its `Phase0/` folder into the root of your existing repository and commit it. Then use that repository's exact `OWNER/REPOSITORY` name in Project instructions and automation prompts.

### Command line

If you already use Git, you can clone the public repository locally, add a remote for a repository you own, and push it there. “Clone” means copy to your computer; “fork” means GitHub creates the copy in your account.

## Before important work

Start with a disposable repository or non-critical project. Watch several runs. Read the issues and PRs. Keep `HUMAN_MERGE_ONLY` until you understand the behavior.

The Markdown state machine coordinates work; it does not replace ordinary repository permissions, backups, CI, tests, security review, or human judgment.
