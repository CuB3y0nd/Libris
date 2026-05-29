# Operations

## Deployment modes

### Existing Caddy on host

Use the base Compose file only. It binds SFTPGo WebDAV to loopback:

```sh
docker compose --env-file .env -f deploy/compose/docker-compose.yml up -d --build
```

Then add `deploy/caddy/Caddyfile.external.example` to your existing Caddy configuration.

### Caddy inside Compose

Use the edge override:

```sh
docker compose --env-file .env \
  -f deploy/compose/docker-compose.yml \
  -f deploy/compose/docker-compose.edge.yml \
  up -d --build
```

## User lifecycle

### Add or update a user

1. Edit `config/users.csv`.
2. Run `make provision`.
3. Ask the user to verify WebDAV in Zotero.

### Remove a user

This harness intentionally does not delete users or S3 data automatically. Offboarding requires an explicit runbook:

1. Disable/delete the SFTPGo user with an explicit future CLI/admin operation.
2. Decide whether to retain, archive, or delete the user's S3 prefix.
3. Record the action.

## Backups

Back up:

- SFTPGo SQLite database volume.
- SFTPGo config volume.
- S3 bucket objects and versioning policy.
- `.env` and `config/users.csv` in a secure secret manager, not Git.

## Observability

Initial version uses container logs. Production hardening should add:

- Caddy access logs with secret-safe fields.
- SFTPGo audit logs.
- S3 server access logs or CloudTrail data events if required.
- Alerts on 5xx rate and provisioning failures.

## Rollback

The provisioner is control-plane-only. Rollback usually means:

```sh
docker compose --env-file .env -f deploy/compose/docker-compose.yml pull sftpgo
docker compose --env-file .env -f deploy/compose/docker-compose.yml up -d
```

For provisioner changes, redeploy the previous image and rerun `make provision`.

## Docker build network

The default image build uses Docker's default build network:

```sh
make docker-build
```

If the Docker build container cannot resolve `index.crates.io` but the host can,
reuse the host network for the build:

```sh
make DOCKER_BUILD_NETWORK=host docker-build
```
