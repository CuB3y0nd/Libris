# ADR 0004: Use Caddy as the Public Edge Proxy

## Status

Accepted

## Context

The user already uses Caddy. WebDAV compatibility is sensitive to path and Host handling.

## Decision

Caddy terminates TLS and reverse-proxies a dedicated WebDAV host to SFTPGo.

## Consequences

Positive:

- Simple TLS automation.
- Existing operational familiarity.
- Clean separation between public edge and private SFTPGo listener.

Negative:

- Misconfigured path stripping can break WebDAV.
- If Caddy is behind another proxy/CDN, trusted proxy settings need explicit review.

## Rules

- Prefer a dedicated hostname, e.g. `dav.example.com`.
- Do not use `handle_path`/prefix stripping for WebDAV.
- Keep SFTPGo Admin API private.

## References

- Caddy reverse_proxy documentation: https://caddyserver.com/docs/caddyfile/directives/reverse_proxy
- SFTPGo WebDAV reverse-proxy notes: https://docs.sftpgo.com/enterprise/webdav/
