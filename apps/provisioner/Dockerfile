# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
WORKDIR /work

COPY Cargo.toml ./
COPY apps/provisioner/Cargo.toml apps/provisioner/Cargo.toml
COPY apps/provisioner/src apps/provisioner/src

RUN if [ -f Cargo.lock ]; then cargo build --release --locked; else cargo build --release; fi

FROM debian:bookworm-slim AS runtime
RUN useradd --system --uid 10001 --home-dir /nonexistent --shell /usr/sbin/nologin provisioner
COPY --from=builder /work/target/release/zotero_s3_webdav_provisioner /usr/local/bin/provisioner
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/provisioner"]
