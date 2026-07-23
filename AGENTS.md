# AGENTS.md — chungusdb

## Overview

Player statistics service. Ingests match stats via gRPC and provides REST API for queries. Stores cumulative and per-match player data in PostgreSQL.

- **Language**: Rust (Axum + Tonic + SQLx)
- **Ports**: 3000 (HTTP), 50052 (gRPC)
- **Database**: PostgreSQL 18
- **Status**: Active

## gRPC API (port 50052)

### `RecordMatchStats` RPC
- **Input**: `RecordMatchStatsRequest { player_stats: map<string, Stats> }` (chungid → stats)
- **Stats**: `{ name, frags, deaths, accuracy, elo }`
- **Response**: `RecordMatchStatsResponse { message }`
- Called by chungusway after each match intermission

## HTTP API (port 3000)

| Endpoint | Method | Status |
|----------|--------|--------|
| `/` | GET | Active (health check) |
| `/player` | GET | Commented out |
| `/player/{id}` | GET | Commented out |
| `/players/batch` | POST | Commented out |

JWT middleware exists but no routes currently use it.

## Database Schema

**`players`** — cumulative stats per player
- `chungid` (UUID PK), `name`, `frags`, `deaths`, `accuracy` (weighted avg), `matches_played`, `elo`, `commendations`, `created_at`, `updated_at`

**`matches`** — one row per game session
- `id` (UUID PK), `created_at`

**`match_participants`** — per-player per-match snapshot
- `id` (UUID PK), `match_id` (FK), `chungid` (FK), `name`, `frags`, `deaths`, `accuracy`, `elo`

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `DATABASE_URL` | PostgreSQL connection string | — (required) |
| `SECRET_CHUNGUS` | JWT signing secret | `""` |
| `LOG_LEVEL` | Logging verbosity | `INFO` |
| `LOG_FILE` | Optional log file path | stderr only |

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point: spawns HTTP + gRPC servers |
| `src/db.rs` | PostgreSQL operations, `process_match_stats` transaction |
| `src/chungusdb_grpc_service.rs` | gRPC service implementation |
| `src/models.rs` | `Player`, `IncomingPlayer`, `Match` structs |
| `src/handlers.rs` | HTTP handlers (currently disabled) |
| `src/middleware.rs` | JWT auth middleware |
| `src/logger.rs` | Fern logger setup |
| `proto/chungusway_chungusdb.proto` | gRPC service definition |
| `migrations/20251215000001_init.sql` | Schema + seed data |
| `compose.yaml` | PostgreSQL for local dev |
| `entrypoint.sh` | Docker entrypoint: waits on `$DATABASE_URL` (pg_isready), runs migrations, execs `./player` |
| `.env.example` | Environment template (copy to `.env`) |

## Development

```bash
cargo build
cargo run                     # binds 0.0.0.0:3000 + 0.0.0.0:50052
SQLX_OFFLINE=true cargo build # build without live DB (uses .sqlx/ metadata)
sqlx migrate run              # run migrations
cargo sqlx prepare            # regenerate .sqlx/ after changing queries or schema (needs live DB + DATABASE_URL)
cargo fmt && cargo clippy
```

## Architecture Notes

- **Upsert logic**: `process_match_stats` runs in a transaction — inserts match, upserts players (cumulative frags/deaths, weighted avg accuracy, increments matches_played, refreshes `name` to the last seen in-game name, sets `updated_at` to the current transaction time), inserts match_participants. Identity is `chungid` (UUID); `players.name` is a mutable display attribute (Steam persona-name model), while `match_participants.name` is a frozen per-match snapshot.
- **SQLx offline mode**: `.sqlx/` contains pre-checked query metadata for Docker/CI builds. It must be regenerated (`cargo sqlx prepare` against a live, migrated DB) whenever queries or schema change — a stale cache breaks the Docker build.
- **Docker**: image builds and runs standalone — `ENTRYPOINT` is `entrypoint.sh` (waits for the DB, runs migrations, starts the service). Builder stage is `rust:1.94` (sqlx-cli 0.9 requires rustc ≥1.94).
- **Logging**: Fern writes `[UTC timestamp][Rust target][level]` to stderr and optionally to `LOG_FILE`. `LOG_LEVEL` defaults to `INFO`. gRPC ingestion logs `event=match_stats_*` summaries and one `event=player_stats_accepted` line per valid `chungid`; authentication logs outcomes and subjects, never bearer tokens or `SECRET_CHUNGUS`.
- No tests, no graceful shutdown
