#!/usr/bin/env bash
# migrate-skills.sh — apply schema compliance fixes to ~/skills/shared
#
# Equivalent to: sctl migrate --root ~/skills/shared
# Use this script if sctl binary isn't on PATH yet.
#
# What it fixes:
#   - name: field in SKILL.md frontmatter must match directory name
#   - CRLF line endings converted to LF
#   - Missing SKILL.md files get a minimal stub
#
# Usage:
#   bash scripts/migrate-skills.sh              # apply fixes
#   bash scripts/migrate-skills.sh --dry-run    # preview only

set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
  echo "[dry-run] No files will be modified."
fi

SKILLS_DIR="${SKILLPACK_ROOT:-$HOME/skills/shared}"

if [[ ! -d "$SKILLS_DIR" ]]; then
  echo "Error: canonical root not found: $SKILLS_DIR" >&2
  exit 1
fi

echo "Migrating skills in: $SKILLS_DIR"
echo ""

fixed_names=0
fixed_crlf=0
created_missing=0
already_ok=0
errors=0

fix_name() {
  local skill_md="$1"
  local expected="$2"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "  [dry-run] NAME     $expected/SKILL.md"
    return
  fi
  # BSD sed (macOS) requires '' for in-place without backup
  sed -i '' "s|^name:.*|name: $expected|" "$skill_md"
}

fix_crlf() {
  local skill_md="$1"
  local skill_name="$2"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "  [dry-run] CRLF     $skill_name/SKILL.md"
    return
  fi
  # Convert CRLF → LF using perl (available on macOS without extras)
  perl -pi -e 's/\r\n/\n/g; s/\r/\n/g' "$skill_md"
  echo "  CRLF     $skill_name/SKILL.md"
}

create_missing() {
  local skill_dir="$1"
  local skill_name="$2"
  local description="${skill_name//-/ } skill"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "  [dry-run] CREATE  $skill_name/SKILL.md"
    return
  fi
  cat > "$skill_dir/SKILL.md" <<SKILLMD
---
name: $skill_name
description: "$description"
---

# $skill_name

<!-- TODO(ckodex): add skill description -->
SKILLMD
  echo "  CREATED  $skill_name/SKILL.md"
}

for dir in "$SKILLS_DIR"/*/; do
  dir_name=$(basename "$dir")
  skill_md="$dir/SKILL.md"
  changed=false

  if [[ ! -f "$skill_md" ]]; then
    create_missing "$dir" "$dir_name"
    created_missing=$((created_missing + 1))
    continue
  fi

  # Fix CRLF first (so subsequent awk sees clean LF)
  if file "$skill_md" | grep -q CRLF || grep -qP "\r" "$skill_md" 2>/dev/null; then
    fix_crlf "$skill_md" "$dir_name"
    fixed_crlf=$((fixed_crlf + 1))
    changed=true
  fi

  # Check name field (awk strips trailing \r for safety)
  fm_name=$(awk 'BEGIN{found=0}/^---/{found++; next} found==1 && /^name:/{gsub(/\r/,""); gsub(/^name:[[:space:]]*/,""); print; exit}' "$skill_md")

  if [[ -z "$fm_name" ]]; then
    echo "  WARN     $dir_name/SKILL.md — missing name: field (check manually)"
    errors=$((errors + 1))
  elif [[ "$fm_name" != "$dir_name" ]]; then
    echo "  NAME     $dir_name/SKILL.md  $fm_name → $dir_name"
    fix_name "$skill_md" "$dir_name"
    fixed_names=$((fixed_names + 1))
    changed=true
  fi

  if [[ "$changed" == "false" && "$fm_name" == "$dir_name" ]]; then
    already_ok=$((already_ok + 1))
  fi
done

echo ""
echo "Migration complete:"
echo "  Skills scanned:   $(( fixed_names + fixed_crlf + created_missing + already_ok ))"
echo "  Already compliant: $already_ok"
[[ $fixed_names -gt 0 ]]    && echo "  Names fixed:       $fixed_names"
[[ $fixed_crlf -gt 0 ]]     && echo "  CRLF → LF:         $fixed_crlf"
[[ $created_missing -gt 0 ]] && echo "  SKILL.md created:  $created_missing"
[[ $errors -gt 0 ]]          && echo "  Warnings:          $errors (check manually)"
