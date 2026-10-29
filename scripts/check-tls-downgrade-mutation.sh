#!/usr/bin/env bash
# Runs the socket-dependent downgrade mutation and restores the guard on every exit.
set -euo pipefail
cd "$(dirname "$0")/.."
GUARD=crates/crawler-fetch/src/redirect_guard.rs
BACKUP="$(mktemp)"
# Logs of the three runs; kept after exit as evidence, and the directory is printed at the end.
LOG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tls-downgrade-mutation.XXXXXX")"
cp "$GUARD" "$BACKUP"
trap 'cp "$BACKUP" "$GUARD"; rm -f "$BACKUP"' EXIT
TEST=test_https_to_http_redirect_is_refused_without_plaintext_request
# No color: the mutated run's output is searched for the assertion message.
COMMAND=(env CARGO_TERM_COLOR=never cargo test --color never -p crawler-fetch --features test-policy --locked --test tls_redirects "$TEST" -- --exact --color never)
"${COMMAND[@]}" 2>&1 | tee "$LOG_DIR/before.log"
python3 - "$GUARD" <<'PY'
import pathlib, sys
path_guard = pathlib.Path(sys.argv[1])
str_source = path_guard.read_text()
str_original = 'if url_from.scheme() == "https" && url_to.scheme() == "http" {'
if str_source.count(str_original) != 1:
    raise SystemExit('expected exactly one downgrade branch')
path_guard.write_text(str_source.replace(str_original, str_original.replace('"http"', '"disabled-http"')))
PY
set +e
"${COMMAND[@]}" 2>&1 | tee "$LOG_DIR/mutated.log"
STATUS=${PIPESTATUS[0]}
set -e
cp "$BACKUP" "$GUARD"
if [[ "$STATUS" -ne 101 ]] || ! grep -q 'downgrade must fail' "$LOG_DIR/mutated.log"; then
  echo "mutation check: expected the downgrade assertion to fail; exit was $STATUS"
  exit 1
fi
"${COMMAND[@]}" 2>&1 | tee "$LOG_DIR/restored.log"
echo 'mutation check: original passed, disabled branch failed at the downgrade assertion, restored passed'
echo "mutation check: logs in $LOG_DIR"
