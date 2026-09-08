#!/usr/bin/env python3
"""
Check for local type definitions that should be canonical.
Scans TypeScript files for interface declarations matching canonical type names
outside of generated/ directories.

Exit 0 if no drift detected, 1 otherwise.
"""

import re
import sys
from pathlib import Path

# Type names that must only appear in generated/ directories
CANONICAL_TYPES = [
    "DimensionScore",
    "SkillSummary",
    "SkillDetail",
    "AssessmentResult",
    "ClientError",
    "Grade",
    "Tier",
    "SyncStatus",
    "RegistryEntry",
    "LifecycleEvent",
    "UserPreference",
]

INTERFACE_PATTERN = re.compile(
    r"export\s+interface\s+({names})\b".format(
        names="|".join(re.escape(t) for t in CANONICAL_TYPES)
    )
)

EXCLUDE_DIRS = [
    "generated",
    "node_modules",
    ".git",
    "_archive",
    "dist",
    ".next",
    "target",
]


def scan(path: Path) -> list[dict]:
    drift = []
    for ts_file in path.rglob("*.ts"):
        if any(part in EXCLUDE_DIRS for part in ts_file.parts):
            continue
        content = ts_file.read_text()
        for match in INTERFACE_PATTERN.finditer(content):
            drift.append({
                "file": str(ts_file),
                "line": content[: match.start()].count("\n") + 1,
                "type": match.group(1),
            })
    return drift


def main() -> int:
    root = Path(__file__).parent.parent
    drift = scan(root)

    if not drift:
        print("OK: No canonical type drift detected.")
        return 0

    print(f"ERROR: Found {len(drift)} local type definition(s) that shadow canonical types:\n")
    for item in drift:
        print(f"  {item['file']}:{item['line']} — interface {item['type']}")
    print(
        "\nThese types must be imported from generated/ (client-model) instead of redefined locally."
    )
    print("See SKILLS-DOSSIER.md §8.5 Phase 2 for remediation guidance.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
