# Third-party Components

## Runtime containers

| Component | Purpose | Notes |
|---|---|---|
| SFTPGo Community | WebDAV server and S3-backed storage adapter | Community edition is AGPLv3. Verify distribution obligations. |
| Caddy | TLS termination and reverse proxy | Public edge; preserve Host semantics for WebDAV. |

## Rust crates

| Crate | Purpose | Rationale |
|---|---|---|
| serde | JSON serialization/deserialization | Stable baseline for typed SFTPGo API contracts. |
| serde_json | JSON payload construction/parsing | Avoids unsafe hand-written JSON for secret-bearing control-plane payloads. |

## Test-only optional components

| Component | Purpose | Notes |
|---|---|---|
| MinIO | Local S3-compatible test double | Not required for production. Use only for E2E tests when AWS credentials are unavailable. |
