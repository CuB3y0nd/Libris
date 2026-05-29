# Zotero Constraints

## Supported scope

This harness supports Zotero **personal-library file sync** through WebDAV.

It does not support Zotero group-library attachment sync through WebDAV. Zotero documents WebDAV as usable for personal-library file sync and states that group libraries cannot use WebDAV.

## Metadata sync

Zotero metadata sync remains Zotero's own service:

```text
items, notes, links, tags, collections -> Zotero data sync
PDFs and attachments                  -> this WebDAV/S3 bridge
```

Do not attempt to sync or share Zotero's local data directory through this stack.

## Client setup

For each user:

```text
Preferences -> Sync -> File Syncing -> WebDAV
URL: https://dav.example.com
Username: per-user WebDAV username
Password: per-user WebDAV password
```

Then run Zotero's **Verify Server** check.

## Operational implication

A shared WebDAV account would collapse multiple users into one S3 prefix and destroy tenant isolation. This repository treats shared accounts as a configuration error.

## References

- Zotero Syncing documentation: https://www.zotero.org/support/sync
