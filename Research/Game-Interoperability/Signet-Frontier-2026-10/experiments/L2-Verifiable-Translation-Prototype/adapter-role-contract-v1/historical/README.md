# Historical (pre-correction) evidence — DO NOT REUSE

These files are frozen copies of the L2 role-contract evidence as committed at the
historical L2 head:

- branch: `research/signet-l2-adapters-ai` (the original research lane)
- head: `a5a677f7bf3e80d2face8f2185b27fcb8e394ab3`
- correction branch: `correction/l2-role-contract-reproducibility`

They are retained only to make the defect auditable. The original branch/head is
**not** rewritten. Do not cite these as current evidence.

## Why the pre-correction evidence failed

| File | Defect | Git blob at historical head |
|---|---|---|
| `role-contract.schema.json` | malformed JSON: one extra `}` (line 1, col 1267) | `13edcbaf242e3b4f217e9a4ea75ad6c63f80d577` |
| `generated/minecraft/adapter.operations.json` | does not match canonical generator output/digest manifest | `57db4db8666423d6a9b6aed1a48ebc4a28fd37f2` |
| `generated/minecraft/signet_adapter_role_ops.h` | does not match canonical generator output/digest manifest | `c53ddb5c36a266cf34b931fec1608a877236190b` |
| `generated/godot/adapter.operations.json` | does not match canonical generator output/digest manifest | `fdcae3e4f0703da6ebfe37df3f9c860d03f5c608` |
| `generated/godot/signet_adapter_role_ops.h` | does not match canonical generator output/digest manifest | `322ce2d30127250a35d3330b56bd7c75d7d98d00` |
| `generation-validation-output.txt` | digest manifest was correct but unverifiable while the schema was unparseable | `1eb0d61b6413b537fa374d8cdda858710a06c930` |
| `validation-output.txt` | claimed 7/7 Minecraft coverage; not reproducible because `check.py` died at `json.loads` | `42aa2598f974f1c91fe851102af93c5815d5cd71` |

`generated/*/signet_adapter_role_ops.rs` matched the manifest and is unaffected.

The corrected artifacts live one directory up and are reproduced by
`run_generation_validation.py` and `check.py`. See `CORRECTION-NOTES.md`.
