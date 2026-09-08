#!/usr/bin/env bash
# sync-protos.sh
# Fallback script that copies canonical proto definitions from the central
# proto/ directory into macOS package directories.
#
# The SPM build plugins in SkillsDaemon and SkillsCLI are configured to read
# proto files directly from proto/ using relative paths. This script is only
# needed in environments where the build system prevents access to files
# outside the package directory (e.g., strict sandboxing).
#
# Usage: ./scripts/sync-protos.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

SKILLS_PROTO="${REPO_ROOT}/proto/macos/v1/skills.proto"

DAEMON_DIR="${REPO_ROOT}/macos/skills-ecosystem/SkillsDaemon/Sources/SkillsDaemon"
CLI_DIR="${REPO_ROOT}/macos/skills-ecosystem/SkillsCLI/Sources/SkillsCLI"

sync_file() {
    local src="$1"
    local dst="$2"

    if [ ! -f "$src" ]; then
        echo "ERROR: Source proto not found: $src"
        exit 1
    fi

    if [ -L "$dst" ]; then
        echo "  Symlink exists: $dst (skipping)"
        return 0
    fi

    cp "$src" "$dst"
    echo "  Copied: $src -> $dst"
}

echo "Syncing protos to macOS targets (fallback for sandboxed builds)..."

sync_file "$SKILLS_PROTO" "${DAEMON_DIR}/Skills.proto"
sync_file "$SKILLS_PROTO" "${CLI_DIR}/Skills.proto"

echo "Proto sync complete."
