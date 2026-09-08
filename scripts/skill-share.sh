#!/usr/bin/env bash
# skill-share — publish a canonical skill to a remote GitLab/GitHub group as its
# own repository, no manual copy. Uses git push-to-create: the skill becomes a
# new project <group-url>/<skill>.git. Auth is whatever git is configured for the
# host (store the token once: printf 'protocol=https\nhost=<host>\nusername=oauth2\npassword=<PAT>\n\n' | git credential approve).
#
# Usage:
#   skill-share.sh <skill-name-or-path> <group-url> [--repo-name <name>] [--branch <b>] [--dry-run]
#
# Example (verified):
#   skill-share.sh structurizr-dsl \
#     https://sc01-trt.thales-systems.ca/gitlab/aiml/common/skills-registry
#   → creates https://…/aiml/common/skills-registry/structurizr-dsl
set -euo pipefail

SKILL="${1:?skill name or path required}"; shift
GROUP="${1:?group url required (e.g. https://host/gitlab/group/subgroup)}"; shift
CANON_ROOT="${SKILLPACK_CANONICAL_ROOT:-$HOME/Skills/shared}"
REPO_NAME=""         # repo name in the group; default = skill dir name
BRANCH="main"
DRY_RUN=0
while [ $# -gt 0 ]; do
  case "$1" in
    --repo-name) REPO_NAME="$2"; shift 2;;
    --branch) BRANCH="$2"; shift 2;;
    --dry-run) DRY_RUN=1; shift;;
    *) echo "unknown flag: $1" >&2; exit 2;;
  esac
done

# Resolve the skill source directory (canonical store, or an explicit path).
if [ -d "$SKILL" ]; then SRC="$SKILL"; else SRC="$CANON_ROOT/$SKILL"; fi
SRC="$(cd "$SRC" 2>/dev/null && pwd -P || true)"
[ -n "$SRC" ] && [ -d "$SRC" ] || { echo "✗ skill not found: $SKILL (looked in $CANON_ROOT)" >&2; exit 1; }
[ -f "$SRC/SKILL.md" ] || echo "⚠ no SKILL.md in $SRC (sharing anyway)"
NAME="${REPO_NAME:-$(basename "$SRC")}"
URL="${GROUP%/}/$NAME.git"

echo "── skill-share ──────────────────────────────────────────────"
echo "  skill  : $(basename "$SRC")  ($(find "$SRC" -type f -not -path '*/.git/*' | wc -l | tr -d ' ') files)"
echo "  source : $SRC"
echo "  target : $URL   (new repo in the group)"
echo "─────────────────────────────────────────────────────────────"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK" 2>/dev/null || true' EXIT

# Stage a clean copy (real files, VCS noise stripped) and make a fresh 1-commit repo.
rsync -a --exclude '.git' --exclude '.DS_Store' --exclude 'node_modules' --exclude 'target' "$SRC"/ "$WORK/repo/"
cd "$WORK/repo"
git init -q -b "$BRANCH"
git add -A
git -c user.name="${GIT_AUTHOR_NAME:-skillpack}" -c user.email="${GIT_AUTHOR_EMAIL:-skillpack@localhost}" \
  commit -q -m "$NAME skill — published from the canonical store via skillpack skill-share"
git remote add origin "$URL"

echo "▸ change set:"; git show --stat --oneline HEAD | sed 's/^/    /' | head -20

if [ "$DRY_RUN" = "1" ]; then
  echo "◌ dry-run — not pushing. Re-run without --dry-run to publish."
  exit 0
fi

echo "▸ push-to-create → $NAME …"
if GIT_TERMINAL_PROMPT=0 git push -u origin "$BRANCH" 2>"$WORK/push.err"; then
  echo "✓ shared '$NAME' → ${GROUP%/}/$NAME"
else
  echo "✗ push failed:" >&2
  sed 's/[A-Za-z0-9_-]\{24,\}/<redacted>/g' "$WORK/push.err" | sed 's/^/    /' >&2
  h="$(echo "$URL" | sed -E 's#https?://([^/]+)/.*#\1#')"
  echo "    → store creds once: printf 'protocol=https\\nhost=$h\\nusername=oauth2\\npassword=<PAT>\\n\\n' | git credential approve" >&2
  exit 1
fi
