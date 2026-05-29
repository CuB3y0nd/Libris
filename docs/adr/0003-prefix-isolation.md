# ADR 0003: Use One S3 Bucket with Per-user Prefixes

## Status

Accepted

## Context

The system needs multi-user isolation without creating an AWS bucket for every user.

## Decision

Use one bucket and deterministic per-user prefixes:

```text
<S3_PREFIX_BASE>/<username>/
```

## Consequences

Positive:

- Simple operations.
- Works with SFTPGo's per-user S3 key prefix configuration.
- Easy to inspect and back up.

Negative:

- Prefix isolation relies on SFTPGo and IAM policy correctness.
- Stronger isolation requires per-user IAM credentials or separate buckets.

## Future hardening

Add an optional mode:

```text
username -> IAM access key scoped to bucket/prefix
```

That mode should remain off by default because it increases provisioning complexity.
