#!/usr/bin/env bash
# Fails if a source, doc, script or config file names an issue number. Issue numbers belong in
# commit bodies and pull request descriptions, never in the tree. Runs on macOS bash 3.2.
#
# Usage: check-no-issue-refs.sh [DIRECTORY]   (default: the repository root)
#
# Two patterns: references such as "issue N", "issue #N" or "(#N)", and names such as
# "issue-N" or "issue_N" (for example in log file names), where N is a number.
set -euo pipefail

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
  exit 0
fi

DIR="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
SCRATCH="$(mktemp "${TMPDIR:-/tmp}/no-issue-refs.XXXXXX")"
trap 'rm -f "$SCRATCH"' EXIT

cd "$DIR"
grep -rn -i -E "issue #?[0-9]+|\(#[0-9]+\)|issue[-_][0-9]+" \
  --include='*.md' --include='*.sh' --include='*.yml' --include='*.toml' --include='*.rs' . \
  | grep -v '^./target' >"$SCRATCH" || true

if [[ -s "$SCRATCH" ]]; then
  cat "$SCRATCH"
  echo "check-no-issue-refs: FAIL: the lines above name an issue number"
  exit 1
fi
echo "check-no-issue-refs: no issue numbers in .md, .sh, .yml, .toml or .rs files"
