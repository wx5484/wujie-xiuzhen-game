#!/bin/sh
set -eu

POSTGRES_HOST="${POSTGRES_HOST:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-cq}"
POSTGRES_USER="${POSTGRES_USER:-cq}"
MIGRATIONS_DIR="${MIGRATIONS_DIR:-/migrations}"

psql_base() {
  psql -h "$POSTGRES_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" "$@"
}

psql_base -v ON_ERROR_STOP=1 \
  -c "create table if not exists schema_migrations (version text primary key, applied_at timestamptz not null default now())"

found=0
for file in "$MIGRATIONS_DIR"/*.sql; do
  if [ ! -f "$file" ]; then
    continue
  fi

  found=1
  version="${file##*/}"
  applied="$(psql_base -At -c "select 1 from schema_migrations where version = '$version'")"

  if [ "$applied" = "1" ]; then
    echo "skip $version"
  else
    echo "applying $version"
    psql_base -v ON_ERROR_STOP=1 <<SQL
begin;
\i $file
insert into schema_migrations (version) values ('$version');
commit;
SQL
  fi
done

if [ "$found" = "0" ]; then
  echo "no migration files found in $MIGRATIONS_DIR"
fi
