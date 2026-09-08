#!/usr/bin/env bash
# sync-agent-tools.sh — idempotent symlink sync from ~/skills/shared/ to all agent tool paths
# Usage: sync-agent-tools.sh [--dry-run]
# Canonical store: ~/skills/shared/  (192 skills)
# All tool paths become: <path>/<skill> -> ~/skills/shared/<skill>  (symlinks, no copies)

set -euo pipefail

SHARED="$HOME/skills/shared"
DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

log()   { printf "[sync] %s\n" "$*"; }
warn()  { printf "[WARN] %s\n" "$*" >&2; }
run()   { if $DRY_RUN; then printf "[dry ] %s\n" "$*"; else eval "$*"; fi; }

if [[ ! -d "$SHARED" ]]; then
  warn "Shared store not found: $SHARED"
  exit 1
fi

# ---------------------------------------------------------------------------
# sync_dir <tool-path>
#   1. mkdir -p if absent
#   2. remove broken symlinks
#   3. convert real dirs (that exist in shared) to symlinks
#   4. add missing symlinks for all skills in shared
#   5. warn on extras (entries not in shared) — never delete
# ---------------------------------------------------------------------------
sync_dir() {
  local tool_path="$1"
  local converted=0 added=0 broken_removed=0 extras=0

  run "mkdir -p \"$tool_path\""

  # Step 2: remove broken symlinks
  while IFS= read -r -d '' link; do
    if [[ -L "$link" && ! -e "$link" ]]; then
      log "  remove broken: $link"
      run "rm -f \"$link\""
      (( broken_removed++ )) || true
    fi
  done < <(find "$tool_path" -maxdepth 1 -type l -print0 2>/dev/null)

  # Step 3 & 4: process each skill in shared
  while IFS= read -r -d '' skill_dir; do
    skill=$(basename "$skill_dir")
    dest="$tool_path/$skill"

    if [[ -L "$dest" ]]; then
      # already a symlink — check it points to shared
      current_target=$(readlink "$dest")
      expected="$SHARED/$skill"
      if [[ "$current_target" != "$expected" ]]; then
        log "  repoint: $skill ($current_target -> $expected)"
        run "rm -f \"$dest\""
        run "ln -s \"$expected\" \"$dest\""
        (( converted++ )) || true
      fi
    elif [[ -d "$dest" ]]; then
      # real directory — convert to symlink
      log "  convert: $skill (real dir -> symlink)"
      run "rm -rf \"$dest\""
      run "ln -s \"$SHARED/$skill\" \"$dest\""
      (( converted++ )) || true
    else
      # missing — add symlink
      run "ln -s \"$SHARED/$skill\" \"$dest\""
      (( added++ )) || true
    fi
  done < <(find "$SHARED" -maxdepth 1 -mindepth 1 -type d -print0 | sort -z)

  # Step 5: warn on extras (in tool_path but not in shared)
  while IFS= read -r -d '' entry; do
    name=$(basename "$entry")
    if [[ ! -e "$SHARED/$name" ]]; then
      warn "  EXTRA (not in shared): $tool_path/$name"
      (( extras++ )) || true
    fi
  done < <(find "$tool_path" -maxdepth 1 -mindepth 1 -print0 2>/dev/null | sort -z)

  log "  done: converted=$converted added=$added broken_removed=$broken_removed extras=$extras"
}

# ---------------------------------------------------------------------------
# Directory-based tool paths
# ---------------------------------------------------------------------------

# Installed tools
log "==> claude"
sync_dir "$HOME/.claude/skills"

log "==> codex"
sync_dir "$HOME/.codex/skills"

log "==> aider"
sync_dir "$HOME/.aider/skills"

log "==> opencode"
sync_dir "$HOME/.config/opencode/skills"

log "==> agents (generic)"
sync_dir "$HOME/.agents/skills"

log "==> gemini"
sync_dir "$HOME/.gemini/skills"

log "==> roo"
sync_dir "$HOME/.roo/rules"

log "==> continue"
sync_dir "$HOME/.continue/rules"

log "==> copilot"
sync_dir "$HOME/.github/instructions"

log "==> augment"
sync_dir "$HOME/.augment/rules"

# Not-yet-installed — create ready paths
log "==> windsurf (not installed, pre-creating)"
sync_dir "$HOME/.windsurf/rules"

log "==> openclaw (not installed, pre-creating)"
sync_dir "$HOME/.openclaw/skills"

log "==> zed (not installed, pre-creating)"
run "mkdir -p \"$HOME/.config/zed/prompt_overrides\""
sync_dir "$HOME/.config/zed/prompt_overrides"

log "==> amp (not installed, pre-creating)"
sync_dir "$HOME/.amp/skills"

log "==> antigravity (not installed, pre-creating)"
sync_dir "$HOME/.antigravity/skills"

log "==> pi (not installed, pre-creating)"
sync_dir "$HOME/.pi/skills"

log "==> hermes (not installed, pre-creating)"
sync_dir "$HOME/.hermes/skills"

log "==> cline (not installed, pre-creating)"
sync_dir "$HOME/Documents/Cline/Rules"

# ---------------------------------------------------------------------------
# Cursor: regenerate skills-index.mdc (single index file, not per-skill dirs)
# ---------------------------------------------------------------------------
log "==> cursor (regenerate skills-index.mdc)"
cursor_rules="$HOME/.cursor/rules"
run "mkdir -p \"$cursor_rules\""

if ! $DRY_RUN; then
  index_file="$cursor_rules/skills-index.mdc"
  skill_count=$(find "$SHARED" -maxdepth 1 -mindepth 1 -type d | wc -l | tr -d ' ')
  {
    printf -- "---\n"
    printf "description: \"Personal skill catalog (%s skills). Canonical store: %s. Auto-generated — do not hand-edit.\"\n" \
      "$skill_count" "$SHARED"
    printf "globs: [\"**/*\"]\n"
    printf "alwaysApply: false\n"
    printf -- "---\n\n"
    printf "# Available Skills\n\n"
    printf "Canonical store: \`%s\`\n\n" "$SHARED"
    printf "| Skill | Path | Description |\n"
    printf "|---|---|---|\n"
    while IFS= read -r -d '' skill_dir; do
      skill=$(basename "$skill_dir")
      skill_md="$skill_dir/SKILL.md"
      if [[ -f "$skill_md" ]]; then
        desc=$(awk '/^---/{found++; next} found==1 && /^description:/{
          gsub(/^description:[[:space:]]*/,""); gsub(/^\"|"$/,""); print; exit
        }' "$skill_md" | cut -c1-120)
      else
        desc=""
      fi
      printf "| \`%s\` | \`%s\` | %s |\n" "$skill" "$skill_dir" "$desc"
    done < <(find "$SHARED" -maxdepth 1 -mindepth 1 -type d -print0 | sort -z)
  } > "$index_file"
  log "  wrote $index_file ($skill_count skills)"
fi

# ---------------------------------------------------------------------------
# Verification
# ---------------------------------------------------------------------------
if ! $DRY_RUN; then
  printf "\n[sync] === VERIFICATION ===\n"
  all_ok=true
  total_shared=$(find "$SHARED" -maxdepth 1 -mindepth 1 -type d | wc -l | tr -d ' ')

  for tool_path in \
    "$HOME/.claude/skills" \
    "$HOME/.codex/skills" \
    "$HOME/.aider/skills" \
    "$HOME/.config/opencode/skills" \
    "$HOME/.agents/skills" \
    "$HOME/.gemini/skills" \
    "$HOME/.roo/rules" \
    "$HOME/.continue/rules" \
    "$HOME/.github/instructions" \
    "$HOME/.augment/rules" \
    "$HOME/.windsurf/rules" \
    "$HOME/.openclaw/skills" \
    "$HOME/.config/zed/prompt_overrides" \
    "$HOME/.amp/skills" \
    "$HOME/.antigravity/skills" \
    "$HOME/.pi/skills" \
    "$HOME/.hermes/skills" \
    "$HOME/Documents/Cline/Rules"; do

    links=$(find "$tool_path" -maxdepth 1 -type l 2>/dev/null | wc -l | tr -d ' ')
    broken=$(find "$tool_path" -maxdepth 1 -type l ! -e 2>/dev/null | wc -l | tr -d ' ')
    real=$(find "$tool_path" -maxdepth 1 ! -type l ! -path "$tool_path" 2>/dev/null | wc -l | tr -d ' ')
    label=$(echo "$tool_path" | sed "s|$HOME/||")

    status="OK"
    [[ "$broken" -gt 0 ]] && { status="BROKEN_LINKS=$broken"; all_ok=false; }
    [[ "$real" -gt 0 ]] && { status="${status} REAL_DIRS=$real"; all_ok=false; }

    printf "  %-40s  links=%-4s / %-4s  broken=%-3s  real=%-3s  %s\n" \
      "$label" "$links" "$total_shared" "$broken" "$real" "$status"
  done

  if $all_ok; then
    printf "\n[sync] ALL OK — %s tools, %s skills each, zero copies, zero broken links\n" \
      "18" "$total_shared"
  else
    printf "\n[sync] ISSUES FOUND — review warnings above\n"
    exit 1
  fi
fi
