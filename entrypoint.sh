#!/bin/bash
set -e

echo "Waiting for database..."
until pg_isready -d "$DATABASE_URL" >/dev/null 2>&1; do
  sleep 1
done

echo "Database is up - running migrations"
sqlx migrate run

echo "Starting application"
exec ./player
