# ADR 0002: Use Rust for the Provisioning Control Plane

## Status

Accepted

## Context

The deployment needs deterministic user provisioning, validation, and SFTPGo Admin API calls. Shell plus `curl`/`jq` works for demos but is brittle for long-term test-driven development.

## Decision

Implement a small synchronous Rust CLI called `provisioner`.

## Consequences

Positive:

- Strong domain invariants.
- Easy unit testing.
- Single static-ish container artifact.
- Minimal runtime dependencies.

Negative:

- Rust standard library has no HTTP client, so a restricted HTTP/1.1 client is implemented for internal cleartext Admin API calls.
- HTTPS Admin API support is intentionally out of scope for v1 because the provisioner communicates over a private Docker network.
