# README FIRST — FREE ENERGY starter pack

**FREE ENERGY** is the public project. This starter pack is for the **Fleet-Control Phase0** repository-backed agent fleet. The published `v0.1.2-phase0-preview` ZIP is a historical snapshot, **not** the latest `phase0/public-v0` branch.

If GitHub or ChatGPT Projects are new to you, begin with the [current online Getting Started guide](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/GETTING-STARTED.md) before copying anything. **The v0.1.2 ZIP contains older setup instructions**; use the online guide for corrected fork/default-branch steps.

**Preview fork warning:** The upstream repository's default `main` has a landing README but no Phase0 files. Forking only the default branch does **not** install Phase0. Either use the starter ZIP to populate a repository you own, or follow the current online guide to fork **all** branches and set `phase0/public-v0` as **your fork's** default branch. Before creating automations, verify that your repository's default view contains `Phase0/05-FLEET-CONFIG.md`.

## What goes where

### Into your GitHub repository

Copy the complete top-level `Phase0/` folder into the root of the repository your agents should work on.

Do not keep Phase0 only on your computer. Every persistent agent needs to read the same committed Phase0 files from GitHub.

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
