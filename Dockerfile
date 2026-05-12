# Changala — alternative Dockerfile for non-nix users
#
# NOTE: CI uses `nix build .#docker-image` instead of this file.
# This Dockerfile is for manual builds without nix.
#
# The binary MUST be compiled on a glibc-based system (Debian/Ubuntu).
#
# Usage:
#   cargo build --release
#   mkdir -p build/linux-amd64
#   cp target/release/changala build/linux-amd64/changala
#   docker build -t changala .

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r changala && useradd -r -g changala changala

ARG TARGETARCH
COPY build/linux-${TARGETARCH}/changala /usr/local/bin/changala
RUN chmod +x /usr/local/bin/changala

USER changala
WORKDIR /app

# Migration directories — needed at runtime by atrg_db::run_isolated_migrations
COPY crates/changala-ring/ring_migrations/ /app/ring_migrations/
COPY crates/changala-aggregator/aggregator_migrations/ /app/aggregator_migrations/

EXPOSE 3000

ENTRYPOINT ["tini", "--"]
CMD ["changala"]
