#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────
# scripts/release.sh — Bump version, commit, tag, push.
#
# Usage:
#   ./scripts/release.sh patch     # 0.1.8  → 0.1.9
#   ./scripts/release.sh minor     # 0.1.9  → 0.2.0
#   ./scripts/release.sh major     # 0.2.0  → 1.0.0
#   ./scripts/release.sh 0.3.0     # set an exact version
#
# What it does:
#   1. Reads the current version from Cargo.toml
#   2. Computes the next version (or uses the one you gave)
#   3. Updates Cargo.toml + Cargo.lock
#   4. Commits: "release: v<next>"
#   5. Tags:   v<next>
#   6. Pushes commit + tag to origin
#
# The existing release.yml workflow picks up the tag push and
# handles Docker images, binaries, and the GitHub Release.
# ──────────────────────────────────────────────────────────────
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

die() { echo -e "${RED}error:${RESET} $*" >&2; exit 1; }

# ── Resolve repo root (always run from repo root) ────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# ── Parse current version ───────────────────────────────────
CURRENT=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"
echo -e "${CYAN}current version:${RESET} ${BOLD}$CURRENT${RESET}"

# ── Compute next version ────────────────────────────────────
ARG="${1:-}"
case "$ARG" in
  patch) NEXT="$MAJOR.$MINOR.$((PATCH + 1))" ;;
  minor) NEXT="$MAJOR.$((MINOR + 1)).0" ;;
  major) NEXT="$((MAJOR + 1)).0.0" ;;
  "")    die "usage: $0 {patch|minor|major|<version>}" ;;
  *)
    # Validate explicit semver
    if ! echo "$ARG" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
      die "invalid version '$ARG' — expected semver like 1.2.3"
    fi
    NEXT="$ARG"
    ;;
esac

echo -e "${CYAN}next version:   ${RESET} ${BOLD}$NEXT${RESET}"

# ── Guard rails ─────────────────────────────────────────────
if git rev-parse "v$NEXT" >/dev/null 2>&1; then
  die "tag v$NEXT already exists"
fi

if [ -n "$(git status --porcelain -- ':!Cargo.toml' ':!Cargo.lock')" ]; then
  die "working tree has uncommitted changes outside Cargo.toml/Cargo.lock — commit or stash first"
fi

# ── Apply version bump ──────────────────────────────────────
sed -i.bak "s/^version = \"$CURRENT\"/version = \"$NEXT\"/" Cargo.toml
rm -f Cargo.toml.bak

# Regenerate lockfile so Cargo.lock stays in sync
cargo generate-lockfile --quiet 2>/dev/null || cargo generate-lockfile

# ── Commit, tag, push ───────────────────────────────────────
git add Cargo.toml Cargo.lock
git commit -m "release: v$NEXT"
git tag -a "v$NEXT" -m "v$NEXT"

echo ""
echo -e "${GREEN}✓${RESET} committed and tagged ${BOLD}v$NEXT${RESET}"
echo ""

read -rp "Push commit + tag to origin? [Y/n] " CONFIRM
CONFIRM="${CONFIRM:-Y}"
if [[ "$CONFIRM" =~ ^[Yy]$ ]]; then
  git push origin HEAD --follow-tags
  echo -e "${GREEN}✓${RESET} pushed — release workflow will pick up ${BOLD}v$NEXT${RESET}"
else
  echo -e "${CYAN}skipped push.${RESET} Run manually:"
  echo "  git push origin HEAD --follow-tags"
fi
