#!/usr/bin/env python3
"""
scaffold.py — create a fresh CKODEX v1.1 skill tree from a blank slate.

Usage:
  python3 scaffold.py <target-dir> \\
      --name <skill-name> \\
      --description <one-line> \\
      [--license MIT|Apache-2.0|...] \\
      [--author <name>] \\
      [--version 0.1.0] \\
      [--synopsis <bounded-text>] \\
      [--init] \\
      [--apply | --write] \\
      [--force]

Defaults to dry-run. Add --apply (or --write) to actually create files.
Use --init to create the target directory if it does not exist.
Use --force to overwrite an existing SKILL.md.

Exit code: 0 on success, non-zero on validation or conflict failure.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from pathlib import Path
from string import Template
from typing import Any

TOOLS_VERSION = "0.1.0"

# Validation rules — must match skill.v1.1.schema.json
NAME_RE = re.compile(r"^(?!-)(?!.*--)[a-z0-9]+(?:-[a-z0-9]+)*$")
NAME_MAX = 64
DESC_MIN, DESC_MAX = 1, 1024
SYN_MIN, SYN_MAX = 1, 2048

# Template locations — relative to this script
TPL_DIR = Path(__file__).resolve().parent.parent / "assets" / "templates"
TPL_SKILL_MD   = TPL_DIR / "SKILL.md.tmpl"
TPL_SKILL_JSON = TPL_DIR / "skill.json.tmpl"
TPL_USAGE_MD   = TPL_DIR / "usage.md.tmpl"
TPL_VALIDATE   = TPL_DIR / "validate.sh.tmpl"


def die(msg: str, code: int = 1) -> None:
    sys.stderr.write(f"ERROR: {msg}\n")
    sys.exit(code)


def validate_name(name: str) -> None:
    if len(name) < 1 or len(name) > NAME_MAX:
        die(f"name length {len(name)} not in [1,{NAME_MAX}]")
    if not NAME_RE.match(name):
        die(f"name {name!r} violates regex (lowercase alnum + hyphens, no leading/trailing or consecutive hyphens)")


def validate_description(desc: str) -> None:
    if len(desc) < DESC_MIN or len(desc) > DESC_MAX:
        die(f"description length {len(desc)} not in [{DESC_MIN},{DESC_MAX}]")


def validate_synopsis(syn: str) -> None:
    if syn and (len(syn) < SYN_MIN or len(syn) > SYN_MAX):
        die(f"synopsis length {len(syn)} not in [{SYN_MIN},{SYN_MAX}]")


def render_template(tpl_path: Path, **vars: str) -> str:
    text = tpl_path.read_text(encoding="utf-8")
    return Template(text).safe_substitute(**vars)


def synopsis_default(name: str, description: str) -> str:
    """A minimal synopsis the author should edit. Bounded at SYN_MAX."""
    s = (f"When the user mentions {name} or related topics: this skill {description.rstrip('.')}. "
         f"Triggers: explicit mention of {name}; topical keywords from the description. "
         f"Inputs and outputs: TODO — fill in after authoring SKILL.md body. "
         f"NOT a substitute for general-purpose tools; refuses out-of-scope requests.")
    return s[:SYN_MAX]


def build_skill_md(name: str, description: str, license: str) -> str:
    return render_template(TPL_SKILL_MD,
                           name=name, description=description, license=license)


def build_skill_json(*, name: str, description: str, synopsis: str,
                     version: str, license: str, author: str) -> str:
    # Render with $-substitution then parse + re-serialize for canonical JSON
    raw = render_template(TPL_SKILL_JSON,
                          name=name, description=description, synopsis=synopsis,
                          version=version, license=license, author=author)
    parsed: Any = json.loads(raw)
    return json.dumps(parsed, indent=2, ensure_ascii=False) + "\n"


def build_usage_md(name: str) -> str:
    return render_template(TPL_USAGE_MD, name=name)


def build_validate_sh(name: str) -> str:
    return render_template(TPL_VALIDATE, name=name, tools_version=TOOLS_VERSION)


def build_manifest(root: Path) -> str:
    """sha256 ledger of every file in root (excluding MANIFEST.json itself)."""
    files = []
    EXCLUDE_DIRS = {"__pycache__", ".pytest_cache", ".mypy_cache"}
    EXCLUDE_SUFFIXES = {".pyc", ".pyo"}
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        if p.name == "MANIFEST.json":
            continue
        if p.suffix in EXCLUDE_SUFFIXES:
            continue
        if any(part in EXCLUDE_DIRS for part in p.parts):
            continue
        digest = "sha256:" + hashlib.sha256(p.read_bytes()).hexdigest()
        files.append({
            "path": str(p.relative_to(root)).replace(os.sep, "/"),
            "sha256": digest,
            "bytes": p.stat().st_size,
        })
    manifest = {
        "schemaVersion": "ckodex.org/pack-manifest/v1",
        "pack": root.name,
        "framework": "CKODEX v16.0",
        "generatedBy": f"ckodex-skill-tools/scaffold.py {TOOLS_VERSION}",
        "files": files,
        "fileCount": len(files),
    }
    return json.dumps(manifest, indent=2) + "\n"


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(
        description="Scaffold a fresh CKODEX v1.1 skill tree.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument("target", type=Path, help="Target directory for the new skill")
    p.add_argument("--name", required=True, help="Skill name (lowercase alnum + hyphens, ≤64 chars)")
    p.add_argument("--description", required=True, help="One-line description (1–1024 chars)")
    p.add_argument("--license", default="Apache-2.0")
    p.add_argument("--author", default="unknown")
    p.add_argument("--version", default="0.1.0")
    p.add_argument("--synopsis", default=None, help="Synopsis text (≤2048 chars). Auto-generated if omitted.")
    p.add_argument("--init", action="store_true", help="Create target dir if it does not exist")
    p.add_argument("--apply", "--write", dest="apply", action="store_true",
                   help="Actually write files (default: dry-run)")
    p.add_argument("--force", action="store_true",
                   help="Overwrite existing files in target")
    args = p.parse_args(argv[1:])

    # Validate inputs
    validate_name(args.name)
    validate_description(args.description)
    syn = args.synopsis if args.synopsis is not None else synopsis_default(args.name, args.description)
    validate_synopsis(syn)

    target = args.target.resolve()

    # Target-directory checks
    if not target.exists():
        if not args.init:
            die(f"{target} does not exist (pass --init to create)")
        if args.apply:
            target.mkdir(parents=True)
    elif not target.is_dir():
        die(f"{target} exists but is not a directory")

    # Conflict check
    skill_md_path = target / "SKILL.md"
    if skill_md_path.exists() and not args.force:
        die(f"{skill_md_path} already exists; pass --force to overwrite or use migrate.py")

    # Plan the output set
    plan = {
        "SKILL.md":             build_skill_md(args.name, args.description, args.license),
        "skill.json":           build_skill_json(
                                    name=args.name, description=args.description,
                                    synopsis=syn, version=args.version,
                                    license=args.license, author=args.author),
        "references/usage.md":  build_usage_md(args.name),
        "scripts/validate.sh":  build_validate_sh(args.name),
    }

    print(f"Target:      {target}")
    print(f"Skill name:  {args.name}")
    print(f"Mode:        {'APPLY (writing files)' if args.apply else 'DRY-RUN (no files written)'}")
    print()
    print("Planned outputs:")
    for rel, content in plan.items():
        size = len(content.encode("utf-8"))
        print(f"  {rel:<28} {size:>6} bytes")
    print(f"  {'MANIFEST.json':<28} (generated after files are written)")

    if not args.apply:
        print()
        print("Dry-run complete. Pass --apply to write these files.")
        return 0

    # Apply
    for rel, content in plan.items():
        out = target / rel
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(content, encoding="utf-8")
        # chmod +x on shell scripts
        if rel.endswith(".sh"):
            out.chmod(0o755)

    # Build MANIFEST.json last (after all files are on disk)
    manifest_json = build_manifest(target)
    (target / "MANIFEST.json").write_text(manifest_json, encoding="utf-8")

    print()
    print(f"OK — wrote {len(plan) + 1} files to {target}")
    print(f"Verify with:  cd {target} && bash scripts/validate.sh")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
