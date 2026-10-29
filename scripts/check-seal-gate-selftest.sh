#!/usr/bin/env bash
# Proves check-seal-gate.sh fails for the right reason on each broken input and passes on good
# input. Uses this workspace's real `cargo metadata` plus fabricated test output, so it needs no
# build. Runs on macOS bash 3.2.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GUARD="$ROOT/scripts/check-seal-gate.sh"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/seal-gate-selftest.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  echo "Usage: scripts/check-seal-gate-selftest.sh"
  echo "Proves check-seal-gate.sh rejects a foreign ledger-seal user, a missing or still-present"
  echo "compile_fail doctest, a test count below the floor, and a skipped seal target."
  exit 0
fi

# Runs the guard with the given arguments and requires it to fail with the given message.
expect_fail() {
  local message="$1" output
  shift
  if output="$("$GUARD" "$@" 2>&1)"; then
    echo "SELF-TEST FAILED: the guard passed: $*"
    exit 1
  fi
  if ! grep -F -q "$message" <<<"$output"; then
    echo "SELF-TEST FAILED: the guard failed for another reason ($*):"
    echo "$output"
    exit 1
  fi
  echo "ok: fails on bad input: $message"
}

# Runs the guard with the given arguments and requires it to pass.
expect_pass() {
  local output
  if ! output="$("$GUARD" "$@" 2>&1)"; then
    echo "SELF-TEST FAILED: the guard failed on good input ($*):"
    echo "$output"
    exit 1
  fi
  echo "ok: passes on good input: $1"
}

# Metadata: the real workspace passes; crawler-fetch enabling the feature fails.
cargo metadata --locked --no-deps --format-version 1 --manifest-path "$ROOT/Cargo.toml" \
  >"$SCRATCH/metadata.json"
python3 -I - "$SCRATCH/metadata.json" "$SCRATCH/metadata-bad.json" <<'PY'
import json, sys
o_meta = json.load(open(sys.argv[1]))
for o_pkg in o_meta["packages"]:
    for o_dep in o_pkg["dependencies"]:
        if o_pkg["name"] == "crawler-fetch" and o_dep["name"] == "crawler-core":
            o_dep["features"] = o_dep.get("features", []) + ["ledger-seal"]
json.dump(o_meta, open(sys.argv[2], "w"))
PY
expect_pass check-metadata "$SCRATCH/metadata.json"
expect_fail "enables ledger-seal: crawler-fetch" check-metadata "$SCRATCH/metadata-bad.json"

# Doctests: the gate's compile_fail test must run without the feature and be gone with it.
DOC_FAIL='test crates/crawler-core/src/receipt_submission.rs - receipt_submission::ReceiptSubmission (line 21) - compile fail ... ok'
DOC_PASS='test crates/crawler-core/src/receipt_submission.rs - receipt_submission::ReceiptSubmission (line 21) ... ok'
printf '%s\n' "$DOC_FAIL" >"$SCRATCH/doc-off.txt"
printf '%s\n' "$DOC_PASS" >"$SCRATCH/doc-on.txt"
printf 'test result: ok. 0 passed\n' >"$SCRATCH/doc-empty.txt"
expect_pass check-doc "$SCRATCH/doc-off.txt" "$SCRATCH/doc-on.txt"
expect_fail "compile_fail doctest did not run" check-doc "$SCRATCH/doc-empty.txt" "$SCRATCH/doc-on.txt"
expect_fail "did not compile and pass" check-doc "$SCRATCH/doc-off.txt" "$SCRATCH/doc-empty.txt"
expect_fail "compile_fail doctest is still present" check-doc "$SCRATCH/doc-off.txt" "$SCRATCH/doc-off.txt"

# Listing: three tests and every seal target pass a floor of 3; a floor of 4 or a skipped
# target fails.
for TARGET in receipt_encoding receipt_rules receipt_schema receipt_size; do
  printf '     Running tests/%s.rs (target/debug/deps/%s-0)\n' "$TARGET" "$TARGET"
done >"$SCRATCH/list.txt"
printf 'test_a: test\ntest_b: test\ntest_c: test\n' >>"$SCRATCH/list.txt"
grep -v 'receipt_size' "$SCRATCH/list.txt" >"$SCRATCH/list-skipped.txt"
expect_pass check-list "$SCRATCH/list.txt" 3
expect_fail "below the floor of 4" check-list "$SCRATCH/list.txt" 4
expect_fail "test target receipt_size did not run" check-list "$SCRATCH/list-skipped.txt" 3

echo "check-seal-gate self-test: all broken inputs fail for the right reason, good inputs pass"
