#!/bin/sh
set -eu

ENV_ADMIN_TOKEN=""
if [ -f .env ]; then
  ENV_ADMIN_TOKEN="$(awk -F= '/^ADMIN_BOOTSTRAP_PASSWORD=/{print substr($0, index($0, "=") + 1)}' .env | tail -n 1 | tr -d '\r')"
fi
ADMIN_TOKEN="${ADMIN_TOKEN:-${ENV_ADMIN_TOKEN:-change_me}}"

echo "== cq-rs CNB verify =="

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required" >&2
  exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
  echo "docker compose is required" >&2
  exit 1
fi

echo "1 stop old stack"
docker compose down

echo "2 build and start"
docker compose up -d --build

echo "3 compose status"
docker compose ps

echo "4 recent migrate logs"
docker compose logs --tail=200 migrate

echo "5 recent server logs"
docker compose logs --tail=200 server

echo "6 wait for readiness"
ready=0
for i in $(seq 1 60); do
  if curl -fsS http://127.0.0.1:3000/api/healthz >/dev/null 2>&1 \
    && curl -fsS http://127.0.0.1:3000/api/readyz >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 2
done

if [ "$ready" != "1" ]; then
  echo "server did not become ready" >&2
  docker compose ps >&2
  docker compose logs --tail=200 migrate >&2
  docker compose logs --tail=200 server >&2
  exit 1
fi

echo "7 full smoke"
ADMIN_TOKEN="$ADMIN_TOKEN" sh scripts/smoke.sh

echo "verify ok"
