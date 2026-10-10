# Getting Started with FREE ENERGY — no Git or ChatGPT experience required

This guide assumes you are starting from zero. You do **not** need to understand Git, branches, pull requests, state machines, or multi-agent systems before you begin.

**FREE ENERGY** is the public project. **Fleet-Control Phase0** is the current repository-local orchestration layer you are installing.

> **Important while FREE ENERGY is a preview (8 October 2026):** The upstream repository's default `main` branch currently has a FREE ENERGY landing README but does not contain the installable Phase0 files. The installable Phase0 files are on [`phase0/public-v0`](https://github.com/Distributed-Minds/Fleet-Control-Public/tree/phase0/public-v0). **A fork that copies only the default branch does not install FREE ENERGY.** Follow step 2 below before creating tasks. This warning can be removed when the upstream default branch actually contains the starter.

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

## Recommended preview setup: choose a working installation path

### 1. Create a GitHub account

Create an account at https://github.com/ and sign in.

### 2. Install the actual Phase0 preview in a repository you control

The upstream repository is still named `Distributed-Minds/Fleet-Control-Public` during the **FREE ENERGY** migration. Its default `main` branch currently contains a landing README but not the Phase0 distribution; use one of these two paths.

**Option A — GitHub website: fork all branches, then select the preview as your fork's default**

1. Inspect [the actual preview branch](https://github.com/Distributed-Minds/Fleet-Control-Public/tree/phase0/public-v0) and confirm it contains `Phase0/`.
2. Open [the repository](https://github.com/Distributed-Minds/Fleet-Control-Public) and choose **Fork**.
3. **Leave `Copy the DEFAULT branch only` unchecked.** GitHub otherwise copies `main`, which does not contain Phase0. [GitHub's fork instructions](https://docs.github.com/en/pull-requests/how-tos/work-with-forks/fork-a-repo?tool=webui) confirm that leaving it unchecked copies all branches.
4. Create the fork in your own account or organization.
5. **In your fork**, open **Settings → Default branch**; change it to `phase0/public-v0` and confirm. [GitHub's default-branch instructions](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-branches-in-your-repository/changing-the-default-branch) require repository admin access.
6. Return to your fork's home page. Confirm the default view shows the **FREE ENERGY** README and `Phase0/05-FLEET-CONFIG.md`. **Stop** if either is missing; otherwise your scheduled agents may read a landing-only branch without Phase0.

This fork method copies **all** upstream branches, including historical/research branches. Those extra branches are not an indication that their contents are active or part of the starter; only your selected default branch is the installation baseline. Do **not** change the upstream repository's default branch.

**Coordination does not arrive with a fork.** Copying all Git branches does not copy upstream GitHub issue threads. Before making recurring tasks, complete the version-specific coordination setup in step 6a **on your fork**, not in the upstream repository.

**Option B — clean existing repository: install the release ZIP**

Open [FREE ENERGY releases](https://github.com/Distributed-Minds/Fleet-Control-Public/releases), download the latest `FREE-ENERGY-Phase0-Starter-…-preview.zip`, and copy its entire `Phase0/` directory into the root of a repository you own; **commit** the files. Preserve any existing source code and your repository's own root `LICENSE`. This avoids copying upstream work-in-progress branches. The ZIP also contains the Project and automation prompt templates used in later steps.

**Check the published ZIP before copying it.** For the existing [v0.1.2 release](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/tag/v0.1.2-phase0-preview), download both `FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip` and its matching [`.zip.sha256` checksum file](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/download/v0.1.2-phase0-preview/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip.sha256) into the same folder. On a system with `sha256sum`, run this command **from that folder**:

```sh
sha256sum --check FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip.sha256
```

On **Windows PowerShell**, run this equivalent check from the folder containing both downloads. It rejects a malformed sidecar, a sidecar naming another ZIP, or a differing file digest:

```powershell
$zip = 'FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip'
$line = (Get-Content -LiteralPath "$zip.sha256" -Raw).Trim()
$pattern = '^(?<digest>[0-9a-fA-F]{64})[ \t]+\*?' + [regex]::Escape($zip) + '$'
$match = [regex]::Match($line, $pattern)
if (-not $match.Success) { throw 'Invalid checksum file or wrong ZIP filename' }
$actual = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash
if ($actual -ne $match.Groups['digest'].Value) { throw 'ZIP SHA-256 mismatch' }
Write-Output "OK: $zip matches its published checksum"
```

Continue only when the verification reports `OK`; **stop** on a missing file or checksum mismatch. Without `sha256sum`, calculate the downloaded ZIP's SHA-256 with a trusted hashing tool and compare it to the digest in the downloaded checksum file. Use the ZIP and checksum from the **same release**, and check that the checksum refers to the exact ZIP filename; do not assume future releases publish the same assets. Matching a checksum detects differences relative to that checksum file; it does **not independently authenticate the publisher**.

**Keep the Phase0 license notice with the installed files.** Check that your installed `Phase0/LICENSE` contains the [FREE ENERGY Phase0 MIT notice](LICENSE). The current online preview includes that file inside `Phase0/`; the historical v0.1.2 ZIP predates this addition, so when installing from that ZIP, copy the public [Phase0 MIT notice](LICENSE) into `Phase0/LICENSE` yourself before committing. **Do not overwrite your existing repository's root `LICENSE`:** Phase0's MIT notice covers the copied Phase0 files, not unrelated files in your project.

**Published ZIP version caveat:** The existing `v0.1.2-phase0-preview` release ZIP predates the corrected fork/default-branch instructions in this online guide. It is a historical starter package, **not** an archive of the current `phase0/public-v0` head. Follow these up-to-date online steps even if you use the ZIP, confirm `Phase0/05-FLEET-CONFIG.md` appears on **your repository's default branch**, and inspect the installed files before creating or enabling automations; do not rely on bundled older setup text alone.

**Coordination-version caveat:** The separate v0.1.2 **source tag** describes a one-issue coordination protocol, while the present `phase0/public-v0` branch requires two trusted A/B slots. The published ZIP's actual contents are **not proven identical to the tag merely by inspecting the tag**. For a ZIP installation, inspect the downloaded `Phase0/05-FLEET-CONFIG.md`, `Phase0/30-COORDINATION.md` and whether `Phase0/160-COORDINATION-SLOTS.md` exists; use **that installed contract**, not an unconditional preview two-slot instruction. If versions disagree or an upgrade is needed, stop and make an explicit, reviewed installation/upgrade decision rather than silently mixing protocols.

Either way, your fleet must target **your repository**, never the upstream `Distributed-Minds/Fleet-Control-Public` repository. Continue only once `Phase0/05-FLEET-CONFIG.md` is visible on **your repository's default branch**.

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

### 4. Connect GitHub and establish the capability you actually have

Open ChatGPT **Settings → Apps** (or **Plugins**, if that is the surface your account shows), choose GitHub, sign in, and authorize **your repository**. Installing the GitHub connection and granting it access to a particular repository are separate steps. An organization owner may need to approve access.

Do not assume that every ChatGPT, app/plugin, interactive, and scheduled surface exposes the same GitHub actions. The exact capability can vary with product surface, account/workspace policy, app/plugin permissions, repository installation scope, organization approval, and GitHub repository rules. A connected repository does not by itself prove write capability.

Where the current product exposes action-permission controls, use the least privilege compatible with the intended work. An action can still require approval, be denied, or be blocked by GitHub even when a broader product permission is selected.

Before you rely on scheduled agents to mutate GitHub, use the scheduled/background execution context itself to establish the required capability. Follow `Phase0/95-GITHUB-CAPABILITY-ACCEPTANCE.md`. Prefer a non-destructive check. If only a reversible mutation can prove the capability, use the contract's disposable non-default probe, stable lineage, exact-resource, current-authority, recovery, and cleanup rules. Never use a capability probe on the default branch.

If an interactive chat can perform an action but the scheduled context is read-only, blocked, approval-paused, or unknown, treat the scheduled context accordingly. Do not infer equivalence.

A newly created or newly authorized repository may take some time to appear.

### 5. Create a ChatGPT Project

In ChatGPT, choose **New project** in the sidebar. Name it anything you like, for example `FREE ENERGY — My Game Fleet`.

Open the project menu (`...`) → **Project settings**.

Use the Project-instructions template from the installation path you chose:

- **Option A — fork:** open [`Phase0/templates/PROJECT-INSTRUCTIONS.md`](Phase0/templates/PROJECT-INSTRUCTIONS.md) on **your fork's default branch**. The fork already includes this template; no release ZIP is needed for this path.
- **Option B — release ZIP:** open `COPY-INTO-CHATGPT/PROJECT-INSTRUCTIONS.md` from the extracted ZIP.

Copy the entire chosen template into Project instructions. Replace every occurrence of:

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

This means agents may prepare branches and PRs, but a human decides what enters the default branch. Technical capability never silently changes this policy authority.


### 6a. Bootstrap and verify your own coordination issues **before scheduling**

**Stop gate:** Do this only after the files from your chosen installation are committed and visible on **your repository's default branch**. GitHub issues belong to a repository, not to its branches. A fork or copied `Phase0/` folder does **not** copy the upstream coordination issues. Do not point your fleet at upstream [#168](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/168) or [#186](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/186), and do not have an unattended agent blindly create them.

**First identify your installed protocol.** Inspect `Phase0/05-FLEET-CONFIG.md` and `Phase0/30-COORDINATION.md` on **your repository's default branch**. If `Phase0/160-COORDINATION-SLOTS.md` exists, inspect it too. In the **current preview (Option A)**, the installed [configuration](Phase0/05-FLEET-CONFIG.md) and [slot contract](Phase0/160-COORDINATION-SLOTS.md) require **exactly two trusted open coordination issues**, one for slot A and one for slot B. If your files lack or contradict that contract (notably an older release ZIP), **do not** apply these A/B creation instructions; follow your actual installed `30-COORDINATION.md` and resolve incompatible versions before enabling automations.

**For an installation of the current two-slot preview:**

1. In **your repository** confirm GitHub Issues is available. Search existing *open* issues for the exact title `[fleet-control] coordination` before making anything. If any matching issue is malformed, untrusted, or duplicated, **stop** for maintainer review rather than creating a replacement and hoping the fleet picks it.
2. Open `Phase0/05-FLEET-CONFIG.md` on **your default branch**. Its `COORDINATION_TRUSTED_AUTHORS=geromet` is an upstream example, **not** automatic permission for your account or your fork. An authorized maintainer must set `COORDINATION_TRUSTED_AUTHORS` to the actual approved GitHub login(s) who will **create** the issues (for example `COORDINATION_TRUSTED_AUTHORS=alice`), and commit that configuration on the installation's default branch. Do not leave `geromet` by accident or trust a user just because a comment or issue body claims their name.
3. **Verify the fork's effective runtime trusted-author source before creating issues.** The value in `Phase0/05-FLEET-CONFIG.md` is *declared policy*, not the authority actually used by the archiver or GitHub Actions. In the current preview, `scripts/coordination_archive.py` resolves trusted authors from explicit `--trusted-authors`, then process environment `COORDINATION_TRUSTED_AUTHORS`, then the fallback `geromet`. The installed `.github/workflows/coordination-slots.yml` uses its own repository Actions variable `vars.COORDINATION_TRUSTED_AUTHORS` (or `geromet` if unset/empty) and passes it as `--trusted-authors`; the workflow's `issue_comment` jobs also check the raw Actions value independently.
   
   - **An authorized maintainer**, in **the fork's own repository**, goes to **Settings → Secrets and variables → Actions → Variables** and verifies/sets the repository variable `COORDINATION_TRUSTED_AUTHORS` to match the committed Markdown creator-login list. Examples: `alice` or `alice,bob`. Use literal commas, **no spaces, empty items or case-insensitive duplicates**. Do not copy the upstream `geromet` fallback for an unrelated new owner's fork. Do not silently change these settings through an agent.
   - Check that the workflow is actually installed on this repository's **default branch** and that its effective `COORDINATION_SOURCE_REF` selects the intended installed `Phase0/` protocol. Do **not** infer that a preview branch, historical ZIP, or Markdown edit installs the default-branch Actions workflow. Record the actual variable source, not a guessed value. An explicit CLI override must be reported separately; it does not update the workflow's event-admission variable.
   - **Verify that GitHub Actions and this workflow are enabled in the fork.** GitHub documents that workflows do not run in newly forked repositories by default. In the fork's **Actions** tab, an authorized human maintainer must enable Actions and, if necessary, enable the `Coordination slots maintenance` workflow, including scheduled execution. Confirm that the workflow is active and that applicable repository/organization Actions policies permit it; a committed YAML file and correct variables do not prove dispatch capability. If the enabled state or policy cannot be verified, report **BLOCKED: fork Actions/workflow disabled or unknown** before scheduling. Do not silently change Actions settings through an agent.
   - If the Actions variable, default-branch workflow, effective source ref, or CLI/environment provenance cannot be checked, or if the declared/effective creator sets differ, record **BLOCKED: trusted-author source unknown or mismatched**. Do not create agents or accept a green slot-only check as installation readiness.

4. An approved human author from that **verified effective and declared** list creates **two open issues in your repository** via **Issues → New issue**, both titled exactly `[fleet-control] coordination`. The first issue body must start *at the first character of the first line*, without a Markdown code fence, with:

   ```text
   FLEET_COORDINATION_V1
   COORDINATION_SLOT=A
   COORDINATION_STATE=ACTIVE
   COORDINATION_EPOCH=1
   ```

   The second issue has its own body (also without a wrapping Markdown fence):

   ```text
   FLEET_COORDINATION_V1
   COORDINATION_SLOT=B
   COORDINATION_STATE=STANDBY
   COORDINATION_EPOCH=0
   ```

   Extra explanatory text may follow after a blank line. The **GitHub creator account**, not a self-declared field, must be in the configured trusted-author list. These are setup examples only; do not edit any already-running slot state to force it to match them.
5. Perform a **read-only preflight** against (a) the exact default-branch `Phase0/05-FLEET-CONFIG.md` declared policy, (b) the effective installed workflow/Actions variable and any explicit CLI override, (c) the API-observed GitHub issue creators and bodies, (d) the workflow's raw trusted-commenter admission predicates when installed, and (e) whether fork Actions and the intended workflow are actually enabled. Do not accept whitespace-padded, empty, duplicate, or otherwise invalid new-installation author tokens just because the current Python parser would trim them. A valid installation must agree on who is authorized at every layer; any mismatch, inaccessible source, unrecognized workflow version or unknown provenance is **BLOCKED** before scheduling. Then check exactly one valid *trusted* open issue per slot (A and B), exact title and first-line marker, unique `COORDINATION_SLOT`, recognized `COORDINATION_STATE` and nonnegative integer `COORDINATION_EPOCH`, and actual API creator accounts in the **effective** approved list. Initially A ACTIVE/1 and B STANDBY/0 is valid. During rotation two ACTIVE issues can temporarily coexist; the **higher epoch** wins. No ACTIVE issue, conflicting equal highest ACTIVE epochs, a missing/closed/untrusted/malformed issue, or a duplicate valid slot is **BLOCKED**, never permission to guess a slot or write to a historical issue.
6. Confirm usable coordination comment capacity (GitHub limits an issue to 2,500 comments; current preview's switch threshold defaults to 2,000 and standby headroom to 500). Confirm that the intended agent execution context can actually **read** both trusted issues and, separately, has the required GitHub action capabilities under step 4 and `Phase0/95-GITHUB-CAPABILITY-ACCEPTANCE.md`. A successful manual inspection does **not** prove scheduled GitHub write access.

**Readiness result:** Only mark coordination **READY** when the installed protocol and effective `COORDINATION_SOURCE_REF`, committed declared trust set, actual Actions/CLI/environment trust sources, enabled fork Actions/workflow state, the workflow's event-admission set where installed, trusted GitHub issue creators, both live slot bodies, unique ACTIVE authority, headroom and required read capability are independently checked. If the Actions variable is unset, the current installed workflow's effective trusted set is **`geromet`**, not the fork owner's login; a Markdown edit alone must never mark a new fork READY. Otherwise report exactly what is **BLOCKED** (for example "B missing", "creator not trusted", "duplicate A", "conflicting ACTIVE epoch", "Issues inaccessible", or "scheduled writes unverified"), fix it through an authorized human action and rerun the read-only check. Do **not** create recurring agents while blocked; do not modify the upstream repository, branch protection, schedules or GitHub permissions merely to make the check pass.

**Historical release ZIP:** Do not assume the two-slot preflight above is applicable. Inspect the *downloaded ZIP's installed* `05-FLEET-CONFIG` and `30-COORDINATION` contract first; use its version-specific issue-count and trust rules. A tagged source tree is not proof of ZIP bytes. If you cannot determine the installed protocol or safely upgrade it, stop scheduling and request maintainer review.

### 7. Create persistent scheduled agents

**Before creating or enabling any recurring task:** complete step 6a's installed-version coordination preflight on **your repository**, including the fork's actual Actions/workflow enabled state, and independently establish the scheduled-context capability required in step 4. If either is blocked or unknown, stop here; creating a task is not a substitute for trusted slot setup.

Use the automation prompts from your chosen installation path:

- **Option A — fork:** open [`Phase0/templates/AUTOMATION-PROMPT.md`](Phase0/templates/AUTOMATION-PROMPT.md) on **your fork's default branch**. It is a generic template: make one separate prompt copy for every enabled identity (`A1`, `A2`, etc.). Replace `<AGENT_ID>` in each copy with that identity; never leave the placeholder unchanged or reuse one identity for multiple automations.
- **Option B — release ZIP:** the ZIP includes an individual prompt for each identity:

  ```text
  COPY-INTO-AUTOMATIONS/A1.md
  COPY-INTO-AUTOMATIONS/A2.md
  COPY-INTO-AUTOMATIONS/A3.md
  COPY-INTO-AUTOMATIONS/A4.md
  COPY-INTO-AUTOMATIONS/A5.md
  ```

For each agent you enable:

1. open its ZIP prompt or the corresponding separate fork-template copy;
2. replace `<OWNER>/<REPOSITORY>` with your repository, and replace `<AGENT_ID>` with the matching identity if you used the generic fork template;
3. create a recurring task/automation in the supported ChatGPT surface you use;
4. paste the prompt as the task instruction;
5. choose a schedule appropriate for your plan and workload;
6. establish scheduled-context GitHub capability for every action class the fleet is expected to use, following `Phase0/95-GITHUB-CAPABILITY-ACCEPTANCE.md`.

Do **not** give all automations the same identity. A1 must remain A1, A2 must remain A2, and so on.

In ChatGPT, supported recurring tasks are managed from **Scheduled**. Availability, supported apps/plugins, approval behavior, and frequency limits can vary by account/workspace and current product surface.

Important: scheduled ChatGPT tasks may not be able to access files uploaded directly to a ChatGPT Project. Fleet-Control therefore keeps the durable fleet operating system in GitHub and tells every persistent run to re-read it there.

### 8. Give the fleet real work

Open a normal chat inside your FREE ENERGY / Fleet Project and ask for the outcome you want.

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

- the coordination issue(s) required by the **installed** protocol (two trusted A/B issues for the current preview, not an unconditional single issue);
- mission issues;
- implementation issues;
- non-default branches;
- pull requests;
- machine-ish state comments.

Those artifacts appear only when the installed protocol, permissions and current authority permit them. Do not delete coordination issues just because they look repetitive; they are durable collision/state logs. Initial bootstrap is a deliberate maintainer preflight, **not** a promise that scheduled agents can create missing trusted issues.

## What the fleet should not do by default

Unless you explicitly change the rules, it should not:

- merge into your default branch;
- mutate unrelated repositories;
- treat connection or product capability as policy authority;
- treat interactive GitHub capability as proof of scheduled capability;
- widen permissions or disable protections merely to make a capability check pass;
- turn “think about this” into product code changes;
- claim tests passed without evidence;
- claim another executor completed work without observing the resulting repository state;
- create duplicate work just to keep every agent busy.

## If you do not want to fork

### Existing repository

Download the release ZIP and follow **Option B** above rather than relying on the ZIP's older bundled instructions. Copy its complete `Phase0/` directory into a repository you own. If `Phase0/` already exists, inspect and reconcile the files instead of overwriting them. For the historical `v0.1.2` ZIP, copy the [public Phase0 MIT notice](LICENSE) to `Phase0/LICENSE` before committing, without replacing your project's root `LICENSE`. Commit on **your repository's default branch**, verify both `Phase0/05-FLEET-CONFIG.md` and `Phase0/LICENSE` are present there, and use that repository's exact `OWNER/REPOSITORY` in Project instructions and automation prompts.

### Command line

If you already use Git and want to install Phase0 into **an existing repository you control**, clone the **preview branch explicitly**, not upstream `main`:

```sh
git clone --depth 1 --single-branch --branch phase0/public-v0 \
  https://github.com/Distributed-Minds/Fleet-Control-Public.git free-energy-preview
```

Confirm `free-energy-preview/Phase0/05-FLEET-CONFIG.md` and `free-energy-preview/Phase0/LICENSE` exist. On **your own repository's default branch**, copy the complete `free-energy-preview/Phase0/` folder into the repository root, inspect the result, and commit it there. If a `Phase0/` folder already exists, review and reconcile its contents instead of overwriting it. **Do not copy the upstream `.git/` directory, replace your project's root `LICENSE`, or push to the FREE ENERGY upstream.** Configure `Phase0/05-FLEET-CONFIG.md` and confirm that it appears on your own default branch before scheduling agents.

**Command-line route — continue with the templates you copied:** Because this route copies only `Phase0/`, use `Phase0/templates/PROJECT-INSTRUCTIONS.md` for step 5 and `Phase0/templates/AUTOMATION-PROMPT.md` for step 7 from **your own repository's default branch**. Do not look for the release-ZIP-only `COPY-INTO-CHATGPT/` and `COPY-INTO-AUTOMATIONS/` folders in that repository. Replace `<OWNER>/<REPOSITORY>` in both templates; make a **separate automation-prompt copy per enabled agent**, replacing `<AGENT_ID>` with that agent's unique identity. Confirm the installed files and scheduled-context permissions before enabling any automations.

A plain clone without `--branch phase0/public-v0` checks out upstream `main`, which **does not contain Phase0**. If you want a full GitHub-hosted fork rather than copying only the starter, follow **Option A** above. “Clone” means copying the repository to your computer; “fork” means GitHub makes a repository copy in your account.

## Before important work

Start with a disposable repository or non-critical project. Watch several runs. Read the issues and PRs. Keep `HUMAN_MERGE_ONLY` until you understand the behavior.

The Markdown state machine coordinates work; it does not replace ordinary repository permissions, backups, CI, tests, security inspection, or human judgment.

## Optional: verify onboarding links from source (maintainers)

The public documentation link checker is maintained in **Rust**, not Python or npm. This step is for contributors editing the source checkout, **not** a prerequisite for installing Phase0 or using the published v0.1.2 ZIP. With `rustc` available, run from the checkout root:

```sh
rustc --edition=2021 -D warnings --test scripts/check-public-doc-links.rs -o /tmp/free-energy-doc-links-tests &&
/tmp/free-energy-doc-links-tests &&
rustc --edition=2021 -D warnings scripts/check-public-doc-links.rs -o /tmp/free-energy-doc-links &&
/tmp/free-energy-doc-links --root "$PWD"
```

The last command checks local file targets in ten onboarding documents (including `CONTRIBUTING.md` and `GLOSSARY.md`), rejecting missing, malformed or unsafe destinations. It does **not** request external URLs, validate `#fragments`, or implement full CommonMark. A nonzero exit is a failed check. Pass `--root` explicitly because the compiled binary may live outside the checkout. The pinned GitHub Actions validator additionally exercises executable failure cases on a disposable fixture root.
