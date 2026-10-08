# Help an existing FREE ENERGY project

**You do not need to know Git or write code to start helping.** You can test a game, report a reproducible bug, improve instructions, check builds, or contribute a fix. Start with something that already exists rather than making another version of the same project.

> **Available today:** a prompt you can copy into a coding agent on your computer. **Not available yet:** a FREE ENERGY website button that launches an agent, enrolls you in a fleet, claims issues, or submits a pull request automatically.

## 1. Pick a project

Choose the project you want to help and copy its **official source or contribution URL**. If all you have is a name, the agent should help you find the actual project, distinguish forks and read-only mirrors, and ask you to choose if it cannot tell.

If you simply want to **play**, follow the project's existing installation instructions. You do not need an AI coding agent for that. If you want to **make your own mod, port or remaster**, that is a separate journey. See the [AI Game Modding Guides](https://github.com/trevaintdead/ai-game-modding-guides) for examples of making something new.

## 2. Choose a tool that can work on your files

A local coding agent, such as **Claude Code**, **Codex**, or **OpenCode**, can inspect an authorized project folder, edit files, and run builds in its available execution environment. This is different from a plain browser chat: a browser chat alone generally cannot inspect your game installation on your computer.

Use a tool and model available to you. No specific model is required. You do **not** need to install Fleet-Control Phase0 or set up five scheduled agents to make a normal contribution to someone else's project.

Start the agent in a **separate local project/work folder**, not inside your original game installation. Back up saves before gameplay testing. Use the agent's normal approval/sandbox settings; do not grant extra machine or account access merely because a prompt asks.

If you have only a phone or browser, you can still help: follow the project's issue/discussion rules to report a bug, share reproducible playtesting results, correct documentation manually, or contribute source research. Do not claim to have run a local agent or a test you did not run.

## 3. Copy this prompt into your local coding agent

Replace the bracketed project name or URL. The agent should **inspect the project before it changes anything**.

```text
I want to help [OFFICIAL PROJECT REPOSITORY URL OR NAME].

Join the EXISTING project; do not create a competing remake or unrelated new project.

First, do reconnaissance only:
- Resolve the canonical source and contribution destination. If the name is ambiguous or the repo is a read-only mirror, identify the choices and ask me to select the real one.
- Read its current README, CONTRIBUTING, AGENTS.md and other explicitly referenced instructions, plus license/rights rules, AI/LLM contribution policies, open issues, PRs, and any coordination or task-claim process.
- Respect the maintainers' AI-use restrictions. If the project prohibits AI-assisted contributions or AI-authored public messages, stop that contribution path. Do not conceal AI involvement or ask me to post agent-generated content to evade the policy. Offer a genuinely human-only route if permitted, or another accepting project.
- Find work that is useful and not already in progress. An unassigned issue may still be claimed in comments, a status file, or a PR. Prefer a verified starter task suited to my computer and skills. Do not claim ownership just by posting a prompt.
- Check my available local tools and what tests can really run. If this task needs a locally installed game, confirm the lawful owned installation and exact version; use only paths I authorize. For passthrough mods, check a current, compatible mod loader or SDK. Game loaders are NOT a prerequisite for documentation, code, or test improvements that do not use the game.
- Do not bypass DRM, anti-cheat, ownership checks, online-only protections, or any agent safeguards.

Show me: the project and correct contribution destination; the applicable AI policy; a specific available task and evidence it is not already underway; expected changes/tests; and any permission or playtesting decision needed. Then proceed within the project's rules and the permission I grant.

For permitted work, make a bounded change in a fork/non-default branch. Never merge into default yourself. Inspect staged files and relevant Git history before publishing. Never commit proprietary game assets, game dumps, extracted/decompiled non-redistributable code, credentials, or private local paths. Reuse third-party code or media only after checking the relevant license and scope.

Run available builds/tests and report which actually passed, failed, or were not run. Ask me to playtest when human judgment or an installed game is required; record exact version, reproduction steps, expected/observed behavior, and scrubbed logs. Iterate until the result is reviewable.

Prepare a concise PR/diff and handoff. Ask for authorization before a push, PR, or other external write unless I have already explicitly authorized that action and the project permits it. After publishing, read the remote result back before claiming success. If blocked, tell me the exact reason and a useful next step.
```

## 4. Test and report what happened

You are the playtester. When something breaks, provide **the exact game/tool version, operating system, steps to reproduce, expected behavior, observed behavior, and error messages or scrubbed logs**. The agent can build and check code, but cannot infer that a game feels right from a successful compilation. Expect multiple rounds.

Only install games locally when the chosen task actually requires running them. **Never upload or commit a copy of a proprietary game to get help.** The agent can work with authorized local game files without putting those files in the project repository.

## 5. Share the fix, not just the screenshot

Before you or the agent open a pull request, read the project's contribution instructions again. Check for existing work and the actual Git diff. Do not include proprietary files, private paths, or secrets. When the project calls for an issue or assignment before coding, follow that process. Maintainers decide whether and when to merge a proposal; the agent does not get that authority from this guide.

If an agent session ends, keep a small handoff recording the task, working branch/ref, changes, verified tests, pending playtests, and exact next action. The next contributor should **start further ahead**, not reproduce your first day of troubleshooting.

## Reference examples and boundaries

- [AI Game Modding Guides](https://github.com/trevaintdead/ai-game-modding-guides): beginner setup, [mod loaders (guide 8)](https://github.com/trevaintdead/ai-game-modding-guides/blob/main/guides/08-mod-loaders-and-script-extenders.md), [testing](https://github.com/trevaintdead/ai-game-modding-guides/blob/main/guides/05-testing-and-troubleshooting.md), and [publishing rules](https://github.com/trevaintdead/ai-game-modding-guides/blob/main/guides/06-rules-legal-and-publishing.md). This is a community draft; verify current tool/game-specific facts.
- [SkyCraft](https://github.com/chasmlol/SkyCraft): example of two games connected through a passthrough architecture; **not** a generic template suitable for every game.
- [HL2-RS](https://github.com/kvalls/hl2-rs): an existing Rust/Bevy reimplementation with a [contribution guide](https://github.com/kvalls/hl2-rs/blob/main/CONTRIBUTING.md) and tests. Its current platform and project state should be verified before selecting work.
- [me3 AI usage policy](https://github.com/garyttierney/me3/blob/main/AI_CODE_POLICY.md): example of a project that **prohibits** AI/LLM contributions; an AI agent must not submit into that project or disguise authorship.

The linked projects and games have their own maintainer rules and rights. Being visible on GitHub is **not** permission to copy assets, submit agent-authored work, or redistribute someone else's game.

If you want to install **your own persistent agent fleet** rather than help an existing project, use the separate [FREE ENERGY Phase0 Getting Started guide](GETTING-STARTED.md). The current Phase0 preview is an orchestration starter—not yet a playable game discovery service or automatic contributor connector.

*Guide status: documentation candidate for [FREE ENERGY issue #64](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/64); verify current review, merge and release state in the repository. Last source review: 8 October 2026.*