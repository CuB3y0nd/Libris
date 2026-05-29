# Provisioner

The provisioner is a synchronous Rust CLI that reconciles `config/users.csv` into SFTPGo users with S3-backed filesystems.

## Inputs

Environment variables:

- `SFTPGO_ENDPOINT` internal HTTP endpoint, e.g. `http://sftpgo:8080`
- `SFTPGO_ADMIN_USER`
- `SFTPGO_ADMIN_PASSWORD`
- `USER_FILE`
- `S3_BUCKET`
- `S3_REGION`
- `S3_ENDPOINT` optional, empty for native AWS S3
- `S3_ACCESS_KEY`
- `S3_SECRET_KEY`
- `S3_PREFIX_BASE`
- `PROVISIONER_DRY_RUN`

## CSV format

```csv
username,password
alice,alice-password
bob,bob-password
```

No quoting is supported intentionally. Usernames are restricted to ASCII letters, digits, `.`, `_`, and `-`.
