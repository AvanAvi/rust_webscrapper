#!/usr/bin/env bash
# Fails if a binary's dependency graph enables crawler-fetch's `test-policy` feature (the
# loopback-only address policy, extra TLS trust root and transport diagnostics used by tests).
#
# How features are read: `cargo tree -i crawler-fetch -e normal,features` prints, for the
# given package, every feature of crawler-fetch that is enabled anywhere in its normal
# dependency graph (directly or through intermediate crates) as a line such as
#   crawler-fetch feature "test-policy"
#
# Usage: check-no-test-policy.sh [MANIFEST_PATH PACKAGE]
#   defaults: this repository's Cargo.toml and crawler-cli (the crawlerd binary), with --locked.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="${1:-$ROOT/Cargo.toml}"
PACKAGE="${2:-crawler-cli}"
# Also prove the production configuration API cannot accept a custom root.
if [[ $# -eq 0 ]]; then
  "$ROOT/scripts/check-test-root-api.sh" clean
fi
LOCK_FLAG=()
if [[ $# -eq 0 ]]; then LOCK_FLAG=(--locked); fi

if ! OUTPUT="$(cargo tree ${LOCK_FLAG[@]+"${LOCK_FLAG[@]}"} --manifest-path "$MANIFEST" -p "$PACKAGE" -e normal,features -i crawler-fetch 2>&1)"; then
  if grep -q "did not match any packages" <<<"$OUTPUT"; then
    echo "check-no-test-policy: $PACKAGE does not depend on crawler-fetch"
    exit 0
  fi
  echo "$OUTPUT"
  exit 1
fi
if grep -q 'crawler-fetch feature "test-policy"' <<<"$OUTPUT"; then
  echo "check-no-test-policy: $PACKAGE enables crawler-fetch/test-policy; it must not"
  exit 1
fi
echo "check-no-test-policy: $PACKAGE does not enable test-policy"
