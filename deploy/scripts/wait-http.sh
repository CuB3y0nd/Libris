#!/usr/bin/env sh
set -eu

url="${1:?usage: wait-http.sh URL [max_attempts]}"
max_attempts="${2:-60}"
attempt=1

while [ "$attempt" -le "$max_attempts" ]; do
  if curl -fsS "$url" >/dev/null 2>&1; then
    exit 0
  fi
  attempt=$((attempt + 1))
  sleep 1
done

echo "Timed out waiting for $url" >&2
exit 1
