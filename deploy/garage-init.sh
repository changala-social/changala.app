#!/usr/bin/env bash
# deploy/garage-init.sh — First-time Garage setup for docker-compose
#
# Run AFTER `docker compose up -d`:
#   bash deploy/garage-init.sh
#
# This script:
#   1. Gets the Garage node ID
#   2. Assigns it a storage layout
#   3. Creates an S3 key and bucket
#   4. Prints the credentials to put in atrg.toml

set -euo pipefail

GARAGE="docker compose exec -T garage /garage"

echo "==> Waiting for Garage..."
for i in $(seq 1 30); do
  if $GARAGE status >/dev/null 2>&1; then break; fi
  sleep 1
done

echo "==> Configuring layout..."
NODE_ID=$($GARAGE node id -q 2>/dev/null | head -1 | cut -d@ -f1 | tr -d ' ')
$GARAGE layout assign "${NODE_ID:0:16}" -z dc1 -c 1G 2>/dev/null || true
$GARAGE layout apply --version 1 2>/dev/null || true

echo "==> Creating key and bucket..."
$GARAGE key create changala-key 2>/dev/null || true
$GARAGE bucket create changala-blobs 2>/dev/null || true
$GARAGE bucket allow --read --write --owner changala-blobs --key changala-key 2>/dev/null || true

echo ""
echo "==> Garage ready. S3 credentials:"
echo ""
$GARAGE key info changala-key 2>/dev/null | grep -E "Key ID|Secret key"
echo ""
echo "Update deploy/atrg.toml [changala.s3] with these values, then:"
echo "  docker compose restart changala"
