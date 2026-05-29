# Architecture

## Problem statement

We need a one-click deployment framework for multiple Zotero users who want personal-library attachment sync via WebDAV, while storing the actual objects in AWS S3.

The core impedance mismatch is:

```text
Zotero expects WebDAV filesystem semantics.
AWS S3 provides object-key semantics.
```

SFTPGo is selected as the protocol bridge. Rust is used only for deterministic provisioning and validation.

## System context

```text
+------------------+       +--------+       +----------------+       +--------+
| Zotero Desktop   | HTTPS | Caddy  | HTTP  | SFTPGo WebDAV  | S3 API| AWS S3 |
| personal library +------>+ TLS    +------>+ per-user FS    +------>+ bucket |
+------------------+       +--------+       +----------------+       +--------+
                                          ^
                                          |
                                   +------+------+
                                   | provisioner |
                                   | Rust CLI    |
                                   +-------------+
```

## Component responsibilities

### Caddy

- Owns public TLS.
- Routes one dedicated WebDAV virtual host to SFTPGo.
- Preserves request path and Host semantics.
- Should not expose the SFTPGo Admin API without a private network or extra auth layer.

### SFTPGo

- Owns WebDAV semantics.
- Owns S3 object operations.
- Stores user definitions in SQLite by default.
- Enforces WebDAV account authentication and per-user storage backend config.

### Rust provisioner

- Reads a restricted `users.csv` file.
- Validates usernames.
- Derives the canonical S3 prefix for every user.
- Calls the SFTPGo Admin API to create/update users.
- Does not proxy file data.
- Does not call AWS S3 directly.

### AWS S3

- Stores attachment objects.
- Uses one bucket and one prefix per user by default.
- Optional stronger isolation can use one IAM access key per user in a future ADR.

## Tenant isolation model

Let `u` be a validated username. The storage prefix is:

```text
S3_PREFIX_BASE/u/
```

For example:

```text
users/alice/
users/bob/
```

Zotero will generally create/use a `zotero` subdirectory beneath the WebDAV root, so the final keyspace is usually:

```text
users/alice/zotero/...
```

This is prefix isolation, not cryptographic isolation. For stronger blast-radius reduction, create separate AWS IAM credentials per user and restrict each credential to the same prefix.

## High-performance stance

The data plane is intentionally short:

```text
Caddy -> SFTPGo -> S3
```

The Rust provisioner is not in the data path, so its throughput is irrelevant after startup. Performance work should focus on:

- Keeping Caddy and SFTPGo on the same host/network when possible.
- Avoiding path rewrites and unnecessary middlewares in Caddy.
- Using AWS S3 in the same region as the server.
- Avoiding Nextcloud/ownCloud-style extra application layers.
- Treating MinIO as a local test double only, not a production hop.

## Failure domains

| Failure | Expected behavior |
|---|---|
| Provisioner exits after success | Normal; data plane continues. |
| Provisioner reruns | Idempotently reconciles users. |
| Caddy restarts | Active WebDAV requests may fail; clients retry. |
| SFTPGo restarts | WebDAV sessions interrupted; clients retry. |
| S3 unavailable | WebDAV operations fail until S3 recovers. |
| User removed from `users.csv` | No deletion by default; destructive reconciliation must be explicit. |

## Dependency rationale

The Rust provisioner uses only:

- `serde`
- `serde_json`

HTTP, Basic Auth, restricted CSV parsing, retry loops, and URL path escaping are small enough to keep in the standard library. This keeps the supply chain small and forces protocol contracts to remain explicit in tests.

## Critical upstream constraints

- Zotero WebDAV sync applies to personal libraries; group libraries cannot use WebDAV.
- SFTPGo's WebDAV reverse-proxy guidance requires real-client-IP configuration and Host preservation for WebDAV COPY/MOVE.
- Caddy passes incoming headers including Host by default, and sets/augments X-Forwarded headers by default.
- Conventional Commits are used for machine-readable change history.

See `docs/zotero-constraints.md` and ADRs for links and rationale.

## References

- Zotero sync documentation: https://www.zotero.org/support/sync
- SFTPGo WebDAV reverse proxy notes: https://docs.sftpgo.com/enterprise/webdav/
- SFTPGo REST API user provisioning examples: https://docs.sftpgo.com/enterprise/rest-api/
- Caddy reverse_proxy documentation: https://caddyserver.com/docs/caddyfile/directives/reverse_proxy
- Conventional Commits 1.0.0: https://www.conventionalcommits.org/en/v1.0.0/
