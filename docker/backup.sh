#!/usr/bin/env sh
set -eu

: "${DATABASE_URL:?DATABASE_URL is required}"
: "${BACKUP_DIR:=/app/data/backup}"

mkdir -p "$BACKUP_DIR"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
pg_dump "$DATABASE_URL" --format=custom --file="$BACKUP_DIR/cq-$stamp.dump"
find "$BACKUP_DIR" -name 'cq-*.dump' -mtime +14 -delete
