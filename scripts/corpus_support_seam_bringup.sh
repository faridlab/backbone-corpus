#!/usr/bin/env bash
# Provision a Postgres with BOTH corpus and backbone-support schemas migrated, then run the
# corpus↔support seam test (CSEAM-1). CSEAM-1 is environmental, not a corpus defect: it imports
# backbone-support (dev-only) and writes to real support.issues / support.service_level_agreements.
# The two modules' migrations do not collide in a shared DB — enums have distinct names (all
# IF NOT EXISTS), tables are schema-isolated (corpus.* vs support.*), and audit triggers are
# schema-scoped CREATE OR REPLACE functions.
#
# Container lifecycle: reuse-if-running, leave up. The DB is dropped/recreated each run so the
# plain CREATE TABLE migrations stay idempotent across repeat runs on a warm container.
set -euo pipefail
cd "$(dirname "$0")/.."

CONTAINER=backbone-corpus-pg
PGUSER=postgres
DB=backbone_corpus
PGPORT=5433
export DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:${PGPORT}/${DB}}"

# 1. Ensure a Postgres is reachable on :5433 — reuse if up, else start one and leave it running.
if ! pg_isready -h localhost -p "$PGPORT" >/dev/null 2>&1; then
  echo "== starting Postgres ($CONTAINER on :$PGPORT) =="
  docker run -d --name "$CONTAINER" \
    -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB="$DB" \
    -p "$PGPORT":5432 postgres:16-alpine >/dev/null
  echo -n "== waiting for Postgres "
  for _ in $(seq 1 30); do
    if pg_isready -h localhost -p "$PGPORT" >/dev/null 2>&1; then echo "ready"; break; fi
    echo -n "."; sleep 1
  done
  pg_isready -h localhost -p "$PGPORT" >/dev/null 2>&1 || { echo " FAILED"; exit 1; }
else
  echo "== reusing Postgres on :$PGPORT =="
fi

psql() { docker exec -i "$CONTAINER" psql -U "$PGUSER" -d "$DB" -v ON_ERROR_STOP=1 "$@"; }

# 2. Reset the DB to a clean state (idempotent across repeat runs on a warm container).
echo "== resetting database $DB =="
docker exec -i "$CONTAINER" psql -U "$PGUSER" -d postgres -v ON_ERROR_STOP=1 <<'SQL'
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'backbone_corpus';
DROP DATABASE IF EXISTS backbone_corpus;
CREATE DATABASE backbone_corpus;
SQL

# 3. Apply corpus migrations, then backbone-support migrations. Schemas are independent;
#    applying support after corpus is deterministic, not a dependency.
apply_dir() {
  local label="$1" dir="$2"
  echo "== applying $label migrations =="
  for f in $(ls "$dir"/*.up.sql 2>/dev/null | sort); do
    psql < "$f" >/dev/null
  done
}
apply_dir corpus migrations
apply_dir support ../backbone-support/migrations

# 4. Run the seam test.
echo "== running CSEAM-1 (corpus↔support seam) =="
cargo test --test corpus_support_seam 2>&1 | grep -E "test result|running [0-9]+ test|FAILED|panicked" || true
echo "== bring-up complete (container $CONTAINER left running) =="
