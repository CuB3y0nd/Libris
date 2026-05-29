# AGENTS.md

This file is the implementation contract for Codex or any automated coding agent working in this repository.

## Mission

Build a reliable, high-performance, test-driven one-click deployment harness for:

```text
Zotero personal-library attachment sync
  -> Caddy
  -> SFTPGo WebDAV
  -> AWS S3, isolated by per-user prefix
```

The Rust code is a control-plane provisioner, not a data-plane proxy. Do not implement WebDAV or S3 transfer logic in Rust unless an ADR explicitly changes the architecture.

## Architectural invariants

1. Caddy is the public TLS edge.
2. SFTPGo is the only WebDAV server.
3. SFTPGo is the only component talking to AWS S3 for file data.
4. Each Zotero user maps to exactly one SFTPGo user.
5. Each SFTPGo user maps to a unique S3 key prefix: `prefix_base/<username>/`.
6. Usernames are validated before they are used in S3 prefixes or URL paths.
7. The provisioner is idempotent: re-running it creates missing users and updates existing users.
8. The provisioner must never print passwords, S3 secrets, JWTs, or raw secret-bearing JSON payloads.
9. Production must not require MinIO; MinIO is allowed only as an optional local/E2E test double.
10. No unsafe Rust.

## Dependency policy

Prefer standard library implementations. Third-party crates require clear justification in `docs/architecture.md` or an ADR.

Allowed baseline crates:

- `serde`
- `serde_json`

Avoid adding runtime crates for HTTP, CSV, CLI parsing, logging, async, regex, or URL parsing unless tests prove the standard-library implementation is inadequate.

## Test-driven development

Before implementing behavior, add or update a failing test. The minimum test layers are:

- Pure unit tests for username validation and S3 prefix derivation.
- Parser tests for the restricted `users.csv` format.
- JSON contract tests for the SFTPGo user payload.
- HTTP unit tests for request construction and response parsing.
- Mock-server integration tests for token acquisition and user upsert behavior.
- Docker smoke tests for WebDAV methods: `MKCOL`, `PUT`, `PROPFIND`, `GET`, `DELETE`.

A change is not done until:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Code organization rules

Keep high cohesion and low coupling:

```text
config/       reading and validating runtime configuration
domain/       pure types and invariants
infra/http/   minimal HTTP transport for internal SFTPGo API calls
infra/sftpgo/ SFTPGo API client and payload contracts
io/           filesystem input, currently restricted users.csv parsing
app.rs        orchestration only
main.rs       process entry point only
```

Do not put unrelated modules in a flat `src/` pile. New capabilities get their own module folders and tests.

## Rust style

- Use explicit domain types rather than raw strings at boundaries.
- Keep functions small and deterministic where possible.
- Return `Result<T, HarnessError>` from library code.
- Do not call `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!` in production code.
- Tests may use clear assertions but should avoid brittle string snapshots for secret-bearing payloads.
- Keep secrets out of `Debug` output.
- Favor synchronous, simple code. The provisioner is control plane, so async is not justified until measured.

## Commit style

Use Conventional Commits 1.0.0. Specification: https://www.conventionalcommits.org/en/v1.0.0/

Use this form:

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Recommended types:

- `feat`: user-visible behavior
- `fix`: bug fix
- `test`: tests only
- `docs`: documentation only
- `refactor`: behavior-preserving restructuring
- `perf`: measurable performance improvement
- `build`: Docker, Cargo, or packaging
- `ci`: CI workflow
- `chore`: maintenance

Examples:

```text
feat(provisioner): upsert sftpgo users with s3 prefixes

test(csv): reject malformed user rows

docs(adr): record caddy as the tls edge

fix(http): parse chunked token responses
```

Breaking changes use `!` or a `BREAKING CHANGE:` footer.

## Security boundaries

- Do not expose SFTPGo Admin API to the public Internet.
- Do not commit `.env`, `config/users.csv`, real S3 keys, or generated tokens.
- Do not share a single Zotero/WebDAV account across users.
- Do not mount the Zotero data directory into WebDAV or S3. Only Zotero-managed attachment sync belongs here.

## When unsure

Prefer a small, tested, reversible change over a broad refactor. If a tradeoff affects data isolation, credentials, or Zotero compatibility, add an ADR before coding.
