# Changala — minimal runtime image
#
# Uses distroless (glibc) — compatible with binaries built on Ubuntu/Nix.
# ~20MB base. No shell, no package manager, no attack surface.
#
# Multi-arch CI build:
#   Expects binaries at build/linux-amd64/changala and build/linux-arm64/changala
#
# Local single-arch build:
#   cargo build --release
#   mkdir -p build/linux-amd64
#   cp target/release/changala build/linux-amd64/changala
#   docker build -t changala .

FROM debian:trixie-slim AS base
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r changala && useradd -r -g changala changala

FROM base

ARG TARGETARCH
COPY build/linux-${TARGETARCH}/changala /usr/local/bin/changala
RUN chmod +x /usr/local/bin/changala

USER changala
WORKDIR /app

EXPOSE 3000

ENTRYPOINT ["tini", "--"]
CMD ["changala"]
