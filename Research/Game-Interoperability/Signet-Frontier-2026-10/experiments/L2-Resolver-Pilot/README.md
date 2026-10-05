# L2 Resolver Pilot — source-anchored seed harness

This is an **unaffiliated research fixture** for the resolver experiment proposed by Signet 2.

It is **not yet the final 30-case benchmark**.

The first purpose is to freeze the evaluation mechanics before introducing CLM, Laya, an LLM, or other learned rankers.

## Seed cases

The seed contains six appearance mappings explicitly stated in the Signet 2 draft:

| Game | Archetype | Gold |
|---|---|---|
| Minecraft | `weapon.ranged` | `minecraft:bow` |
| Minecraft | `health_pickup` | `minecraft:golden_apple` |
| Doom | `weapon.ranged` | `doom:pistol` |
| Doom | `health_pickup` | `doom:medikit` |
| OpenArena | `weapon.ranged` | `openarena:machinegun` |
| OpenArena | `health_pickup` | `openarena:health_orb` |

The Minecraft ranged-weapon candidate set also comes directly from the Signet 2 example.

Some distractors for the other five seed cases are synthetic because the current Signet 2 document states the selected appearance but does not publish full palettes for those rows. `cases.json` records this explicitly so synthetic benchmark scaffolding cannot be mistaken for project facts.

## Perturbations

Every seed case is tested three ways:

1. **base** — the candidate list as stored;
2. **candidate-order rotations** — detect order-sensitive resolvers;
3. **NO_MATCH** — remove the correct candidate and require abstention.

This follows the L2 finding that a closed list prevents invention but does not guarantee that the correct answer is present.

## Current baselines

- `lookup` — explicit hand-written table; returns `__NO_MATCH__` if its known answer is absent;
- `lexical` — intentionally simple token-overlap ranker;
- `first` — deliberately bad control that always selects the first candidate.

## First result

```text
lookup:  base_accuracy=1.000 no_match=1.000 order_stability=1.000
lexical: base_accuracy=1.000 no_match=0.000 order_stability=0.167
first:   base_accuracy=1.000 no_match=0.000 order_stability=0.000
```

The important result is **not** that lexical achieved 100%.

It is that raw base accuracy is misleading here.

The Signet examples list the intended answer first, so a first-candidate baseline also gets 100%. Candidate-order stability and no-match behavior reveal the failure immediately.

Therefore the final Stage-A benchmark must not publish top-1 accuracy without at least:

- candidate-order perturbation;
- no-match/abstention cases;
- explicit candidate-set provenance.

## What this does not prove

- It does not compare CLM, Laya or an LLM yet.
- It does not show that six cases are representative.
- It does not claim synthetic distractors are canonical game palettes.
- It does not claim lookup is a fair learned-model baseline; it is the explicit-table reference.
- It does not replace Signet's proposed approximately 30-case experiment.

## Next expansion rule

Scale toward the 30-case pilot only with cases whose:

- gold mapping is source-backed or independently reviewed;
- candidate-set provenance is recorded;
- correct-answer absence can be represented explicitly;
- perturbations do not count as independent semantic cases.

That avoids inflating the benchmark by treating candidate-order rotations as new examples.
