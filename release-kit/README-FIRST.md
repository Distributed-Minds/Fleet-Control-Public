# README FIRST — FREE ENERGY starter pack

**FREE ENERGY** is the public project. This starter pack is for the **Fleet-Control Phase0** repository-backed agent fleet. The published `v0.1.2-phase0-preview` ZIP is a historical snapshot, **not** the latest `phase0/public-v0` branch.

If GitHub or ChatGPT Projects are new to you, begin with the [current online Getting Started guide](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/GETTING-STARTED.md) before copying anything. **The v0.1.2 ZIP contains older setup instructions**; use the online guide for corrected fork/default-branch steps.

**Published ZIP integrity check (v0.1.2 only):** Before installing files from the downloaded ZIP, retrieve its [`.sha256` sidecar](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/download/v0.1.2-phase0-preview/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip.sha256), run `sha256sum FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip` on Linux, and compare the digests. **Do not install if the sidecar is missing or the digests differ.** A match establishes consistency with the checksum published alongside that ZIP, **not** independent authenticity, content safety, rights clearance, or currency of the bundled guide. The [current installation checklist](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/release-kit/INSTALL-CHECKLIST.md) has the ZIP-specific check; it does not apply to the all-branches fork path.

**Preview fork warning:** The upstream repository's default `main` has a landing README but no Phase0 files. Forking only the default branch does **not** install Phase0. Either use the starter ZIP to populate a repository you own, or follow the current online guide to fork **all** branches and set `phase0/public-v0` as **your fork's** default branch. Before creating automations, verify that your repository's default view contains `Phase0/05-FLEET-CONFIG.md`.

## What goes where

### Into your GitHub repository

Copy the complete top-level `Phase0/` folder into the root of the repository your agents should work on.

**Historical v0.1.2 ZIP license fix:** The published ZIP predates the `Phase0/LICENSE` file added to the current online preview. If installing from that ZIP, copy the [current FREE ENERGY Phase0 MIT notice](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/LICENSE) into `Phase0/LICENSE` before committing. If installing from the current preview, preserve its existing `Phase0/LICENSE`. **Keep your own project's root `LICENSE` unchanged:** Phase0's MIT notice does not relicense unrelated project files. See the [current installation checklist](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/release-kit/INSTALL-CHECKLIST.md).

Do not keep Phase0 only on your computer. Every persistent agent needs to read the same committed Phase0 files from GitHub.

**ZIP layout versus source checkout:** The `COPY-INTO-CHATGPT/` and `COPY-INTO-AUTOMATIONS/` locations below describe the starter ZIP layout; **they do not exist in a checkout or fork of the current `phase0/public-v0` source tree**. For that source checkout, use [`Phase0/templates/PROJECT-INSTRUCTIONS.md`](../Phase0/templates/PROJECT-INSTRUCTIONS.md) for your ChatGPT Project and [`Phase0/templates/AUTOMATION-PROMPT.md`](../Phase0/templates/AUTOMATION-PROMPT.md) for each scheduled agent. Replace `<OWNER>/<REPOSITORY>` in both templates and give **each copy** of the automation prompt a unique `<AGENT_ID>` (for example, `A1` through `A5`), aligned with `Phase0/05-FLEET-CONFIG.md`. These templates are instructions, not evidence that a scheduled context has GitHub mutation capabilities; verify those separately using the current online setup guide.

### Into ChatGPT Project instructions

Open:

```text
COPY-INTO-CHATGPT/PROJECT-INSTRUCTIONS.md
```

Copy its entire contents into your ChatGPT Project instructions and replace `<OWNER>/<REPOSITORY>` with your repository name.

### Into persistent automations/tasks

Open the files in:

```text
COPY-INTO-AUTOMATIONS/
```

Each file is a different persistent identity. Replace `<OWNER>/<REPOSITORY>` in every prompt, then create one recurring automation per identity you actually enable.

If you configure only three agents, use A1, A2, and A3 and set `FLEET_SIZE=3` / `AGENTS=A1,A2,A3` in `Phase0/05-FLEET-CONFIG.md`.

## Do not upload secrets

Do not put passwords, API keys, private tokens, recovery codes, or other credentials into Phase0 files, Project instructions, task prompts, issues, or public repositories.
