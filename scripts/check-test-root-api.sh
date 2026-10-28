#!/usr/bin/env bash
# Proves the custom-root API exists only in a test-policy build without opening sockets.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CASE="${1:-clean}"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
mkdir -p "$SCRATCH/src"
sed "s|@CRAWLER_FETCH@|$ROOT/crates/crawler-fetch|" \
  "$ROOT/scripts/fixtures/test-policy-probe/$CASE.Cargo.toml" > "$SCRATCH/Cargo.toml"
cp "$ROOT/Cargo.lock" "$SCRATCH/Cargo.lock"
# This names the API itself; no certificate or runtime construction is necessary.
printf 'fn main() { let _ = crawler_fetch::ClientConfig::with_test_root; }\n' > "$SCRATCH/src/main.rs"
if OUTPUT="$(CARGO_TERM_COLOR=never CARGO_TARGET_DIR="$ROOT/target/test-root-api" cargo check --color never --offline --manifest-path "$SCRATCH/Cargo.toml" 2>&1)"; then
  if [[ "$CASE" != enabled ]]; then
    echo 'check-test-root-api: production unexpectedly exposes with_test_root'
    exit 1
  fi
else
  if [[ "$CASE" != clean ]] || ! grep -q 'error\[E0599\]: no associated function or constant named `with_test_root`' <<<"$OUTPUT"; then
    echo "$OUTPUT"
    exit 1
  fi
fi
echo "check-test-root-api: $CASE has the expected custom-root API availability"
