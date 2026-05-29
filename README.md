# Libris

A self-hosted, multi-tenant Zotero storage platform powered by WebDAV and S3-compatible object storage.

## What It Deploys

Libris provisions this production data path:

```text
Zotero WebDAV sync -> Caddy -> SFTPGo WebDAV -> AWS S3
```

The Rust provisioner is control-plane only. It reads `config/users.csv`, creates
or updates SFTPGo users, and assigns each user a private S3 key prefix:

```text
S3_PREFIX_BASE/<username>/
```

It does not proxy WebDAV traffic and does not upload files to S3 itself.

## Requirements

- Docker with Compose v2
- A reachable AWS S3 bucket
- AWS access credentials allowed to read/write the configured bucket and prefix
- A DNS name for WebDAV, for example `dav.example.com`
- Caddy, either already running on the host or started by this Compose project
- Rust only if you want to run local development checks outside Docker

## Production Setup With AWS S3

1. Create the local config files:

```sh
cp .env.example .env
cp config/users.csv.example config/users.csv
```

2. Edit `.env` for your deployment:

```sh
PUBLIC_WEBDAV_HOST=dav.example.com
PUBLIC_ADMIN_HOST=sftpgo-admin.example.com

SFTPGO_ADMIN_USER=admin
SFTPGO_ADMIN_PASSWORD=<long-random-admin-password>
SFTPGO_WEBDAV_PORT=5227
SFTPGO_ADMIN_PORT=8080

S3_BUCKET=<your-aws-bucket>
S3_REGION=<your-aws-region>
S3_ENDPOINT=
S3_ACCESS_KEY=<your-aws-access-key>
S3_SECRET_KEY=<your-aws-secret-key>
S3_PREFIX_BASE=users
```

For native AWS S3, keep `S3_ENDPOINT` empty. MinIO is only a local test double.

3. Edit `config/users.csv`:

```csv
username,password
alice,long-random-webdav-password
bob,another-long-random-webdav-password
```

Usernames may contain ASCII letters, digits, `.`, `_`, and `-`. Each user gets a
separate SFTPGo account and a separate S3 prefix.

## Deploy With An Existing Host Caddy

This is the default mode when Caddy already runs on the machine.

1. Start SFTPGo and run the provisioner:

```sh
make up
```

If Docker build containers cannot resolve `index.crates.io` but the host can,
use:

```sh
make DOCKER_BUILD_NETWORK=host up
```

2. Add `deploy/caddy/Caddyfile.external.example` to your host Caddy config and
replace the example hostnames. The default local WebDAV upstream is:

```text
127.0.0.1:5227
```

3. Reload Caddy.

4. Configure Zotero:

```text
File Syncing: WebDAV
URL: https://dav.example.com
Username: alice
Password: the password from config/users.csv
```

## Deploy Caddy Inside Compose

Use this mode if this project should run Caddy too:

```sh
make up-edge
```

This exposes ports `80` and `443` from the Caddy container and reverse-proxies
the dedicated WebDAV host to SFTPGo inside the Compose network.

## Verify The Deployment

Run the WebDAV smoke test against the public URL:

```sh
SMOKE_WEBDAV_URL=https://dav.example.com \
SMOKE_WEBDAV_USER=alice \
SMOKE_WEBDAV_PASSWORD=<alice-webdav-password> \
make smoke
```

The smoke test exercises:

```text
MKCOL -> PUT -> PROPFIND -> GET -> DELETE
```

Then use Zotero's **Verify Server** button and perform a real attachment sync.
Objects should appear under:

```text
users/<username>/zotero/
```

## Add Or Update Users

1. Edit `config/users.csv`.
2. Re-run provisioning:

```sh
make provision
```

Provisioning is idempotent. Existing users are updated; missing users are
created. Users removed from `config/users.csv` are not deleted automatically.

## Local E2E Test With MinIO

For local testing without touching AWS, run:

```sh
make up-e2e
SMOKE_WEBDAV_URL=http://127.0.0.1:5227 \
SMOKE_WEBDAV_USER=alice \
SMOKE_WEBDAV_PASSWORD=change-this-alice-password \
make smoke
```

This starts MinIO as an S3-compatible test double and creates the configured
bucket before provisioning users. Do not put MinIO in the production path.

## Stop The Stack

```sh
make down
```

To remove local build artifacts:

```sh
make clean
```

## Development Checks

Before committing code changes, run:

```sh
make validate
```

That runs:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The Rust provisioner intentionally uses only `serde` and `serde_json` as direct
runtime crates.

## Security Notes

- Do not commit `.env`, `config/users.csv`, real AWS keys, WebDAV passwords, or
  generated tokens.
- Keep the SFTPGo Admin API private. The default Compose file binds it to
  loopback.
- Do not share one WebDAV account across multiple Zotero users.
- Do not mount Zotero's local data directory into WebDAV or S3.
