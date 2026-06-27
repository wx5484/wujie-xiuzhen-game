#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

IMAGE_NAME="${IMAGE_NAME:-cq-rs:latest}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.prod.yml}"
DOMAIN="${DOMAIN:-}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    return 1
  fi
}

rand_secret() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -base64 36 | tr -d '\n'
  else
    date +%s%N | sha256sum | awk '{print $1}'
  fi
}

if [ "$(uname -s)" != "Linux" ]; then
  echo "This deploy script is intended for Ubuntu/Linux servers." >&2
  exit 1
fi

if [ -r /etc/os-release ]; then
  . /etc/os-release
  if [ "${ID:-}" != "ubuntu" ]; then
    echo "Warning: detected ${PRETTY_NAME:-unknown Linux}; continuing anyway." >&2
  elif [ "${VERSION_ID:-}" != "24.04" ]; then
    echo "Warning: detected Ubuntu ${VERSION_ID}; this script is tuned for Ubuntu 24.04." >&2
  fi
fi

if ! need_cmd docker; then
  echo "Docker not found. Installing Docker Engine with the official convenience script..."
  sudo apt-get update
  sudo apt-get install -y ca-certificates curl git openssl
  curl -fsSL https://get.docker.com | sudo sh
fi

if ! docker compose version >/dev/null 2>&1; then
  echo "docker compose plugin is required but was not found." >&2
  exit 1
fi

if [ ! -f .env ]; then
  cp .env.example .env
  POSTGRES_PASSWORD="$(rand_secret)"
  ADMIN_PASSWORD="$(rand_secret)"
  sed -i "s#POSTGRES_PASSWORD=.*#POSTGRES_PASSWORD=${POSTGRES_PASSWORD}#g" .env
  sed -i "s#ADMIN_BOOTSTRAP_PASSWORD=.*#ADMIN_BOOTSTRAP_PASSWORD=${ADMIN_PASSWORD}#g" .env
  sed -i "s#DATABASE_URL=.*#DATABASE_URL=postgres://cq:${POSTGRES_PASSWORD}@postgres:5432/cq#g" .env
  sed -i "s#APP_ENV=.*#APP_ENV=production#g" .env
  sed -i "s#PUBLIC_DIR=.*#PUBLIC_DIR=/app/public#g" .env
  sed -i "s#BACKUP_DIR=.*#BACKUP_DIR=/app/data/backup#g" .env
  echo "Created .env with generated POSTGRES_PASSWORD and ADMIN_BOOTSTRAP_PASSWORD."
fi

if [ -n "$DOMAIN" ]; then
  sed -i "1s#.*#${DOMAIN} {#g" docker/Caddyfile
  sed -i "s#PUBLIC_BASE_URL=.*#PUBLIC_BASE_URL=https://${DOMAIN}#g" .env
  echo "Configured Caddy and PUBLIC_BASE_URL for https://${DOMAIN}."
else
  echo "DOMAIN is not set. docker/Caddyfile currently decides the public host."
  echo "Example: DOMAIN=game.example.com bash scripts/deploy-ubuntu24.sh"
fi

echo "Building ${IMAGE_NAME}..."
docker build -t "$IMAGE_NAME" .

echo "Starting production stack..."
export SERVER_IMAGE="$IMAGE_NAME"
docker compose -f "$COMPOSE_FILE" up -d

echo "Current containers:"
docker compose -f "$COMPOSE_FILE" ps

echo "Deployment command finished. Check logs with:"
echo "  docker compose -f ${COMPOSE_FILE} logs -f migrate server caddy"
