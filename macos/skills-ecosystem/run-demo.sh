#!/bin/bash
set -e

# Setup clean sandbox directories
DEMO_ROOT="/Users/mchorfa/Developer/skills-ecosystem/demo-sandbox"
mkdir -p "$DEMO_ROOT/canonical"
mkdir -p "$DEMO_ROOT/agents/claude"
mkdir -p "$DEMO_ROOT/agents/windsurf"
mkdir -p "$DEMO_ROOT/agents/cursor"

echo "=== 1. Creating Mock Skills ==="
# Skill A: Valid
mkdir -p "$DEMO_ROOT/canonical/docker-expert"
echo -e "---\ndescription: \"Provides high-performance docker instructions.\"\n---\n# Docker Expert\nRun fast." > "$DEMO_ROOT/canonical/docker-expert/SKILL.md"

# Skill B: Valid
mkdir -p "$DEMO_ROOT/canonical/git-master"
echo -e "---\ndescription: \"Advanced conventional commit workflows.\"\n---\n# Git Master\nKeep history clean." > "$DEMO_ROOT/canonical/git-master/SKILL.md"

echo "=== 2. Running status with correct environment ==="
export SKILLS_ROOT="$DEMO_ROOT/canonical"
export CLAUDE_SKILLS_DIR="$DEMO_ROOT/agents/claude"
export WINDSURF_SKILLS_DIR="$DEMO_ROOT/agents/windsurf"
export CURSOR_RULES_DIR="$DEMO_ROOT/agents/cursor"

# Build target to ensure latest is used
cd /Users/mchorfa/Developer/skills-ecosystem/SkillsCLI
swift build -c release

CLI_BIN="/Users/mchorfa/Developer/skills-ecosystem/SkillsCLI/.build/release/SkillsCLI"

echo ""
echo ">>> Executing: skills-cli --status"
"$CLI_BIN" --status

echo ""
echo "=== 3. Performing a live synchronization sync ==="
echo ">>> Executing: skills-cli"
"$CLI_BIN"

echo ""
echo "=== 4. Checking manifest.yaml generated atomically ==="
cat "$DEMO_ROOT/canonical/manifest.yaml"

echo ""
echo "=== 5. Checking generated symlinks for Claude and Windsurf ==="
ls -l "$DEMO_ROOT/agents/claude"
ls -l "$DEMO_ROOT/agents/windsurf"

echo ""
echo "=== 6. Checking generated Cursor rules index file ==="
cat "$DEMO_ROOT/agents/cursor/skills-index.mdc"

echo ""
echo "=== 7. Verifying IPGuard boundary protection ==="
echo "Creating a prohibited skill matching 'ip-pending'..."
mkdir -p "$DEMO_ROOT/canonical/cortaix-csr-secrets"
echo -e "---\ndescription: \"Internal design rules.\"\n---\n# Restricted" > "$DEMO_ROOT/canonical/cortaix-csr-secrets/SKILL.md"

echo ">>> Executing: skills-cli (Should block and fail-closed!)"
if "$CLI_BIN"; then
  echo "ERROR: Should have been blocked by IPGuard!"
  exit 1
else
  echo "SUCCESS: IPGuard successfully caught the boundary violation and halted execution."
fi

# Clean up
rm -rf "$DEMO_ROOT/canonical/cortaix-csr-secrets"
