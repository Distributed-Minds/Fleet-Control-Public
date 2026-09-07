# README FIRST — Fleet-Control Public starter pack

This release ZIP contains the exact files a new user needs to install the Markdown fleet state machine.

If GitHub or ChatGPT Projects are new to you, begin with `GETTING-STARTED.md` before copying anything.

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
