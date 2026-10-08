#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
schema = json.loads((ROOT / "schemas/typed-translation-profile.v0.schema.json").read_text(encoding="utf-8"))
profile = json.loads((ROOT / "examples/typed-profile.example.json").read_text(encoding="utf-8"))
errors = sorted(Draft202012Validator(schema).iter_errors(profile), key=lambda e: list(e.absolute_path))

if errors:
    print("typed profile: FAIL")
    for err in errors:
        where = "/".join(str(p) for p in err.absolute_path) or "<root>"
        print(f"{where}: {err.message}")
    raise SystemExit(1)

print("typed profile: PASS")
