# Security Design

## Threat model

Primary assets:

- Zotero attachment files in S3.
- WebDAV credentials.
- S3 access keys.
- SFTPGo admin credentials and JWTs.

Primary threats:

- Cross-user file access caused by bad prefix derivation.
- Public exposure of SFTPGo Admin API.
- Leaking secrets through logs or command output.
- Path traversal through usernames.
- Over-broad S3 credentials increasing blast radius.

## Controls

| Threat | Control |
|---|---|
| Username path traversal | Strict username validator; no `/`, `\`, `..`, whitespace, colon, or empty names. |
| Cross-user access | Deterministic prefix `base/username/`; duplicate username rejection. |
| Admin API exposure | Bind admin port to loopback or private Docker network; Caddy admin host gated by IP matcher. |
| Secret leakage | Redacted errors; no payload logging. |
| S3 blast radius | One bucket/prefix in v1; optional per-user IAM credentials in a future hardening mode. |
| Proxy spoofing | Configure SFTPGo trusted proxy ranges and Caddy trusted proxies if behind CDN. |

## Recommended AWS S3 settings

- Enable bucket versioning.
- Enable default encryption.
- Block public access.
- Use least-privilege IAM for the SFTPGo access key.
- Restrict the key to the target bucket and `S3_PREFIX_BASE/*`.
- Consider object lock only if your retention policy requires it.

## Secret handling

Do not commit:

- `.env`
- `config/users.csv`
- Caddy private keys
- SFTPGo database backups
- S3 access keys

The provided files are examples only.
