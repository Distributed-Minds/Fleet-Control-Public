#!/usr/bin/env python3
import json
from pathlib import Path
from jsonschema import Draft202012Validator
ROOT=Path(__file__).resolve().parent
schema=json.loads((ROOT/"role-contract.schema.json").read_text())
def load(name):
 d=json.loads((ROOT/name).read_text());e=list(Draft202012Validator(schema).iter_errors(d))
 if e: raise SystemExit(f"{name}: schema FAIL: {e[0].message}")
 print(f"{name}: schema PASS");return d
mc=load("minecraft.example.json");godot=load("godot.example.json")
req=json.loads((ROOT/"minecraft-requirements.json").read_text())
ops={x["id"] for x in mc["operations"]};required={x["id"] for x in req["requirements"]};missing=required-ops
print(f"minecraft-responsibility-coverage: {len(required)-len(missing)}/{len(required)}")
if missing: raise SystemExit("missing: "+",".join(sorted(missing)))
print("minecraft-role-contract: PASS")
assert not [x for x in godot["operations"] if x["role"]!="HOST"]
print("godot-generic-gameplay-claims: 0")
print("godot-engine-vs-game-boundary: PASS")
