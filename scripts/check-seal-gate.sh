#!/usr/bin/env bash
# Guards the ledger-seal gate: ReceiptSubmission::seal exists only with crawler-core's
# `ledger-seal` feature, and only crawler-ledger may enable it. Runs on macOS bash 3.2.
#
# Full run (no arguments) checks, in order:
#   1. metadata: no workspace package except crawler-ledger enables crawler-core/ledger-seal;
#   2. sources:  no `.seal(` call in any crate's src/ except crawler-core (the definition)
#                and crawler-ledger;
#   3. doc:      `cargo test -p crawler-core --doc` without the feature runs the compile_fail
#                doctest (an outside crate cannot call seal), and with the feature the same
#                snippet compiles (so the compile failure is the gate, not a typo);
#   4. count:    `cargo test -p crawler-core --features ledger-seal --tests -- --list` lists at
#                least the committed floor of tests and runs every seal-using target. Cargo skips
#                targets with unmet required-features silently, so this is what catches lost tests.
#
# Check modes read captured output instead of running cargo; the self-test feeds them
# fabricated input:
#   check-seal-gate.sh check-metadata METADATA_JSON
#   check-seal-gate.sh check-doc DOC_OUTPUT_WITHOUT_FEATURE DOC_OUTPUT_WITH_FEATURE
#   check-seal-gate.sh check-list LIST_OUTPUT FLOOR
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FLOOR_FILE="$ROOT/scripts/fixtures/core-test-floor.txt"
# Test targets that call seal; each must appear in the listing with the feature on.
SEAL_TARGETS="receipt_encoding receipt_rules receipt_schema receipt_size"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/seal-gate.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT

usage() {
  sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'
}

fail() {
  echo "check-seal-gate: FAIL: $*"
  exit 1
}

check_metadata() {
  python3 -I - "$1" <<'PY' || fail "a package other than crawler-ledger enables crawler-core/ledger-seal"
import json, sys
o_meta = json.load(open(sys.argv[1]))
vec_str_bad = []
for o_pkg in o_meta["packages"]:
    for o_dep in o_pkg["dependencies"]:
        if o_dep["name"] == "crawler-core" and "ledger-seal" in o_dep.get("features", []):
            if o_pkg["name"] != "crawler-ledger":
                vec_str_bad.append(f'{o_pkg["name"]} ({o_dep.get("kind") or "normal"})')
for str_bad in vec_str_bad:
    print(f"check-seal-gate: enables ledger-seal: {str_bad}")
sys.exit(1 if vec_str_bad else 0)
PY
  echo "check-seal-gate: metadata ok, only crawler-ledger enables ledger-seal"
}

check_sources() {
  local hits
  hits="$(grep -rn -E '\.seal\(' "$ROOT"/crates/*/src --include='*.rs' \
    | grep -v -E '^'"$ROOT"'/crates/(crawler-core|crawler-ledger)/src/' || true)"
  if [[ -n "$hits" ]]; then
    echo "$hits"
    fail "seal is called outside crawler-core and crawler-ledger"
  fi
  echo "check-seal-gate: sources ok, no seal call outside crawler-core and crawler-ledger"
}

check_doc() {
  grep -E -q 'receipt_submission.*compile fail.* \.\.\. ok' "$1" ||
    fail "without ledger-seal, the compile_fail doctest did not run and pass"
  grep -E -q 'receipt_submission.* \.\.\. ok' "$2" ||
    fail "with ledger-seal, the seal snippet did not compile and pass"
  ! grep 'receipt_submission' "$2" | grep -q 'compile fail' ||
    fail "with ledger-seal, the compile_fail doctest is still present"
  echo "check-seal-gate: doc ok, seal fails to compile without the feature and compiles with it"
}

check_list() {
  local count floor target
  floor="$2"
  count="$(grep -c ': test$' "$1" || true)"
  if ((count < floor)); then
    fail "crawler-core lists $count tests with ledger-seal, below the floor of $floor"
  fi
  for target in $SEAL_TARGETS; do
    grep -q "Running tests/$target.rs" "$1" || fail "test target $target did not run"
  done
  echo "check-seal-gate: count ok, $count tests (floor $floor), all seal targets ran"
}

full_run() {
  cd "$ROOT"
  cargo metadata --locked --no-deps --format-version 1 >"$SCRATCH/metadata.json"
  check_metadata "$SCRATCH/metadata.json"
  check_sources
  cargo test --locked -p crawler-core --doc >"$SCRATCH/doc-off.txt" 2>&1 ||
    { cat "$SCRATCH/doc-off.txt"; fail "doctests without the feature failed"; }
  cargo test --locked -p crawler-core --features ledger-seal --doc >"$SCRATCH/doc-on.txt" 2>&1 ||
    { cat "$SCRATCH/doc-on.txt"; fail "doctests with the feature failed"; }
  check_doc "$SCRATCH/doc-off.txt" "$SCRATCH/doc-on.txt"
  cargo test --locked -p crawler-core --features ledger-seal --tests -- --list \
    >"$SCRATCH/list.txt" 2>&1 || { cat "$SCRATCH/list.txt"; fail "listing tests failed"; }
  check_list "$SCRATCH/list.txt" "$(tr -d '[:space:]' <"$FLOOR_FILE")"
  echo "check-seal-gate: all checks passed"
}

case "${1:-}" in
  "") full_run ;;
  check-metadata) check_metadata "$2" ;;
  check-doc) check_doc "$2" "$3" ;;
  check-list) check_list "$2" "$3" ;;
  -h | --help) usage ;;
  *) usage >&2; exit 2 ;;
esac
