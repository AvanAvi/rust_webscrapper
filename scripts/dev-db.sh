#!/usr/bin/env bash
# Manages the local development Postgres (docker-compose.dev.yml), or reports on an existing
# DATABASE_URL without starting anything. Runs on macOS bash 3.2: no associative arrays, no
# mapfile, no grep -P, and no empty arrays under `set -u`. Never prints passwords or full URLs.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMPOSE_FILE="$ROOT/docker-compose.dev.yml"
PROJECT=proofcrawl-dev
SERVICE=postgres
TIMEOUT="${DEV_DB_TIMEOUT:-90}"
SCRATCH="$(mktemp "${TMPDIR:-/tmp}/dev-db.XXXXXX")"
trap 'rm -f "$SCRATCH"' EXIT
trap 'exit 130' INT TERM

usage() {
  cat <<'EOF'
Usage: scripts/dev-db.sh <command> [--yes]

Local development Postgres (postgres:18.6-bookworm on 127.0.0.1) for proofcrawl.

Commands:
  up       Start the container and wait until Postgres accepts connections.
  down     Stop and remove the container; the data volume is kept.
  status   Report whether the database is running and ready (exit 3 if not running).
  reset    Delete the container AND its data volume, then start fresh. Needs --yes.
  url      Show host, port and database (never the password or the full URL).
  psql     Run psql inside the container (no host psql needed); extra arguments pass through.
  --help   Show this help.

If DATABASE_URL is already set, the script uses that database and never starts, stops or
deletes a container: up and status only check that its host and port accept TCP connections.

Environment:
  DATABASE_URL        An existing Postgres to use instead of the container.
  DEV_DB_TIMEOUT      Readiness timeout in seconds (default 90).
  DEV_DB_CONTAINER    With DATABASE_URL: a container to run pg_isready in (used by CI).

Credentials come from .env (gitignored), created from .env.example on first use.
See docs/DEVELOPMENT.md for setup on older Macs, troubleshooting and fallbacks.
EOF
}

say() { echo "dev-db: $*"; }
die() { echo "dev-db: $*" >&2; exit 1; }

fallbacks() {
  echo "dev-db: fallbacks: install a native Postgres (Postgres.app) or point DATABASE_URL at any" >&2
  echo "dev-db: reachable Postgres; see docs/DEVELOPMENT.md." >&2
}

# Picks `docker compose` (v2) or `docker-compose` (v1). Only the exit status and the version line
# count: Docker CLI warnings such as "Plugin ... is not valid" on stderr are ignored.
detect_compose() {
  local version
  if version="$(docker compose version 2>"$SCRATCH")"; then
    COMPOSE=(docker compose)
  elif version="$(docker-compose --version 2>"$SCRATCH")"; then
    COMPOSE=(docker-compose)
  else
    echo "dev-db: neither 'docker compose' nor 'docker-compose' is available." >&2
    fallbacks
    exit 1
  fi
  version="$(printf '%s\n' "$version" | sed -n '/[Vv]ersion/{p;q;}')"
  say "using '${COMPOSE[*]}' (${version:-version unknown})" >&2
}

compose() {
  "${COMPOSE[@]}" -f "$COMPOSE_FILE" -p "$PROJECT" "$@"
}

# Exports the known keys from .env; values already set in the shell win, as in compose.
load_env() {
  local file="$ROOT/.env" line key
  if [[ ! -f "$file" ]]; then
    cp "$ROOT/.env.example" "$file"
    chmod 600 "$file"
    say "created .env from .env.example (dev-only credentials)"
  fi
  while IFS= read -r line || [[ -n "$line" ]]; do
    key="${line%%=*}"
    case "$key" in
      POSTGRES_USER | POSTGRES_PASSWORD | POSTGRES_DB | DEV_DB_PORT)
        if [[ -z "${!key:-}" ]]; then export "$key=${line#*=}"; fi ;;
    esac
  done <"$file"
  DEV_DB_PORT="${DEV_DB_PORT:-55432}"
  export DEV_DB_PORT
}

# Both compose forms need the docker command itself: for `docker info` and `docker inspect`.
require_daemon() {
  if ! command -v docker >/dev/null 2>&1; then
    echo "dev-db: the docker command is not on PATH." >&2
    fallbacks
    exit 1
  fi
  if docker info >"$SCRATCH" 2>&1; then
    return 0
  fi
  echo "dev-db: the Docker daemon is not reachable. Start Docker Desktop and retry:" >&2
  grep -i -E 'error|cannot' "$SCRATCH" | sed 's/^/dev-db:   /' >&2 || true
  fallbacks
  exit 1
}

# Splits DATABASE_URL into URL_HOST, URL_PORT and URL_DB; the credentials are dropped unread.
parse_database_url() {
  local rest="${DATABASE_URL#*://}" hostport
  rest="${rest##*@}"
  hostport="${rest%%/*}"
  URL_DB=""
  if [[ "$rest" == */* ]]; then URL_DB="${rest#*/}"; fi
  URL_DB="${URL_DB%%\?*}"
  URL_HOST="${hostport%:*}"
  URL_PORT="${hostport##*:}"
  if [[ "$hostport" != *:* || "$hostport" == *']' ]]; then URL_HOST="$hostport"; URL_PORT=""; fi
  URL_HOST="${URL_HOST:-localhost}"
  URL_PORT="${URL_PORT:-5432}"
}

# Returns 0 when host:port accepts a TCP connection within $3 seconds (bash /dev/tcp, no tools).
probe_tcp() {
  local host="${1#[}" pid deadline=$((SECONDS + $3))
  host="${host%]}"
  (exec 3<>"/dev/tcp/$host/$2") 2>/dev/null &
  pid=$!
  while kill -0 "$pid" 2>/dev/null; do
    if ((SECONDS >= deadline)); then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
      return 1
    fi
    sleep 1
  done
  wait "$pid"
}

external_status() {
  parse_database_url
  say "DATABASE_URL is set: host=$URL_HOST port=$URL_PORT database=${URL_DB:-<default>}; no container is managed"
  if [[ -n "${DEV_DB_CONTAINER:-}" ]]; then
    wait_ready docker exec "$DEV_DB_CONTAINER" pg_isready -h 127.0.0.1 -p 5432 -q
    say "ready: pg_isready succeeded inside container ${DEV_DB_CONTAINER:0:12}"
  elif probe_tcp "$URL_HOST" "$URL_PORT" "$TIMEOUT"; then
    say "reachable: $URL_HOST:$URL_PORT accepts TCP connections"
  else
    die "not reachable: $URL_HOST:$URL_PORT did not accept a TCP connection within ${TIMEOUT}s"
  fi
}

# Runs the given readiness command until it succeeds or DEV_DB_TIMEOUT seconds pass.
wait_ready() {
  local deadline=$((SECONDS + TIMEOUT))
  until "$@" >/dev/null 2>&1; do
    if ((SECONDS >= deadline)); then
      echo "dev-db: Postgres was not ready after ${TIMEOUT}s" >&2
      return 1
    fi
    sleep 1
  done
}

# pg_isready inside the container over TCP, so the init-time socket-only server does not count.
container_ready() {
  compose exec -T "$SERVICE" pg_isready -h 127.0.0.1 -p 5432 -U "$POSTGRES_USER" -d "$POSTGRES_DB" -q
}

is_running() {
  local id
  id="$(compose ps -q "$SERVICE" 2>/dev/null || true)"
  [[ -n "$id" && "$(docker inspect -f '{{.State.Running}}' "$id" 2>/dev/null)" == "true" ]]
}

describe() {
  say "$1: host=127.0.0.1 port=$DEV_DB_PORT database=$POSTGRES_DB"
}

prepare() {
  detect_compose
  load_env
  require_daemon
}

cmd_up() {
  if [[ -n "${DATABASE_URL:-}" ]]; then external_status; return; fi
  prepare
  compose up -d "$SERVICE"
  if ! wait_ready container_ready; then
    compose logs --tail 20 "$SERVICE" >&2 || true
    exit 1
  fi
  describe "ready"
}

cmd_down() {
  if [[ -n "${DATABASE_URL:-}" ]]; then say "DATABASE_URL is set; no container to stop"; return; fi
  prepare
  compose down
  say "stopped; the data volume is kept (use 'reset --yes' to delete it)"
}

cmd_status() {
  if [[ -n "${DATABASE_URL:-}" ]]; then external_status; return; fi
  prepare
  if ! is_running; then
    say "not running"
    exit 3
  fi
  wait_ready container_ready
  describe "running and ready"
}

cmd_reset() {
  if [[ -n "${DATABASE_URL:-}" ]]; then die "DATABASE_URL is set; refusing to reset a database this script does not own"; fi
  if [[ "${1:-}" != "--yes" ]]; then die "reset deletes the dev data volume; run 'scripts/dev-db.sh reset --yes'"; fi
  prepare
  compose down -v
  say "container and data volume removed"
  cmd_up
}

cmd_url() {
  if [[ -n "${DATABASE_URL:-}" ]]; then
    parse_database_url
    say "DATABASE_URL: host=$URL_HOST port=$URL_PORT database=${URL_DB:-<default>}"
    return
  fi
  load_env
  describe "dev database"
  say "user and password are in .env; docs/DEVELOPMENT.md shows how to export DATABASE_URL"
}

cmd_psql() {
  if [[ -n "${DATABASE_URL:-}" ]]; then die "DATABASE_URL is set; use your own client for that database"; fi
  prepare
  if [[ -t 0 ]]; then
    compose exec "$SERVICE" psql -X -U "$POSTGRES_USER" -d "$POSTGRES_DB" "$@"
  else
    compose exec -T "$SERVICE" psql -X -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" "$@"
  fi
}

case "${1:-}" in
  up) cmd_up ;;
  down) cmd_down ;;
  status) cmd_status ;;
  reset) cmd_reset "${2:-}" ;;
  url) cmd_url ;;
  psql) shift; cmd_psql "$@" ;;
  -h | --help | help) usage ;;
  *) usage >&2; exit 2 ;;
esac
