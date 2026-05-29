# Code Style

## Rust

- Edition: 2021.
- `unsafe` is forbidden.
- Public APIs use domain types where possible.
- No `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!` in production code.
- Modules should be cohesive and small.
- Side effects stay at the edge: `config`, `io`, `infra`.
- Domain modules must remain pure.

## Error messages

Errors should explain what failed without printing secrets. Good:

```text
failed to create SFTPGo user alice: HTTP 500
```

Bad:

```text
failed payload { access_secret: ... }
```

## Formatting

Use:

```sh
cargo fmt --all
```

## Linting

Use:

```sh
cargo clippy --workspace --all-targets -- -D warnings
```
