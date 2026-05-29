# Testing Strategy

## Test pyramid

```text
Many    unit tests: domain invariants, parsers, payload JSON
Some    mock HTTP integration tests: SFTPGo Admin API flow
Few     Docker smoke tests: Caddy/SFTPGo/WebDAV behavior
Manual  Zotero Verify Server test
```

## Unit tests

Required coverage:

- Username validation rejects path traversal and separators.
- S3 prefix derivation is stable and slash-normalized.
- `users.csv` parser rejects malformed rows and duplicate usernames.
- SFTPGo JSON payload matches the expected API contract.
- Basic Auth encoder is deterministic.
- HTTP response parser handles content-length and chunked responses.

## Mock integration tests

Use a local `TcpListener`-based mock server instead of adding an HTTP test crate. The mock server should prove:

- Token acquisition sends Basic Auth.
- Missing user triggers `POST /api/v2/users`.
- Existing user triggers `PUT /api/v2/users/{username}`.
- Non-2xx/404 responses fail closed.
- Secrets are not printed in error messages.

## Docker smoke test

The minimum WebDAV method contract is:

```text
MKCOL
PUT
PROPFIND Depth: 1
GET
DELETE
```

Run:

```sh
make up
SMOKE_WEBDAV_URL=https://dav.example.com \
SMOKE_WEBDAV_USER=alice \
SMOKE_WEBDAV_PASSWORD=... \
make smoke
```

## Manual Zotero acceptance test

1. Configure WebDAV in Zotero for a test user.
2. Click **Verify Server**.
3. Attach a small PDF to a personal-library item.
4. Sync.
5. Verify object keys appear under `users/<username>/zotero/`.
6. Configure the same Zotero account on a second device.
7. Verify the file downloads correctly.

## Regression policy

Any bug found in production or during manual Zotero verification must first be encoded as an automated test when feasible.
