# Adapter role contract v1

This revises the first L2 shim **because the Minecraft gateway target falsified it**.

The v0 generated function set was calibration-oriented and did not cover the real translator responsibilities documented by Signet.

v1 therefore describes operations by translator role:

- HOST
- IMPORTER
- WORLD
- INPUT
- PRESENTATION
- AUTHORITY

The descriptor records the local mechanism and authority class, but leaves `semantic_contract_ref` external. L2 owns adapter/runtime shape; the shared meaning belongs to the semantic/profile layer.

The Minecraft example covers all seven responsibilities extracted from Signet's current gateway documentation. The generic Godot example claims only the engine host/version boundary; it does **not** pretend a bare GDExtension knows a particular game's controls, terrain or presentation hooks.

This is a descriptor/evidence model, not yet the final generated ABI.
