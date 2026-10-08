# L2 third-party source and rights provenance

**Scope:** corrected Signet L2 research **archive**, specifically the 55 source files staged by [PR #78](https://github.com/Distributed-Minds/Fleet-Control-Public/pull/78) from `correction/l2-role-contract-reproducibility@ba65b131559a28cf0cb7d3416d492dbd1d91b316`. This file is a supplemental ledger, **not** a change to the original corrected 55 blobs or their historical test logs.

**Disposition: RIGHTS_PENDING / archival only.** Referencing an engine or protocol does not imply its authors endorsed this project, grant rights to third-party game content, establish authorship of the research files, or make the archived scripts supported FREE ENERGY tools. The public repository's root MIT license does not silently relicense upstream material. An independent contents/similarity and applicable-rights review remains required before claiming broader reuse clearance.

## Pinned external evidence and classification

| Upstream family | Exact source and upstream rights | Relationship observed in this research archive | Current disposition |
| --- | --- | --- | --- |
| **Godot Engine** | [`godotengine/godot@ed1daf0bf001b61586d9930840f2f1394092c079`](https://github.com/godotengine/godot/tree/ed1daf0bf001b61586d9930840f2f1394092c079), tag `4.7.2-stable`; [`core/extension/gdextension_interface.json`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.json), Git blob `9b55cc9810d458940acf7659dfac879abbcb8949`; [`LICENSE.txt`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/LICENSE.txt), Git blob `0e3ba08d6b2e8cf435241829c96f10b74e4356fe` (**MIT**) | `L2-Verifiable-Translation-Prototype/targets/godot-4.7.2-gdextension/godot_abi_subset.h`, Git blob `79ffa3393a242b09b3dcf98b8cc2a7698d55f0c5`, reproduces the needed interface typedef/enum shapes. The sibling README attributes the pinned Godot interface. Its C host harness is a simulation, **not** a live Godot load. | The full upstream MIT notice is reproduced **below for this identified Godot-derived interface surface only**. Whether this subset constitutes legally substantial copied material remains unadjudicated. |
| **Signet Protocol** | Historical `kian-cx/signetprotocol` alias; resolved canonical repository [`signetprotocol/signet@2ddb136ee941705d3e1c020eaddad93be65026f2`](https://github.com/signetprotocol/signet/tree/2ddb136ee941705d3e1c020eaddad93be65026f2); [`LICENSE`](https://github.com/signetprotocol/signet/blob/2ddb136ee941705d3e1c020eaddad93be65026f2/LICENSE), Git blob `d645695673349e3947e8e5ae42332d0ac3164cd7` (**Apache-2.0**); [`NOTICE`](https://github.com/signetprotocol/signet/blob/2ddb136ee941705d3e1c020eaddad93be65026f2/NOTICE), Git blob `e3439c9785f22aba6042205eee7ae6660d5792f0` | `targets/minecraft-rcon-gateway/README.md` pins gateway and translator documentation, and the L2 examples discuss interface/translator responsibilities. No third-party Signet executable was proved copied by the bounded source review. | Technical citation, **not** whole-archive Apache-2.0 relicensing. If a substantial adapted/copied work is later identified, separately determine required Apache-2.0 license, changes and NOTICE obligations. |
| **gorcon/rcon** | [`gorcon/rcon@ee38ab9793a296a003d36002ee279df1c1cf06b6`](https://github.com/gorcon/rcon/tree/ee38ab9793a296a003d36002ee279df1c1cf06b6), `packet.go` blob `168527b1866a0e0ca19558c1df46edcba18e877a`; [`LICENSE`](https://github.com/gorcon/rcon/blob/ee38ab9793a296a003d36002ee279df1c1cf06b6/LICENSE), Git blob `a5db1e2b7f9220848f832b4d4e5cac9e1836a799` (**MIT; copyright 2020 Pavel Korotkiy / outdead**) | `targets/minecraft-rcon-gateway/rcon_frame.py` (`d2d1b250d66583e08e63cd1b7fbfba2e95a85697`) refers to framing/protocol behavior; prior source inspection did **not** identify a verbatim copy of upstream `packet.go`. | Reference for protocol verification, not a verified code-copy/license-transfer determination. Preserve its notice if copying/substantial adaptation is later established. |

All pinned Git blob identifiers above are **Git object IDs**, not evidence of runtime correctness or license compliance. File comparisons, artifact generation claims, and previously recorded test results are independent of rights approval.

## Godot MIT license notice for the extracted API surface

The following notice is reproduced from the **pinned Godot `LICENSE.txt` above**, to retain the upstream author's licensing information with the `godot_abi_subset.h` archival fixture. It does **not** grant a new license over unrelated L2 files, third-party games, or the overall FREE ENERGY product.

```text
Copyright (c) 2014-present Godot Engine contributors (see AUTHORS.md).
Copyright (c) 2007-2014 Juan Linietsky, Ariel Manzur.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Generated data and historical runtime claims

- `adapter-shim-generation/adapter-shim.v0.example.json` and `adapter-role-contract-v1/` contain experiment-authored descriptors, generator scripts, and C/Rust/C#/TypeScript outputs. Their READMEs record historical generator hashes and local results. **Those records do not establish original authorship of every underlying API type, new execution success, or permission to relicense external material.**
- `adapter-role-contract-v1/historical/` deliberately preserves previously broken source and generated-output snapshots. In particular, its malformed historical JSON schema is **not** a currently valid schema or supported executable baseline.
- Minecraft and Godot names identify compatibility targets; no original game content, game rights, or vendor endorsement is claimed by these fixtures. No live Minecraft server or live Godot engine use is established by this ledger.
- Python, JavaScript and historical generated TypeScript under this archive remain **inert evidence**, not maintained FREE ENERGY build, validation or installation dependencies. Issue [#70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70) requires compiled Rust and no npm for maintained tooling.

## Remaining review before wider use

1. Inspect each relevant archival artifact for actual verbatim/derived protected source and authorship, not just names, typedef shape, or file extensions. Determine whether any additional notice, attribution or license text must accompany a substantially copied component.
2. Confirm game titles, trademarks, credentials, server terms and content rights are handled separately; nothing here authorizes redistribution of proprietary game data.
3. Preserve the original 55 corrected L2 Git blob IDs and historical evidence. If a future executable is promoted out of the archive, implement and test the maintained Rust/non-npm path independently, and re-run relevant conformance suites.
4. Obtain authorized review for non-default research integration of PR #78; do not infer website/default-branch publication, deployment, or release authorization.

**Status: PROVENANCE RECORDED; RIGHTS CLEARANCE, EXTERNAL GAME VALIDATION, RUST CONFORMANCE AND CI REMAIN UNVERIFIED.**
