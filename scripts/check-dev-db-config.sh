#!/usr/bin/env bash
# Static checks for the dev Postgres setup: pinned image, loopback-only port, the compose subset
# that works on docker-compose 1.29 and docker compose 2.0 and newer, the PG 18 volume path,
# gitignored .env, and bash 3.2 safe scripts. Needs no Docker daemon. Bash 3.2 safe.
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Usage: scripts/check-dev-db-config.sh

Checks docker-compose.dev.yml, .env.example, .gitignore, the CI service image and the dev-db
scripts. Runs `config -q` with every compose CLI that is installed. Exits 1 on any failure.
EOF
  exit 0
fi

FILE=docker-compose.dev.yml
IMAGE=postgres:18.6-bookworm
SCRIPTS="scripts/dev-db.sh scripts/check-dev-db-persistence.sh"
FAILURES=0
SCRATCH="$(mktemp "${TMPDIR:-/tmp}/dev-db-config.XXXXXX")"
trap 'rm -f "$SCRATCH"' EXIT

pass() { echo "ok:   $*"; }
fail() { echo "FAIL: $*"; FAILURES=$((FAILURES + 1)); }
check() {
  local label="$1"
  shift
  if "$@" >/dev/null 2>&1; then pass "$label"; else fail "$label"; fi
}
# Succeeds when no non-comment line of file $2 matches pattern $1.
absent() { ! grep -v -E '^[[:space:]]*#' "$2" | grep -E -q "$1"; }

check "image pinned to $IMAGE" grep -E -q "^    image: $IMAGE\$" "$FILE"
check "no other image in $FILE" test "$(grep -E -c '^ +image:' "$FILE")" -eq 1
check "CI service uses $IMAGE" grep -F -q "image: $IMAGE" .github/workflows/ci.yml
check "port published on 127.0.0.1 only" grep -E -q '^      - "127\.0\.0\.1:\$\{DEV_DB_PORT:-55432\}:5432"$' "$FILE"
check "no 0.0.0.0 binding" absent '0\.0\.0\.0' "$FILE"
check "mem_limit set" grep -E -q '^    mem_limit: [0-9]+m$' "$FILE"
check "no deploy section" absent '^ *deploy:' "$FILE"
check "no top-level name:" absent '^name:' "$FILE"
check "no top-level version:" absent '^version:' "$FILE"
check "named volume at the PG 18 path" grep -E -q '^      - pgdata:/var/lib/postgresql$' "$FILE"
check "no pre-18 data path" absent '/var/lib/postgresql/data' "$FILE"
check "named volume declared" grep -E -q '^  pgdata:$' "$FILE"
check "durability settings marked DEV ONLY" grep -F -q '# DEV ONLY' "$FILE"
check ".env is gitignored" git check-ignore -q .env
check ".env.example is not ignored" /bin/bash -c '! git check-ignore -q .env.example'
check ".env is not tracked" test -z "$(git ls-files .env)"
for SCRIPT in $SCRIPTS; do
  check "$SCRIPT parses under /bin/bash" /bin/bash -n "$SCRIPT"
  check "$SCRIPT avoids bash 4+ and grep -P" absent 'declare -A|mapfile|readarray|grep -P|\$\{[a-zA-Z_]+(,,|\^\^)\}|\|&|&>>' "$SCRIPT"
done

# Failure messages: runs dev-db.sh up with a PATH that holds only stub commands and the basic tools
# the script needs, so a real docker on the system cannot interfere.
FAKE="$(mktemp -d "${TMPDIR:-/tmp}/dev-db-fake.XXXXXX")"
trap 'rm -f "$SCRATCH"; rm -rf "$FAKE"' EXIT
make_bin() {
  mkdir -p "$FAKE/$1"
  for TOOL in bash sed grep mktemp rm cp chmod cat dirname sleep; do
    ln -s "$(command -v "$TOOL")" "$FAKE/$1/$TOOL"
  done
}
expect_message() {
  env -u DATABASE_URL PATH="$FAKE/$1" bash scripts/dev-db.sh up >"$SCRATCH" 2>&1 && return 1
  grep -F -q "$2" "$SCRATCH"
}
make_bin no-docker
printf '#!/bin/sh\necho "docker-compose version 1.29.0, stub"\n' >"$FAKE/no-docker/docker-compose"
make_bin no-daemon
printf '#!/bin/sh\ncase "$1" in\n  compose) echo "Docker Compose version v2.0.0-stub" ;;\n  info) echo "ERROR: Cannot connect to the Docker daemon (stub)"; exit 1 ;;\n  *) exit 1 ;;\nesac\n' >"$FAKE/no-daemon/docker"
chmod +x "$FAKE/no-docker/docker-compose" "$FAKE/no-daemon/docker"
check "message when docker is missing" expect_message no-docker "the docker command is not on PATH"
check "message when the daemon is unreachable" expect_message no-daemon "the Docker daemon is not reachable"

# `config -q` validates the file without a daemon; the dummy values come from .env.example.
set -a
# shellcheck source=/dev/null
. ./.env.example
set +a
if docker compose version >/dev/null 2>&1; then
  check "docker compose config" docker compose -f "$FILE" -p proofcrawl-dev config -q
fi
if docker-compose --version >/dev/null 2>&1; then
  check "docker-compose config" docker-compose -f "$FILE" -p proofcrawl-dev config -q
fi

if ((FAILURES > 0)); then
  echo "check-dev-db-config: $FAILURES check(s) failed"
  exit 1
fi
echo "check-dev-db-config: all checks passed"
