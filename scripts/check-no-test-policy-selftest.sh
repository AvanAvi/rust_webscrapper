#!/usr/bin/env bash
# Proves check-no-test-policy.sh works: it must FAIL for a throwaway binary that enables
# crawler-fetch/test-policy and PASS for one that does not. The manifests are copied to a
# scratch directory (outside the workspace) with the absolute path to crawler-fetch filled in.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

for CASE in enabled clean; do
  mkdir -p "$SCRATCH/$CASE/src"
  sed "s|@CRAWLER_FETCH@|$ROOT/crates/crawler-fetch|" "$ROOT/scripts/fixtures/test-policy-probe/$CASE.Cargo.toml" > "$SCRATCH/$CASE/Cargo.toml"
  printf 'fn main() {}\n' > "$SCRATCH/$CASE/src/main.rs"
  cp "$ROOT/Cargo.lock" "$SCRATCH/$CASE/Cargo.lock"
done

# The enabled case must fail for the right reason, not because of any other error.
if ENABLED_OUT="$("$ROOT/scripts/check-no-test-policy.sh" "$SCRATCH/enabled/Cargo.toml" probe 2>&1)"; then
  echo "SELF-TEST FAILED: the check passed on a binary that enables test-policy"
  exit 1
fi
if ! grep -q "enables crawler-fetch/test-policy" <<<"$ENABLED_OUT"; then
  echo "SELF-TEST FAILED: the check failed for another reason:"
  echo "$ENABLED_OUT"
  exit 1
fi
if ! CLEAN_OUT="$("$ROOT/scripts/check-no-test-policy.sh" "$SCRATCH/clean/Cargo.toml" probe 2>&1)"; then
  echo "SELF-TEST FAILED: the check failed on a clean binary:"
  echo "$CLEAN_OUT"
  exit 1
fi
if ! grep -q "probe does not enable test-policy" <<<"$CLEAN_OUT"; then
  echo "SELF-TEST FAILED: unexpected output for the clean binary:"
  echo "$CLEAN_OUT"
  exit 1
fi
echo "check-no-test-policy self-test: fails on enabled, passes on clean"

# The enabled feature exposes the root API; a default build must reject its use.
"$ROOT/scripts/check-test-root-api.sh" enabled
"$ROOT/scripts/check-test-root-api.sh" clean
