#!/usr/bin/env python3
"""
migrate.py — upgrade an existing Agent Skills v1 skill to CKODEX v1.1 in place.

Usage:
  python3 migrate.py <target-dir> [--apply | --write] [--force]

Defaults to dry-run. Add --apply to actually modify files. Use --force to
overwrite author-authored synopsis/ontology/runtime blocks that already exist.

What it does:
  1. Parses SKILL.md frontmatter (required: name, description).
  2. Loads existing skill.json if present (treated as the source of truth for fields it carries).
  3. Composes a bounded synopsis (≤2048 chars) from the SKILL.md body.
  4. Bumps apiVersion to ckodex.org/skill/v1.1.
  5. Adds empty stub blocks for ontology and runtime, marked with
     MIGRATION: comments where the v1.1 schema accepts annotations.
  6. Writes MANIFEST.json sha256 ledger.
  7. Writes scripts/validate.sh if absent.
  8. NEVER overwrites author-authored synopsis/ontology/runtime unless --force.

Exit code: 0 on success, non-zero on parse/validation/conflict failure.
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

TOOLS_VERSION = "0.1.0"

NAME_RE = re.compile(r"^(?!-)(?!.*--)[a-z0-9]+(?:-[a-z0-9]+)*$")
NAME_MAX = 64
DESC_MIN, DESC_MAX = 1, 1024
SYN_MAX = 2048
SYN_TARGET = 1500   # comfortable target so a hand-edit fits

TPL_DIR = Path(__file__).resolve().parent.parent / "assets" / "templates"
TPL_VALIDATE = TPL_DIR / "validate.sh.tmpl"


def die(msg: str, code: int = 1) -> None:
    sys.stderr.write(f"ERROR: {msg}\n")
    sys.exit(code)


# ───────────────────────────────────────────────────────────────
# Frontmatter parse — minimal YAML subset, no external deps
# ───────────────────────────────────────────────────────────────

def parse_frontmatter(md_text: str) -> dict[str, str]:
    """Parse the YAML frontmatter at the top of a SKILL.md file.

    Supports flat key: value pairs. Does NOT support nested mappings or
    block scalars — Agent Skills v1 frontmatter is documented as flat.
    """
    m = re.match(r"^---\s*\n(.*?)\n---\s*\n", md_text, re.DOTALL)
    if not m:
        return {}
    fm: dict[str, str] = {}
    for line in m.group(1).splitlines():
        line = line.rstrip()
        if not line or line.startswith("#"):
            continue
        if ":" not in line:
            continue
        k, v = line.split(":", 1)
        fm[k.strip()] = v.strip().strip('"\'')
    return fm


def extract_body(md_text: str) -> str:
    """Strip frontmatter; return the markdown body."""
    return re.sub(r"^---\s*\n.*?\n---\s*\n", "", md_text, count=1, flags=re.DOTALL)


def compose_synopsis(body: str, name: str, description: str) -> str:
    """Build a bounded synopsis by linearizing the body."""
    # Strip code fences
    body = re.sub(r"```.*?```", " ", body, flags=re.DOTALL)
    # Strip markdown headers but keep their text
    body = re.sub(r"^#{1,6}\s*", "", body, flags=re.MULTILINE)
    # Collapse whitespace
    body = re.sub(r"\s+", " ", body).strip()

    if not body:
        # Fallback: synthesize from description
        s = (f"When the user mentions {name} or related topics: "
             f"{description.rstrip('.')}. "
             f"Triggers: explicit mention of {name}. "
             f"Inputs/outputs: see SKILL.md. NOT a substitute for general tools.")
        return s[:SYN_MAX]

    head = body[:SYN_TARGET]
    tail = "" if len(body) <= SYN_TARGET else " ...[truncated; hand-edit recommended]"
    syn = head + tail
    if len(syn) > SYN_MAX:
        syn = syn[: SYN_MAX - 5] + "...[T]"
    return syn


# ───────────────────────────────────────────────────────────────
# Validation
# ───────────────────────────────────────────────────────────────

def validate_inputs(name: str, description: str) -> None:
    if not (1 <= len(name) <= NAME_MAX) or not NAME_RE.match(name):
        die(f"name {name!r} fails v1 regex or length limit")
    if not (DESC_MIN <= len(description) <= DESC_MAX):
        die(f"description length {len(description)} not in [{DESC_MIN},{DESC_MAX}]")


# ───────────────────────────────────────────────────────────────
# Stub blocks (added with MIGRATION: annotation comments-as-keywords)
# ───────────────────────────────────────────────────────────────

def stub_ontology() -> dict:
    return {
        "@context": "https://ckodex.org/skill-context/v1",
        "subjects":    [],
        "produces":    [],
        "consumes":    [],
        "related":     [],
        "exclusiveOf": [],
        "triggers":    [],
        "galMinimum":  1,
        "proofTypes":  [],
    }


def stub_runtime() -> dict:
    return {
        "implicitInvocation": True,
        "minTier": "L1",
        "maxTier": "L3",
        "l4xAllowed": False,
    }


# ───────────────────────────────────────────────────────────────
# Migration core
# ───────────────────────────────────────────────────────────────

def migrate_manifest(
    existing: dict,
    *,
    fm_name: str,
    fm_description: str,
    fm_license: str | None,
    synopsis: str,
    force: bool,
) -> tuple[dict, list[str]]:
    """Return (new_manifest, notes). notes are human-readable change descriptions."""
    notes: list[str] = []
    m = dict(existing) if existing else {}

    # 1. apiVersion
    prev_api = m.get("apiVersion", "")
    m["apiVersion"] = "ckodex.org/skill/v1.1"
    if prev_api != m["apiVersion"]:
        notes.append(f"apiVersion: {prev_api or '(absent)'} → ckodex.org/skill/v1.1")

    # 2. kind
    if m.get("kind") != "Skill":
        m["kind"] = "Skill"
        notes.append("kind: set to 'Skill'")

    # 3. metadata
    md = dict(m.get("metadata") or {})
    if md.get("name") != fm_name:
        md["name"] = fm_name
        notes.append(f"metadata.name ← SKILL.md frontmatter ({fm_name})")
    if md.get("description") != fm_description:
        md["description"] = fm_description
        notes.append("metadata.description ← SKILL.md frontmatter")
    if fm_license and not md.get("license"):
        md["license"] = fm_license
        notes.append(f"metadata.license ← SKILL.md frontmatter ({fm_license})")

    if "synopsis" in md and not force:
        notes.append("metadata.synopsis: PRESERVED (use --force to overwrite)")
    else:
        if "synopsis" in md and force:
            notes.append("metadata.synopsis: OVERWRITTEN (forced)")
        else:
            notes.append(f"metadata.synopsis: ADDED ({len(synopsis)} chars; HAND-EDIT recommended)")
        md["synopsis"] = synopsis

    m["metadata"] = md

    # 4. entrypoints
    ep = dict(m.get("entrypoints") or {})
    if ep.get("skillMd") != "SKILL.md":
        ep["skillMd"] = "SKILL.md"
        notes.append("entrypoints.skillMd: set to 'SKILL.md'")
    m["entrypoints"] = ep

    # 5. ontology stub
    if "ontology" in m and not force:
        notes.append("ontology: PRESERVED (use --force to overwrite with stub)")
    else:
        if "ontology" in m and force:
            notes.append("ontology: OVERWRITTEN with stub (forced)")
        else:
            notes.append("ontology: ADDED stub (HAND-EDIT to fill subjects/produces/consumes/triggers)")
        m["ontology"] = stub_ontology()

    # 6. runtime stub
    if "runtime" in m and not force:
        notes.append("runtime: PRESERVED (use --force to overwrite with stub)")
    else:
        if "runtime" in m and force:
            notes.append("runtime: OVERWRITTEN with stub (forced)")
        else:
            notes.append("runtime: ADDED stub (HAND-EDIT minTier/maxTier/l4xAllowed for your skill)")
        m["runtime"] = stub_runtime()

    return m, notes


def build_manifest_ledger(root: Path) -> str:
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
        "generatedBy": f"ckodex-skill-tools/migrate.py {TOOLS_VERSION}",
        "files": files,
        "fileCount": len(files),
    }
    return json.dumps(manifest, indent=2) + "\n"


def build_validate_sh(name: str) -> str:
    text = TPL_VALIDATE.read_text(encoding="utf-8")
    return Template(text).safe_substitute(name=name, tools_version=TOOLS_VERSION)


# ───────────────────────────────────────────────────────────────
# Main
# ───────────────────────────────────────────────────────────────

def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(
        description="Migrate an Agent Skills v1 skill to CKODEX v1.1 in place.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument("target", type=Path, help="Directory containing the existing skill")
    p.add_argument("--apply", "--write", dest="apply", action="store_true",
                   help="Actually write changes (default: dry-run)")
    p.add_argument("--force", action="store_true",
                   help="Overwrite author-authored synopsis/ontology/runtime blocks")
    args = p.parse_args(argv[1:])

    target = args.target.resolve()
    if not target.is_dir():
        die(f"{target} is not a directory")

    skill_md_path = target / "SKILL.md"
    if not skill_md_path.exists():
        die(f"{skill_md_path} does not exist")

    md_text = skill_md_path.read_text(encoding="utf-8")
    fm = parse_frontmatter(md_text)
    if not fm:
        die(f"{skill_md_path} has no YAML frontmatter")
    fm_name = fm.get("name", "")
    fm_desc = fm.get("description", "")
    fm_lic  = fm.get("license")

    validate_inputs(fm_name, fm_desc)

    # Load existing skill.json if present
    skill_json_path = target / "skill.json"
    existing: dict = {}
    if skill_json_path.exists():
        try:
            existing = json.loads(skill_json_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            if not args.force:
                die(f"{skill_json_path} is unparseable JSON ({e}); pass --force to overwrite")
            print(f"  WARN  existing skill.json unparseable; will overwrite (forced)")

    # Build synopsis from SKILL.md body
    body = extract_body(md_text)
    syn = compose_synopsis(body, fm_name, fm_desc)

    # Run migration
    new_manifest, notes = migrate_manifest(
        existing,
        fm_name=fm_name,
        fm_description=fm_desc,
        fm_license=fm_lic,
        synopsis=syn,
        force=args.force,
    )

    # Decide on validator.sh
    validate_sh_path = target / "scripts" / "validate.sh"
    add_validate = not validate_sh_path.exists()

    # Decide on references/usage.md (validate.sh §1 requires it)
    usage_md_path = target / "references" / "usage.md"
    add_usage = not usage_md_path.exists()

    # Summary
    print(f"Target:    {target}")
    print(f"Mode:      {'APPLY (writing files)' if args.apply else 'DRY-RUN (no files written)'}")
    print(f"Force:     {args.force}")
    print()
    print(f"Skill identity:")
    print(f"  name:        {fm_name}")
    print(f"  description: {fm_desc[:80]}{'...' if len(fm_desc)>80 else ''}")
    print(f"  synopsis:    {len(syn)} chars (composed from SKILL.md body)")
    print()
    print(f"Planned changes to skill.json:")
    for note in notes:
        print(f"  - {note}")
    print()
    print(f"Auxiliary files:")
    print(f"  - {'WRITE' if add_validate else 'KEEP'}  scripts/validate.sh")
    print(f"  - {'WRITE' if add_usage else 'KEEP'}  references/usage.md")
    print(f"  - WRITE  MANIFEST.json (sha256 ledger of all files)")

    if not args.apply:
        print()
        print("Dry-run complete. Pass --apply to write these changes.")
        return 0

    # Ensure declared resources match what we will create on disk.
    # The stub manifest doesn't declare references/usage.md, but the
    # generated validate.sh §1 expects it. Add it to resources.references
    # if we're creating it.
    if add_usage:
        res = new_manifest.setdefault("resources", {})
        refs = res.setdefault("references", [])
        if "references/usage.md" not in refs:
            refs.append("references/usage.md")
    if add_validate:
        res = new_manifest.setdefault("resources", {})
        scripts = res.setdefault("scripts", [])
        if "scripts/validate.sh" not in scripts:
            scripts.append("scripts/validate.sh")

    # Apply
    new_json = json.dumps(new_manifest, indent=2, ensure_ascii=False) + "\n"
    skill_json_path.write_text(new_json, encoding="utf-8")

    if add_validate:
        validate_sh_path.parent.mkdir(parents=True, exist_ok=True)
        validate_sh_path.write_text(build_validate_sh(fm_name), encoding="utf-8")
        validate_sh_path.chmod(0o755)

    if add_usage:
        usage_md_path.parent.mkdir(parents=True, exist_ok=True)
        usage_md_path.write_text(
            f"# {fm_name} — Usage Reference\n\n"
            f"This reference was auto-created by ckodex-skill-tools/migrate.py during the v1→v1.1 migration.\n"
            f"Hand-edit to add worked examples and edge cases.\n",
            encoding="utf-8",
        )

    # Build MANIFEST.json last
    (target / "MANIFEST.json").write_text(build_manifest_ledger(target), encoding="utf-8")

    print()
    print(f"OK — migrated {target}")
    print(f"Verify with:  cd {target} && bash scripts/validate.sh")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
