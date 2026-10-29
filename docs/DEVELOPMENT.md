# Development database

proofcrawl's Postgres-backed parts (F6 onward) read **`DATABASE_URL`**. Any reachable Postgres 18 works. Docker is optional: `scripts/dev-db.sh` is only a convenience for starting a throwaway local database. This guide covers setup on an older Intel Mac, troubleshooting, and the fallbacks.

## Quick start (Docker)

```sh
scripts/dev-db.sh up        # start and wait until ready (creates .env from .env.example on first use)
scripts/dev-db.sh status    # running and ready? exit 3 if not running
scripts/dev-db.sh url       # host, port and database (never the password)
scripts/dev-db.sh psql      # psql inside the container; no host psql needed
scripts/dev-db.sh down      # stop; data is kept
scripts/dev-db.sh reset --yes   # delete the container AND the data volume, then start fresh
scripts/check-dev-db-persistence.sh   # proves data survives down/up
scripts/check-dev-db-config.sh        # static checks, no daemon needed
```

The database is `postgres:18.6-bookworm`, published on `127.0.0.1:55432` only, with a 512 MiB memory limit and a named volume (`proofcrawl-dev_pgdata`).

To point F6 code and tests at it without printing the password, export `DATABASE_URL` from `.env` in your shell:

```sh
set -a; . ./.env; set +a
export DATABASE_URL="postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@127.0.0.1:$DEV_DB_PORT/$POSTGRES_DB"
```

`scripts/dev-db.sh` itself only looks at `DATABASE_URL` in the environment. When it is set, `up` and `status` check that host and port accept TCP connections, `down` does nothing, `reset` and `psql` refuse, and no container is ever started.

## Credentials

`.env.example` (committed) holds throwaway dev values. `.env` (gitignored, mode 600) is created from it on first use, and you may change it. Neither may ever hold a real secret. Values already exported in your shell take precedence over `.env`, as in compose. The scripts print host, port and database only, never the password or a full connection string.

## Setup on an older Mac

These notes are written for older Intel Macs with 8 GiB RAM.

- **Docker Desktop.** Current Docker Desktop releases support only the current and two previous major macOS releases; since 4.49.0 the minimum is macOS 14 (Sonoma). Older Macs that cannot upgrade macOS need an older Docker Desktop release, which may no longer be downloadable from Docker. Sources: https://docs.docker.com/desktop/setup/install/mac-install/ and https://docs.docker.com/desktop/release-notes/
- **Docker Engine.** The image needs Docker Engine 20.10.10 or newer, the release that added `clone3` to the default seccomp profile for current Debian-based images. Source: https://github.com/moby/moby/releases/tag/v20.10.10
- **Mixed installs.** If you have upgraded Docker Desktop in place or installed it twice, the `docker compose` plugin symlink in `~/.docker/cli-plugins` may still point at an older install. It can then report an older compose version than the `docker-compose` on your PATH. `dev-db.sh` prints which one it chose and its version on every run, and the compose file works with both.
- **Required commands.** `dev-db.sh` needs the `docker` command on PATH plus either `docker compose` (v2 plugin) or `docker-compose` (v1). Without `docker` it stops with "the docker command is not on PATH"; with `docker` but no running daemon it stops with "the Docker daemon is not reachable".
- **Starting the daemon.** Start Docker Desktop from Applications and wait until `docker info` shows a `Server:` section. `dev-db.sh` never starts the daemon itself.
- **Memory.** Docker Desktop runs a Linux VM whose memory size depends on the release and settings; `docker info` shows it as `Total Memory`, and some older releases have no setting to lower it. Postgres inside it is limited to 512 MiB (`mem_limit`) with `shared_buffers=128MB`. Close memory-heavy apps on an 8 GiB machine, and run `scripts/dev-db.sh down` when you are done.
- **Warnings.** With mixed installs the Docker CLI may print `Plugin "…" is not valid` warnings, for plugin symlinks whose targets an older install doesn't ship. They are harmless, and `dev-db.sh` ignores them.

## Compose compatibility rules

`docker-compose.dev.yml` must work with older Compose versions too, down to `docker-compose` 1.29 (v1) and `docker compose` 2.0. Some newer forms are therefore avoided:

| Avoided | Why | Used instead |
|---|---|---|
| top-level `name:` | not in the schema of either version (rejected) | `-p proofcrawl-dev` passed by the script |
| top-level `version:` | optional in both, and obsolete in the Compose Spec | omitted |
| `deploy.resources.limits` | v1 ignores it without the deprecated `--compatibility` flag | `mem_limit: 512m` |
| `/var/lib/postgresql/data` mount | the pre-18 path: PostgreSQL 18 images keep data in `/var/lib/postgresql/18/docker`, so mounting the old path loses data on re-create | `pgdata:/var/lib/postgresql` |

`scripts/check-dev-db-config.sh` enforces these rules. It also runs `config -q` with every compose CLI that is installed.

## Dev-only settings

The compose file starts Postgres with `fsync=off`, `synchronous_commit=off` and `full_page_writes=off` to make tests faster on a slow laptop. These settings are **for development only**. A crash of the Docker VM can corrupt the cluster, and the fix is `scripts/dev-db.sh reset --yes`. Never copy them into any configuration that holds real ledger data.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| `the Docker daemon is not reachable … Cannot connect to the Docker daemon` | Docker Desktop is not running. Start it, or use a fallback below. |
| `neither 'docker compose' nor 'docker-compose' is available` | No compose CLI on PATH. Use a fallback below. |
| `Postgres was not ready after 90s` | The first start initialises the cluster, which is slow on old hardware. The script prints the last 20 log lines. Retry with `DEV_DB_TIMEOUT=180`. |
| `port is already allocated` / `address already in use` | Another service holds 55432. Set `DEV_DB_PORT` in `.env`. |
| Data gone after `down` and `up` | The volume path is wrong (see the rules above). `scripts/check-dev-db-persistence.sh` detects this. |
| Data gone after a Docker crash | Expected with the dev-only settings. Run `reset --yes`. |
| `DATABASE_URL is set …` when you wanted the container | `unset DATABASE_URL`. |

## Fallbacks without Docker

Every path ends in a `DATABASE_URL`, so F6 code and tests don't care which one you use.

1. **Native Postgres (Postgres.app).** Version 2.9.6 is a Universal build that requires macOS 10.15, so it runs on an older Intel Mac with no VM. That makes it the lightest option on 8 GiB. Use PostgreSQL 18, create a database and user, and export `DATABASE_URL` pointing at `127.0.0.1:5432`. Source: https://postgresapp.com/downloads.html. A Homebrew install is not recommended on Intel: Homebrew treats Intel macOS as Tier 3, with few prebuilt bottles, so it would compile from source. Source: https://docs.brew.sh/Support-Tiers
2. **Any reachable Postgres 18.** Point `DATABASE_URL` at it. `scripts/dev-db.sh status` confirms the host and port accept connections.
3. **Colima or Podman.** Possible, but on older Intel macOS they need Homebrew Tier 3 source builds, and Lima (under Colima) uses QEMU instead of Apple's virtualization before macOS 13.5. They are not recommended on older Intel Macs. Source: https://lima-vm.io/docs/config/vmtype/

## CI

The `dev-db` job in `.github/workflows/ci.yml` runs on ubuntu-24.04:
- it starts a `postgres:18.6-bookworm` service container on `127.0.0.1:55432`;
- it runs `check-dev-db-config.sh`;
- it runs the readiness check (`DATABASE_URL` set, `pg_isready` inside the service container via `DEV_DB_CONTAINER`);
- it checks that `up` with `DATABASE_URL` set starts no container;
- it runs the persistence test with compose on port 55433.

CI runs a modern Docker, so it does not prove compatibility with older Docker and Compose versions; check those locally.
