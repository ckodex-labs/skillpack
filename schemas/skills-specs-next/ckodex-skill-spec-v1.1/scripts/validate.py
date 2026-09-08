#!/usr/bin/env python3
"""
ckodex-skill-spec validator. Stdlib + jsonschema only.

Usage:
  python3 scripts/validate.py path/to/skill.json [path/to/skill.json ...]

Exit code: 0 if all pass, non-zero if any fail.

Selects skill.v1.schema.json or skill.v1.1.schema.json based on the
manifest's apiVersion field. A v1 manifest under v1.1 validates as the
backward-compatibility case (no synopsis, no ontology, no runtime).
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

try:
    from jsonschema import Draft202012Validator
except ImportError:
    sys.stderr.write(
        "ERROR: jsonschema package required.\n"
        "  python3 -m pip install --user jsonschema\n"
    )
    sys.exit(2)


SCRIPT_DIR = Path(__file__).resolve().parent
SCHEMA_DIR = SCRIPT_DIR.parent / "schemas"

V1_PATH    = SCHEMA_DIR / "skill.v1.schema.json"
V1_1_PATH  = SCHEMA_DIR / "skill.v1.1.schema.json"


def load_json(p: Path) -> Any:
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def select_schema(manifest: dict) -> tuple[dict, str]:
    api = manifest.get("apiVersion", "")
    if api == "ckodex.org/skill/v1":
        return load_json(V1_PATH), "v1"
    elif api == "ckodex.org/skill/v1.1":
        return load_json(V1_1_PATH), "v1.1"
    elif api == "":
        return load_json(V1_1_PATH), "v1.1 (default; apiVersion missing)"
    else:
        raise ValueError(f"unknown apiVersion: {api!r}")


def validate_one(manifest_path: Path) -> tuple[bool, list[str]]:
    try:
        manifest = load_json(manifest_path)
    except json.JSONDecodeError as e:
        return False, [f"json parse error: {e}"]

    try:
        schema, label = select_schema(manifest)
    except ValueError as e:
        return False, [str(e)]

    validator = Draft202012Validator(schema)
    errors = sorted(validator.iter_errors(manifest), key=lambda e: list(e.absolute_path))

    if not errors:
        return True, [f"validated against {label}"]
    msgs = []
    for e in errors:
        path = "/".join(str(p) for p in e.absolute_path) or "(root)"
        msgs.append(f"{path}: {e.message}")
    return False, msgs


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 0

    total_fail = 0
    for arg in argv[1:]:
        p = Path(arg)
        if not p.exists():
            print(f"FAIL  {p}: file not found")
            total_fail += 1
            continue
        ok, msgs = validate_one(p)
        if ok:
            print(f"PASS  {p}: {msgs[0]}")
        else:
            print(f"FAIL  {p}")
            for m in msgs:
                print(f"  - {m}")
            total_fail += 1

    print()
    if total_fail == 0:
        print(f"0F / 0W — all {len(argv)-1} manifest(s) validated")
        return 0
    else:
        print(f"FAIL — {total_fail} of {len(argv)-1} manifest(s) failed")
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
