# syntax=docker/dockerfile:1
# landline backend container (A33, NFR-DEPLOY-03). An *evaluated* profile; the
# native systemd unit remains the reference deployment (ADR-06). The image runs
# as a non-root user with a read-only root filesystem (NFR-SEC-10) — enforce the
# read-only rootfs, dropped capabilities, and no-new-privileges at runtime (see
# compose.yml).

FROM rust:1.94-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p landline-backend

FROM debian:bookworm-slim AS runtime
# Unprivileged service user (NFR-SEC-10).
# hadolint ignore=DL3008
RUN useradd --system --no-create-home --uid 10001 landline \
 && apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/landline /usr/local/bin/landline
USER landline
ENV LANDLINE_CONFIG=/etc/landline/config.toml
# Loopback bind by default (NFR-SEC-13); publish only on the private tunnel.
EXPOSE 8443
ENTRYPOINT ["/usr/local/bin/landline"]
