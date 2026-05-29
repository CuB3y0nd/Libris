#!/usr/bin/env sh
set -eu

if [ ! -f .env ]; then
  cp .env.example .env
  echo "Created .env from .env.example. Edit it before running again." >&2
  exit 1
fi

if [ ! -f config/users.csv ]; then
  cp config/users.csv.example config/users.csv
  echo "Created config/users.csv from example. Edit it before running again." >&2
  exit 1
fi

exec docker compose --env-file .env -f deploy/compose/docker-compose.yml up -d --build
