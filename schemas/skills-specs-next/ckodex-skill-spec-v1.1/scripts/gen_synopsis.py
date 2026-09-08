#!/usr/bin/env python3
"""
gen_synopsis.py — build-time helper for CKODEX skill manifests v1.1.

Generates a bounded synopsis string from the SKILL.md body and writes it
back into skill.json at metadata.synopsis. This is a deterministic
template-and-trim helper, not an LLM-driven summarizer — synopses must
be authored or regenerated when SKILL.md changes substantively.

Usage:
  python3 scripts/gen_synopsis.py path/to/skill-root/

Expects: skill-root/SKILL.md and skill-root/skill.json.
Writes:  skill-root/skill.json with metadata.synopsis populated/updated.

The bundled output is bounded to 2048 chars per the v1.1 schema. If the
generated synopsis would exceed the bound, the tool truncates with an
explicit "...[truncated]" marker; authors should hand-edit in that case.
"""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path

MAX_SYNOPSIS_LEN = 2048
TARGET_LEN       = 1500  # comfortable target; leaves room for hand-tuning


def extract_skill_md_body(md_text: str) -> str:
    """Strip YAML frontmatter; collapse markdown into a flat-ish paragraph."""
    # Drop YAML frontmatter
    body = re.sub(r"^---\s*\n.*?\n---\s*\n", "", md_text, count=1, flags=re.DOTALL)
    # Drop code fences (their contents are usually examples, not the description)
    body = re.sub(r"```.*?```", " ", body, flags=re.DOTALL)
    # Drop headers (keep the text)
    body = re.sub(r"^#{1,6}\s*", "", body, flags=re.MULTILINE)
    # Collapse whitespace
    body = re.sub(r"\s+", " ", body).strip()
    return body


def build_synopsis(body: str, name: str, description: str) -> str:
    """Compose a synopsis from key fragments of the body."""
    head = body[:TARGET_LEN]
    tail_marker = "" if len(body) <= TARGET_LEN else " ...[truncated; hand-edit recommended]"
    synopsis = head + tail_marker
    if len(synopsis) > MAX_SYNOPSIS_LEN:
        synopsis = synopsis[: MAX_SYNOPSIS_LEN - 5] + "...[T]"
    return synopsis


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__)
        return 2

    root = Path(argv[1])
    md_path  = root / "SKILL.md"
    js_path  = root / "skill.json"

    if not md_path.exists():
        print(f"ERROR: {md_path} not found")
        return 1
    if not js_path.exists():
        print(f"ERROR: {js_path} not found")
        return 1

    md_text = md_path.read_text(encoding="utf-8")
    body = extract_skill_md_body(md_text)

    with open(js_path, encoding="utf-8") as f:
        manifest = json.load(f)

    md = manifest.setdefault("metadata", {})
    name = md.get("name", "")
    description = md.get("description", "")
    synopsis = build_synopsis(body, name, description)
    md["synopsis"] = synopsis

    with open(js_path, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2, ensure_ascii=False)
        f.write("\n")

    print(f"OK — wrote synopsis ({len(synopsis)} chars) to {js_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
