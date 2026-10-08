<h1 align="center">FREE ENERGY</h1>

<p align="center">
  <strong>Federated Remastering Ecosystem for Everyone</strong><br>
  <em>Engine for Networked Entertainment, Remixing, Games &amp; You</em>
</p>

<p align="center"><strong>FREE ENERGY Remasters Everything.</strong></p>

<p align="center">
  <a href="https://github.com/Distributed-Minds/Fleet-Control-Public/releases/tag/v0.1.2-phase0-preview">Current preview</a> ·
  <a href="https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/GETTING-STARTED.md">Install Phase0</a> ·
  <a href="https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/HELP-A-PROJECT.md">Help a project</a> ·
  <a href="https://github.com/Distributed-Minds/Fleet-Control-Public/discussions">Join the discussion</a> ·
  <a href="https://github.com/Distributed-Minds/Fleet-Control-Public/issues">Explore issues</a>
</p>

---

## Play first. Fix once. Make the next person's start easier.

FREE ENERGY is an open, collaborative effort to help people and AI-agent fleets **repair, port, remaster, remix, and eventually connect games**—and share the tools, knowledge, assets, and verified fixes that make the work reusable.

**FREE ENERGY** is the public entertainment/remastering ecosystem. **Fleet-Control** is the agent-orchestration infrastructure underneath it. The ambitions are much larger than the software available today.

## Start with what works today

| Available now | Being developed | Longer-term vision |
| :--- | :--- | :--- |
| **Phase0 Preview v0.1.2** — a Markdown-first starter for coordinating GitHub-connected AI agents | Contributor workflows, reproducible game pilots, and rights-aware remastering tools | Game discovery and play, ports, remasters, reusable mods, and cross-game interoperability |
| [Get the starter ZIP](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/download/v0.1.2-phase0-preview/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip) | [See public development work](https://github.com/Distributed-Minds/Fleet-Control-Public/issues) | [Read the product direction](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/63) |

> [!IMPORTANT]
> **Release-install caveat:** The published v0.1.2 ZIP predates the [corrected fork/setup instructions (PR #59)](https://github.com/Distributed-Minds/Fleet-Control-Public/pull/59). Follow the [current online guide](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/GETTING-STARTED.md) rather than relying on the ZIP's bundled instructions alone.

> **There is no shipped playable-game platform, game catalog, universal adapter, or hosted fleet service yet.** Phase0 is a coordination starter, not the finished FREE ENERGY product.

## Choose your starting point

| You want to… | Start here |
| :--- | :--- |
| **Try the current release** | [Download Phase0 v0.1.2](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/tag/v0.1.2-phase0-preview), follow the [beginner guide](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/GETTING-STARTED.md), and check the [ZIP SHA-256 file](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/download/v0.1.2-phase0-preview/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip.sha256). |
| **Understand the fleet** | Read the [public Phase0 source](https://github.com/Distributed-Minds/Fleet-Control-Public/tree/phase0/public-v0), [coordination model](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/Phase0/README.md), and [branding contract](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/BRANDING.md). |
| **Help an existing project** | Follow the [manual contribution guide and copyable local-agent prompt](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/HELP-A-PROJECT.md), or help with playtesting and documentation without a local agent. Check the project's AI-contribution policy and existing claims first; this does **not** enroll or assign you automatically. |
| **Make or remix a project** | Browse the [game-workflow guides](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/WORKFLOW-GUIDES.md) for approaches, tools and rights boundaries. These are references, **not** a shipped remastering service or playable catalog. |
| **Talk to the community** | Use [GitHub Discussions](https://github.com/Distributed-Minds/Fleet-Control-Public/discussions) to ask questions and express interest. This is a public contact point, **not** automatic agent enrollment or repository authorization. |

**Installing Phase0:** Follow the [current online beginner guide](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/GETTING-STARTED.md) and choose either **fork all branches**, then set **your fork's** default to `phase0/public-v0`, or **install the release ZIP** into a repository you control. The upstream repository's default `main` does **not** include the installable Phase0 starter; a default-branch-only fork is insufficient. Before enabling automations, verify `Phase0/05-FLEET-CONFIG.md` appears on **your repository's default branch**.

## The direction

The intended collaboration loop is:

```text
FIND / PLAY
     ↓
REPRODUCE / REPORT
     ↓
REPAIR / REMASTER
     ↓
TEST / VERIFY
     ↓
SHARE / REUSE
     ↺  the next contributor starts further ahead
```

A compatibility fix, working build, carefully licensed asset, or repeatable test should become a durable contribution—not disappear into another isolated conversation. The goal is **less duplicated work, more playable results**. This loop is a product direction, not a claim that every stage is already implemented.

## Contribute carefully

- **Use the existing project and its rules.** Search for upstream work, issues, and contribution/AI policies before editing or publishing.
- **Keep provenance and rights visible.** Original code, third-party game assets, source ports, and remixes can have different permissions. Publicly available is not automatically redistributable.
- **Verify before claiming success.** A code change, model assertion, or passing build does not alone prove a playable result.

**Privacy and security:** Issues and Discussions are public. Do not post credentials, private repository material, sensitive personal information, or exploit-enabling security details. A verified confidential reporting channel has not yet been documented; [requirements are tracked separately](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/38). That public issue is not a private disclosure channel.

The original public bootstrap material is [MIT-licensed](LICENSE), while individual third-party projects, assets, and sources retain their own rights.

---

<sub>FREE ENERGY is the public-facing project name; the repository slug still reflects its Fleet-Control origins. See the <a href="https://github.com/Distributed-Minds/Fleet-Control-Public/issues/57">public identity and website migration tracker</a>.</sub>
