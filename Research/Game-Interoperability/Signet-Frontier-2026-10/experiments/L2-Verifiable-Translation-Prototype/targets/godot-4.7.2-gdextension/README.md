# Godot 4.7.2 GDExtension target proof

Unaffiliated L2 research fixture; not a Signet or Godot implementation.

This tests the previously generated C shim at a real native-engine ABI boundary.

Primary engine source:
- Godot `4.7.2-stable`, commit `ed1daf0bf001b61586d9930840f2f1394092c079`
- `core/extension/gdextension_interface.json` blob `9b55cc9810d458940acf7659dfac879abbcb8949`
- interface format version `1`

Godot documents that JSON as the low-level C-API source of truth and an input for generated language bindings.

`godot_abi_subset.h` contains only the exact 4.7.2 types needed by this proof. `godot_target.c` implements the generated L2 C shim and loads `get_godot_version2` through the normal procedure-address boundary.

Local build/harness result:

```text
godot-entry-init: PASS
godot-version-discovery: 0x040702
generated-shim-surface: PASS
```

The harness uses the real 4.7.2 ABI signatures but simulates the Godot host callback; this is **ABI_HOST_HARNESS_PASS**, not a claim that the library was loaded by the Godot executable.

Only `surface_version` is generically implementable. Input, motion observation and presentation still require project/game-specific Godot hooks. This is the desired architecture boundary, not hidden model behavior.
