# ADR 0005: Treat External Integrations as Tested Contracts

## Status

Accepted

## Context

SFTPGo, Caddy, Zotero, and S3 are external systems. Integration drift is likely over time.

## Decision

Encode every boundary as a testable contract:

- SFTPGo JSON payload contract tests.
- Restricted CSV parser tests.
- WebDAV smoke script.
- Manual Zotero Verify Server acceptance checklist.

## Consequences

Positive:

- Safer upgrades.
- Clearer Codex implementation target.
- Better regression capture.

Negative:

- Some behavior still requires manual Zotero testing because Zotero itself is not part of CI.
