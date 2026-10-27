#!/usr/bin/env bash
# Fails if the crawlerd binary's dependency graph enables crawler-fetch's `test-policy`
# feature (the loopback-only address policy used by tests). `cargo tree -i crawler-fetch
# -e features` lists the features crawler-fetch gets in crawlerd's graph.
set -euo pipefail
cd "$(dirname "$0")/.."
if ! OUTPUT="$(cargo tree --locked -p crawler-cli -e normal,features -i crawler-fetch 2>&1)"; then
  if grep -q "did not match any packages" <<<"$OUTPUT"; then
    echo "check-no-test-policy: crawlerd does not depend on crawler-fetch (yet)"
    exit 0
  fi
  echo "$OUTPUT"
  exit 1
fi
if grep -q 'feature "test-policy"' <<<"$OUTPUT"; then
  echo "crawlerd enables crawler-fetch/test-policy; it must not"
  exit 1
fi
echo "check-no-test-policy: crawlerd does not enable test-policy"
