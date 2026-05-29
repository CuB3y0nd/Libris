#!/usr/bin/env sh
set -eu

: "${SMOKE_WEBDAV_URL:?set SMOKE_WEBDAV_URL}"
: "${SMOKE_WEBDAV_USER:?set SMOKE_WEBDAV_USER}"
: "${SMOKE_WEBDAV_PASSWORD:?set SMOKE_WEBDAV_PASSWORD}"

base="${SMOKE_WEBDAV_URL%/}"
auth="${SMOKE_WEBDAV_USER}:${SMOKE_WEBDAV_PASSWORD}"
probe_dir="${base}/zotero/smoke-$(date +%s)"
probe_file="${probe_dir}/probe.txt"

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT
printf 'zotero webdav smoke test\n' > "$tmp_file"

curl -fsS -u "$auth" -X MKCOL "${base}/zotero" >/dev/null 2>&1 || true
curl -fsS -u "$auth" -X MKCOL "$probe_dir" >/dev/null
curl -fsS -u "$auth" -T "$tmp_file" "$probe_file" >/dev/null
curl -fsS -u "$auth" -X PROPFIND -H 'Depth: 1' "$probe_dir/" >/dev/null
curl -fsS -u "$auth" "$probe_file" | grep -q 'zotero webdav smoke test'
curl -fsS -u "$auth" -X DELETE "$probe_dir" >/dev/null

echo "WebDAV smoke test passed for ${SMOKE_WEBDAV_USER}."
