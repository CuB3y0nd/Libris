# Contributing

## Local validation

```sh
make validate
```

## Commit messages

Use Conventional Commits 1.0.0:

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Examples:

```text
feat(provisioner): support per-user quota bytes
fix(caddy): preserve webdav host header through proxy
test(payload): assert s3 key prefix isolation
```

## Pull request checklist

- [ ] A failing test was added before the implementation.
- [ ] `make validate` passes.
- [ ] No secrets were committed.
- [ ] Any architecture tradeoff is documented in an ADR.
- [ ] User-visible docs are updated.

Reference: https://www.conventionalcommits.org/en/v1.0.0/
