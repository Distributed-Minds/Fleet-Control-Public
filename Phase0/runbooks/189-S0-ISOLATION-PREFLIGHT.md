# #189 / S0 — isolated agent-harness host: preflight and evidence packet

**Contract:** [issue #189](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/189), Phase0 specification **1**, S0 only.  
**Readiness:** [independent PLAN READY](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/189#issuecomment-6098215676) and [independent ADVERSARIAL READY](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/189#issuecomment-6098355206), **both limited to preparation of this non-destructive packet**.  
**Evidence at publication:** DESIGN / NOT_RUN. This Markdown is not a host inspection, a security attestation, or permission to provision a device.

## Operator-supplied host checkpoint (2026-10-10) — not S0 acceptance

The [canonical #189 checkpoint](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/189) records later **operator-provided terminal observations**. These supersede this packet's original *Windows/not-yet-installed* planning assumption, but **were not collected, repeated or validated by the GitHub agent**. They are not a coherent, timestamped network or guest-containment attestation.

| Surface | Operator-reported evidence | Current limitation |
| --- | --- | --- |
| Host operating system | A newly provisioned Linux build node is being administered; KVM/libvirt commands succeeded | Exact `/etc/os-release`, kernel and configuration generation still need a local capture |
| CPU/virtualization | Intel VT-x; `kvm_intel` / `kvm` loaded; `/dev/kvm` exists; `virt-host-validate qemu` passes core host checks | No disposable guest was booted or tested; `/dev/kvm` was observed as world-accessible and needs policy review |
| libvirt | `libvirtd` enabled/running; `qemu:///system` queried successfully; virtual `default` network active | No VMs listed; active default NAT is **not** guest egress isolation |
| Optional GPU | `nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv` reported **NVIDIA GeForce GTX 1050, 4096 MiB, 580.178.04** | Driver/device reporting only; CUDA, NVENC, remotely dispatched jobs and guest GPU access **NOT RUN** |
| IOMMU/VFIO | Host hardware DMAR available, but IOMMU disabled in kernel | No VFIO passthrough readiness claim; optional GPU work must not weaken VM isolation |

**Evidence progression:** The original S0 decision ledger below remains a snapshot of document creation (`NOT_RUN`); the host capability prerequisites now have *partial operator observations*. Each S0 security and integration verdict remains **NOT_RUN / BLOCKED pending the separately approved, versioned local negative/positive controls**, especially IPv4+IPv6 on both Wi-Fi and Ethernet, privileged socket/canary denial, guest teardown and fresh-harness execution. Do **not** reinstall Fedora/NVIDIA or edit host firewall/virtualization settings based solely on this report.

**Next trusted collection:** after separate local approval, record exact OS/kernel/driver/tool versions, `/dev/kvm` permissions, current network routes, effective nftables policy and immutable guest-image/configuration identity using the S0 evidence envelope below. Do not elevate older shell transcripts into a security PASS.


## 0. Boundaries and operator decisions

The trusted primary is the operator's Fedora desktop and approval surface. The secondary is the proposed Fedora Server hypervisor/limited dispatch appliance. An ephemeral KVM guest is a third, untrusted zone where an agent harness would eventually run. The physical Ethernet cable and a design diagram do not establish sandbox containment.

Before any local command, a trusted human/local operator must identify the real device, approve the exact information-collection scope and confirm no sensitive values are to be exported. **Stop immediately if the secondary remains Windows or has not been approved for Fedora installation:** this packet never changes disks, firmware, partition tables, firewall policy, routes, services, containers, guest images, credentials or removable media.

- **Permitted by this S0 planning packet:** prepare an inventory checklist, an evidence manifest, expected-result matrix and *suggested* read-only commands for a human to run later.
- **Not permitted by this packet:** unattended host access; Fedora installation; guest creation or destruction; firewall/bridge edits; attack probes; credential or private-key transfer; actual agent-harness migration; release/deployment; GitHub write authority from guest output.
- **Requires separately scoped local approval:** even read-only host execution and collection; privileged inventory commands such as `sudo nft list ruleset`; creation of fake canaries, network probes, guest launch/teardown and kill/negative tests. Do not assume GitHub account permissions grant device-level access.
- **Remains unsupported:** real #70/#50/#43/#91 dispatcher and provider-effect adapters, trustworthy external fence, public contributor enrollment, privileged guest-to-primary actions or automated default-branch merge.

## 1. Evidence envelope

Create one local, sanitized row per test observation. The report should reference evidence by a controlled artifact ID and digest, not paste secrets or raw private-network data into a public issue.

| Field | Required recording |
| --- | --- |
| `test_id` | Stable ID from the matrices below |
| `observed_at_utc` | Actual observation timestamp; never substitute document creation time |
| `executor` / `approval_ref` | Authorized human/local executor and exact scope of authorization |
| `machine_role` | `PRIMARY`, `SECONDARY` or `GUEST:<ephemeral-id>` |
| `subject_generation` | OS/kernel and package versions, network/firewall policy generation, VM image/overlay digest as applicable |
| `source` / `destination` / `address_family` | Path under test, including interface and IPv6 zone when relevant |
| `command_and_version` | Actual invoked command, tool version, start/end, exit code and sanitized output reference |
| `expected` / `observed` | Distinct fields; do not copy a fixture label as an observation |
| `status` | Exactly `NOT_RUN`, `PASS`, `FAIL`, or `BLOCKED` |
| `basis` / `gaps` | Host policy/counter evidence and reason evidence is complete or missing |

**Rules:** An absent executable, failed permission check, unspecified device, changed route/firewall policy or missing proof is `BLOCKED` / `NOT_RUN`, not `PASS`. Use `FAIL` for an observed violation. A packet can legitimately finish *preparation* with all test rows `NOT_RUN`. Actual guest containment is not established until the relevant rows are observed and reviewed.

## 2. Human-approved read-only inventory — separate local execution

Each command block below is a *proposed* human/local-harness command set. It was **not executed** by the GitHub operator. Operators must inspect commands, choose redaction rules, capture actual exit codes and record versions before running. Avoid running it verbatim if command, OS or interface assumptions do not match.

### PRIMARY — trusted desktop (inventory only)

```bash
date -u '+%Y-%m-%dT%H:%M:%SZ'
uname -srmo
ip -br link
ip -br addr
ip -4 route show table all
ip -6 route show table all
ip -4 rule show
ip -6 rule show
ss -lntup
nmcli -f GENERAL,IP4,IP6 connection show --active
# Privileged READ, subject to separate local approval:
# sudo nft list ruleset
```

Record the real Ethernet/Wi-Fi interfaces, routes to the secondary over both possible paths, address-family policy, effective inbound admin-port deny rules, and any local listeners. `ss`, `nmcli`, `ip` and `nft` output may expose other users/devices or private routing; sanitize locally. Do not export raw capture or credentials to this public repository.

### SECONDARY — prospective Fedora Server host (inventory only, after approved install)

```bash
date -u '+%Y-%m-%dT%H:%M:%SZ'
uname -srmo
lscpu
free -h
lsblk -o NAME,SIZE,TYPE,MOUNTPOINTS,TRAN
ip -br link
ip -br addr
ip -4 route show table all
ip -6 route show table all
ip -4 rule show
ip -6 rule show
ss -lntup
nmcli -f GENERAL,IP4,IP6 connection show --active
# Only if already installed, and approved for read-only inventory:
# virt-host-validate qemu
# virsh list --all
# sudo nft list ruleset
```

Validate /dev/kvm availability **without** installing or enabling KVM; distinguish firmware-disabled virtualization, absent hardware access, absent packages and inaccessible permissions. Record the exact hypervisor version and whether guests are even possible. KVM unavailable means `BLOCKED`, **not** automatic fallback to a shared-kernel/privileged container. No guest resources are inferred from host RAM totals alone.

### GUEST — only after a separately approved, isolated test guest exists

```bash
date -u '+%Y-%m-%dT%H:%M:%SZ'
uname -srmo
ip -br addr
ip -4 route show table all
ip -6 route show table all
ip -4 rule show
ip -6 rule show
ss -lntup
findmnt -r
env | cut -d= -f1 | sort
```

Only list environment **variable names**, never environment values or tokens. A guest must not inherit primary-home paths, SSH agent sockets, host runtime sockets, broad provider tokens or removable-storage passthrough. Inventory is not a penetration test and does not prove those paths are inaccessible.

## 3. Dual-path and adversarial test design — NOT AUTHORIZED TO EXECUTE BY THIS FILE

Do not run negative network/host probes before separate operator authorization, a disposable guest and nominated dummy targets exist. Avoid opportunistic port scanning of landlord/shared-LAN equipment, uncontrolled production admin ports or non-consenting hosts.

| ID | Attempt / control | Mandatory evidence | Initial status |
| --- | --- | --- | --- |
| N01 | Guest → PRIMARY via direct Ethernet, IPv4 | Guest route + host effective firewall rule/counter + approved nominated test endpoint | NOT_RUN |
| N02 | Guest → PRIMARY via secondary Wi-Fi, IPv4 | Actual Wi-Fi/LAN path, deny observation at both ends; do not assume isolated SSIDs | NOT_RUN |
| N03 | Guest → PRIMARY, IPv6 global and ULA where applicable | IPv6 route/rule/firewall and source/destination identity | NOT_RUN |
| N04 | Guest → PRIMARY, IPv6 link-local | Scoped interface/address (`%ifname`) and host-side deny corroboration | NOT_RUN |
| N05 | Guest → forbidden DNS/proxy/host-loopback/forwarded service path | Resolver and proxy identities; deny proof against nominated destination | NOT_RUN |
| N06 | Guest → secondary hypervisor-management interface | Guest path and hypervisor-side deny; no management socket inherited | NOT_RUN |
| N07 | Guest → narrowly allowlisted **unprivileged result receiver** (positive control) | Authenticated success that does not give shell/SSH/admin access | NOT_RUN |
| N08 | Guest-root → dummy host workspace/home/SSH-agent/privileged runtime socket | Dummy canary IDs; denied read/write/forward, mounts/devices/socket inventory | NOT_RUN |
| N09 | Guest-root → removable media, hypervisor device passthrough or sibling guest | Host and guest device/mount definition + dummy canary denial | NOT_RUN |
| N10 | Dispose test guest; clean-start another guest; attempt prior-canary recovery | Overlay/base image IDs, guest-termination and fresh-guest evidence, host-vs-guest retention | NOT_RUN |
| N11 | Reboot/restart lifecycle; check worker auto-resume or stale forwarding | Service and scheduler identity, killed old guest, no restored worker authority | NOT_RUN |
| N12 | Replay/tamper a fake approval or result; attempt untrusted output → host action | Trusted approval origin, generation/nonce/fence; no GitHub effect or host shell from result | NOT_RUN |

For **each denial** record both negative and appropriate positive controls. ICMP timeout alone does not demonstrate firewall enforcement. A listener not currently running does not demonstrate that a guest cannot reach it once running. If Wi-Fi/IPv6 routing or DNS changes during collection, invalidate the affected observations. Unknown network topology is `BLOCKED`, never "safe by default."

## 4. S0 nine-step decision ledger

Maintain this table in a later **sanitized, dated execution report**. The values below intentionally record *no local work performed*.

| Step from #189 | What must be evidenced | Status at document creation |
| --- | --- | --- |
| S0-1 Human/device boundary | Device model/ownership, install/backup decision, consent and allowed scope | NOT_RUN |
| S0-2 KVM capacity | CPU flags, /dev/kvm, guest feasibility, hypervisor, RAM/SSD and pass-through review | NOT_RUN |
| S0-3 Two-machine network inventory | Both NICs, IPv4/IPv6, DNS, listeners, NAT/forwarding and effective policy | NOT_RUN |
| S0-4 Network negative/positive matrix | N01–N07 with dual-end evidence and an unprivileged result path | NOT_RUN |
| S0-5 Canaries and credentials | N08–N09; *only fake canaries*, blocked sockets/devices/host mounts | NOT_RUN |
| S0-6 Disposable teardown | N10–N11; fresh image, old state inaccessible, no restart resurrection | NOT_RUN |
| S0-7 Real harness | Identified harness PID/process tree in disposable guest, narrow tools, kill and result provenance | NOT_RUN |
| S0-8 Assignment/effects | Human-governed approval; trusted services missing ⇒ `BLOCKED_AUTHORITY` or `UNSUPPORTED_EFFECT` | NOT_RUN |
| S0-9 Rollback | Verified abort path, scoped credential revocation and sanitized incident preservation | NOT_RUN |

**S0 report disposition:** `NOT_RUN` until a human/local executor performs and records evidence. The report may name precise blockers and next approvals without declaring installation or containment successful. **S3 harness migration** additionally requires execution of a real agent harness *on the guest*, not merely remote Rust builds.

## 5. Integrity, authorization and stop/recovery procedure

1. Record a deliberate source/subject pair and current test-policy generation before collecting evidence. Do not join unrelated host captures into a seemingly coherent simultaneous proof.
2. Do not store passkeys, real tokens, SSH private keys, raw agent sockets, private browser profiles, human home directories or removable media in guest tests. Use fake per-test canaries, narrowly authorized destinations and sanitized reports.
3. If a forbidden route, mount, device, privileged socket or protected service is reachable, record `FAIL`, cease the pilot, preserve sanitized evidence, revoke any scoped *test* credentials and ask the human to review containment. No automatic firewall change or destructive cleanup is authorized here.
4. On KVM, network or local permission uncertainty, record `BLOCKED` and identify the exact human decision, host privilege or current-fact gap. Do not convert `UNKNOWN` to `PASS` based on a diagram, prior session or historical CI.
5. After any device, route, Wi-Fi association, policy/rule, hypervisor image or credential-generation change, invalidate the relevant recorded proofs and recheck under the new basis.
6. Keep guest output as untrusted evidence; a generated command/result never grants authorization. Only the human-controlled default-branch process can promote public Phase0 code to `main`.

## 6. Handoff checklist

- [ ] Human/local executor confirms the *physical* secondary and the allowed information-collection scope.
- [ ] Host inventory is gathered separately on primary and secondary, redacted and timestamped.
- [ ] Installation and guest-testing approvals, if any, are independent of this report.
- [ ] Every N01–N12 row is `NOT_RUN`, `PASS`, `FAIL` or `BLOCKED` with actual evidence links and current policy basis.
- [ ] Real-harness migration remains gated until disposable VM, network, storage, approval, teardown and emergency-stop controls are all observed.
- [ ] Issue #189 stays open. The wider Rust orchestration and provider-effect work in #70/#50/#43/#91 remains independently gated.

**This file ships only an auditable preparation artifact.** The host has not been inspected, deployed, tested, certified or modified by this GitHub operator.
