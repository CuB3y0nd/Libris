# ADR 0001: Use SFTPGo as the WebDAV-to-S3 Gateway

## Status

Accepted

## Context

Zotero speaks WebDAV for personal-library attachment sync. AWS S3 does not expose WebDAV semantics. We need a gateway that handles WebDAV operations and stores objects in S3.

## Decision

Use SFTPGo as the WebDAV server and S3 backend adapter.

## Consequences

Positive:

- Avoids reimplementing WebDAV/S3 edge cases.
- Supports per-user storage configuration.
- Keeps Rust code out of the data plane.

Negative:

- Brings SFTPGo's AGPLv3 Community licensing considerations.
- Requires compatibility tracking against SFTPGo releases.
- Some WebDAV/cloud-backend limitations must be handled operationally.

## References

- SFTPGo project overview: https://github.com/drakkan/sftpgo
- SFTPGo WebDAV documentation: https://docs.sftpgo.com/enterprise/webdav/
- SFTPGo REST API examples: https://docs.sftpgo.com/enterprise/rest-api/
