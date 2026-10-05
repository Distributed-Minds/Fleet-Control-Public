# Minecraft controlled-server gateway transport proof

Unaffiliated L2 research fixture based on Signet's documented beta Minecraft Java gateway.

Observed Signet source: `kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2`.

Primary pages:
- `docs/content/guides/gateway.mdx` blob `00dd8db68b6bb123a6c3295ce822ea505e388bcf`
- `docs/content/concepts/translators.mdx` blob `6f537cffac65da1287106e2fdcf222aa0e3348a5`

They document a local Minecraft server controlled over RCON using mechanisms including `/fill`, `/data get entity`, `/tp`, `/damage`, `/kill`, and statistic scoreboards.

The public repository does not expose the gateway implementation source in the observed tree, so this fixture does **not** claim to reconstruct it.

RCON framing is cross-checked against `gorcon/rcon` `packet.go` blob `168527b1866a0e0ca19558c1df46edcba18e877a` (MIT).

Executed local result:

```text
source-rcon-framing: PASS
commands-roundtripped: 5
v0-shim-coverage: NO=5, PARTIAL=2
v0-shim-sufficiency: FAIL_EXPECTED
```

The transport proof passes. The important negative result is that the first generated L2 shim cannot cleanly express the documented gateway responsibilities. Deterministic generation therefore survives the test, but the v0 **logical contract does not**.

The next ABI should be derived from real translator responsibilities, not expanded ad hoc to make this test green.
