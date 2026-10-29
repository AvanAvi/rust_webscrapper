#!/usr/bin/env bash
# Fails if crawler-fetch's normal (production) dependency graph contains axum. axum is only
# for mock-site; production code uses the `http` crate for request and response types.
set -euo pipefail
cd "$(dirname "$0")/.."
# No color: escape codes in the tree output would sit between the words this grep matches.
TREE="$(CARGO_TERM_COLOR=never cargo tree --color never -p crawler-fetch -e normal --locked)"
if grep -Eq '(^|[^a-z_-])axum(-core)? v' <<<"$TREE"; then
  echo "check-fetch-deps: axum is in crawler-fetch's normal dependencies:"
  grep -E 'axum' <<<"$TREE"
  exit 1
fi
echo "check-fetch-deps: crawler-fetch has no axum in its normal dependencies"
