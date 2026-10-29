#!/usr/bin/env bash
# dev-db persistence test: writes a row, runs `down` then `up`, and checks the row survived. This
# proves the data volume is mounted at the PostgreSQL 18 path /var/lib/postgresql. Bash 3.2 safe.
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Usage: scripts/check-dev-db-persistence.sh

Starts the dev database (scripts/dev-db.sh up), writes a marker row, stops it (down), starts it
again (up) and fails unless the row is still there. Drops its table on exit. Needs a running
Docker daemon and DATABASE_URL unset, because it tests the dev container's own volume.
EOF
  exit 0
fi
if [[ -n "${DATABASE_URL:-}" ]]; then
  echo "persistence: DATABASE_URL is set; unset it, this test checks the dev container's volume" >&2
  exit 1
fi

DEV_DB="$(pwd)/scripts/dev-db.sh"
TABLE=dev_db_persistence_check
MARKER="row-$$-$RANDOM"
cleanup() {
  "$DEV_DB" psql -q -c "DROP TABLE IF EXISTS $TABLE" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$DEV_DB" up
"$DEV_DB" psql -q -c "CREATE TABLE IF NOT EXISTS $TABLE (marker text PRIMARY KEY)" \
  -c "INSERT INTO $TABLE VALUES ('$MARKER')"
echo "persistence: wrote marker $MARKER"
"$DEV_DB" down
"$DEV_DB" up
FOUND="$("$DEV_DB" psql -t -A -c "SELECT count(*) FROM $TABLE WHERE marker = '$MARKER'")"
if [[ "$FOUND" != "1" ]]; then
  echo "persistence: FAILED, marker $MARKER not found after down/up (count: '$FOUND')" >&2
  exit 1
fi
echo "persistence: marker $MARKER survived down/up; the volume at /var/lib/postgresql persists"
