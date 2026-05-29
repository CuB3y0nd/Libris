#!/usr/bin/env sh
set -eu
exec docker compose --env-file .env -f deploy/compose/docker-compose.yml down "$@"
