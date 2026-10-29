#!/usr/bin/env bash
# Proves check-no-issue-refs.sh fails on each kind of issue reference and passes on clean files.
# The fixtures are written to a temporary directory at test time and never committed, because a
# committed fixture would trip the check on this repository. Runs on macOS bash 3.2.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GUARD="$ROOT/scripts/check-no-issue-refs.sh"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/no-issue-refs-selftest.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  echo "Usage: scripts/check-no-issue-refs-selftest.sh"
  echo "Proves check-no-issue-refs.sh rejects 'issue N', 'issue #N', '(#N)' and 'issue-N' forms."
  exit 0
fi

# Each case: file name, then a line that must make the guard fail. The number is assembled at
# run time so this script does not contain the patterns it tests.
N=7
CASES="notes.md:see issue $N for details
heading.md:### Coverage (issue #$N)
fix.sh:# fix (#$N)
run.sh:tee /tmp/issue-$N-before.log
ci.yml:# tracked in Issue_$N
deps.toml:# issue #$N pins this
lib.rs:// workaround for issue $N"

while IFS= read -r LINE; do
  FILE="${LINE%%:*}"
  CASE_DIR="$SCRATCH/case-$FILE"
  mkdir -p "$CASE_DIR"
  printf '%s\n' "${LINE#*:}" >"$CASE_DIR/$FILE"
  if OUTPUT="$("$GUARD" "$CASE_DIR" 2>&1)"; then
    echo "SELF-TEST FAILED: the guard passed on $FILE: ${LINE#*:}"
    exit 1
  fi
  if ! grep -F -q "./$FILE:1:" <<<"$OUTPUT"; then
    echo "SELF-TEST FAILED: the guard failed without naming $FILE:"
    echo "$OUTPUT"
    exit 1
  fi
  echo "ok: fails on $FILE"
done <<<"$CASES"

# Clean files, including words that only look similar, must pass.
mkdir -p "$SCRATCH/clean"
printf 'Issues are tracked outside the tree.\nSee the issue tracker.\n' >"$SCRATCH/clean/notes.md"
printf 'COLOR="#0075ca"\n# check-no-issue-refs.sh\n' >"$SCRATCH/clean/run.sh"
if ! OUTPUT="$("$GUARD" "$SCRATCH/clean" 2>&1)"; then
  echo "SELF-TEST FAILED: the guard failed on clean files:"
  echo "$OUTPUT"
  exit 1
fi
echo "ok: passes on clean files"
echo "check-no-issue-refs self-test: every reference form fails, clean files pass"
