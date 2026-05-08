# Changala — minimal runtime image
#
# Supports multi-arch builds (amd64, arm64).
# Binary is built externally (CI) and copied in per-platform.
#
# Multi-arch CI build:
#   Expects binaries at build/linux-amd64/changala and build/linux-arm64/changala
#
# Local single-arch build:
#   cargo build --release
#   mkdir -p build/linux-amd64
#   cp target/release/changala build/linux-amd64/changala
#   docker build -t changala .

FROM alpine:3.21

RUN apk add --no-cache ca-certificates tini \
    && addgroup -S changala && adduser -S changala -G changala

ARG TARGETARCH
COPY build/linux-${TARGETARCH}/changala /usr/local/bin/changala
RUN chmod +x /usr/local/bin/changala

USER changala
WORKDIR /app

EXPOSE 3000

ENTRYPOINT ["tini", "--"]
CMD ["changala"]
