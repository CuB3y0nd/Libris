# Security Policy

## Supported surface

The supported public surface is Caddy's HTTPS WebDAV virtual host. The SFTPGo Admin API must remain private to the Docker network or loopback.

## Reporting vulnerabilities

Open a private security advisory in the hosting platform used for this repository. Do not disclose credentials, bucket names, or real user files in public issues.

## Hard requirements

- Unique WebDAV account per Zotero user.
- Unique S3 prefix per WebDAV user.
- No secrets in logs.
- No public Admin API.
- No shared Zotero data directory over WebDAV/S3.
