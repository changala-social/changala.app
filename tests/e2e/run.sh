#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Colors ────────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  RED='\033[0;31m'
  YELLOW='\033[1;33m'
  CYAN='\033[0;36m'
  BOLD='\033[1m'
  RESET='\033[0m'
else
  GREEN='' RED='' YELLOW='' CYAN='' BOLD='' RESET=''
fi

PASS_COUNT=0
FAIL_COUNT=0

section() {
  echo ""
  echo -e "${CYAN}══════════════════════════════════════════════════════════════${RESET}"
  echo -e "${BOLD}  $1${RESET}"
  echo -e "${CYAN}══════════════════════════════════════════════════════════════${RESET}"
}

run_test() {
  local name="$1"
  local method="$2"
  local path="$3"
  local body="${4:-}"
  local expected="${5:-200}"

  local url="http://127.0.0.1:13000${path}"
  local curl_args=(-s -o /tmp/changala_e2e_response -w "%{http_code}" -X "$method")

  if [ -n "$body" ]; then
    curl_args+=(-H "Content-Type: application/json" -d "$body")
  fi

  local http_code
  http_code=$(curl "${curl_args[@]}" "$url" 2>/dev/null) || http_code="000"

  if [ "$http_code" = "$expected" ]; then
    echo -e "  ${GREEN}✅ PASS${RESET}  $name (HTTP $http_code)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo -e "  ${RED}❌ FAIL${RESET}  $name (expected $expected, got $http_code)"
    if [ -f /tmp/changala_e2e_response ]; then
      echo "         Response: $(head -c 200 /tmp/changala_e2e_response)"
    fi
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

run_test_body() {
  local name="$1"
  local method="$2"
  local path="$3"
  local body="${4:-}"
  local expected_code="${5:-200}"
  local jq_filter="$6"

  local url="http://127.0.0.1:13000${path}"
  local curl_args=(-s -o /tmp/changala_e2e_response -w "%{http_code}" -X "$method")

  if [ -n "$body" ]; then
    curl_args+=(-H "Content-Type: application/json" -d "$body")
  fi

  local http_code
  http_code=$(curl "${curl_args[@]}" "$url" 2>/dev/null) || http_code="000"

  if [ "$http_code" != "$expected_code" ]; then
    echo -e "  ${RED}❌ FAIL${RESET}  $name (expected HTTP $expected_code, got $http_code)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    return
  fi

  local jq_result
  jq_result=$(jq -r "$jq_filter" /tmp/changala_e2e_response 2>/dev/null || echo "jq_error")

  if [ "$jq_result" = "true" ]; then
    echo -e "  ${GREEN}✅ PASS${RESET}  $name (HTTP $http_code, body check passed)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo -e "  ${RED}❌ FAIL${RESET}  $name (HTTP $http_code, body check failed: $jq_result)"
    echo "         Response: $(head -c 200 /tmp/changala_e2e_response)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

# ── Temp directory & cleanup ──────────────────────────────────────────────────
TMPDIR="$(mktemp -d /tmp/changala-e2e.XXXXXX)"
echo -e "${YELLOW}Working directory: $TMPDIR${RESET}"

cleanup() {
  section "Cleanup"

  if [ -n "${CHANGALA_PID:-}" ] && kill -0 "$CHANGALA_PID" 2>/dev/null; then
    echo "  Stopping changala-ring server (PID $CHANGALA_PID)..."
    kill "$CHANGALA_PID" 2>/dev/null || true
    wait "$CHANGALA_PID" 2>/dev/null || true
  fi

  if [ -f "$TMPDIR/garage/garage.pid" ]; then
    local gpid
    gpid=$(cat "$TMPDIR/garage/garage.pid")
    if kill -0 "$gpid" 2>/dev/null; then
      echo "  Stopping Garage (PID $gpid)..."
      kill "$gpid" 2>/dev/null || true
      wait "$gpid" 2>/dev/null || true
    fi
  fi

  if [ -d "$TMPDIR/pgdata" ]; then
    echo "  Stopping PostgreSQL..."
    pg_ctl -D "$TMPDIR/pgdata" stop -m fast 2>/dev/null || true
  fi

  echo "  Removing temp directory: $TMPDIR"
  rm -rf "$TMPDIR"
  rm -f /tmp/changala_e2e_response

  echo ""
  echo -e "${CYAN}══════════════════════════════════════════════════════════════${RESET}"
  echo -e "${BOLD}  Test Summary${RESET}"
  echo -e "${CYAN}══════════════════════════════════════════════════════════════${RESET}"
  echo -e "  Total:  $((PASS_COUNT + FAIL_COUNT))"
  echo -e "  ${GREEN}Passed: $PASS_COUNT${RESET}"
  echo -e "  ${RED}Failed: $FAIL_COUNT${RESET}"
  echo -e "${CYAN}══════════════════════════════════════════════════════════════${RESET}"

  if [ "$FAIL_COUNT" -gt 0 ]; then
    exit 1
  fi
}
trap cleanup EXIT

# ── Find changala binary ──────────────────────────────────────────────────────
section "Locating changala-ring binary"

if [ -n "${CHANGALA_BIN:-}" ]; then
  CHANGALA_BIN="$CHANGALA_BIN"
elif [ -f "$SCRIPT_DIR/../../target/release/changala-ring" ]; then
  CHANGALA_BIN="$SCRIPT_DIR/../../target/release/changala-ring"
elif [ -f "$SCRIPT_DIR/../../target/debug/changala-ring" ]; then
  CHANGALA_BIN="$SCRIPT_DIR/../../target/debug/changala-ring"
else
  echo -e "${RED}ERROR: Cannot find changala-ring binary.${RESET}"
  echo "  Set CHANGALA_BIN or run 'cargo build' first."
  exit 1
fi

CHANGALA_BIN="$(cd "$(dirname "$CHANGALA_BIN")" && pwd)/$(basename "$CHANGALA_BIN")"
echo "  Using: $CHANGALA_BIN"

# ── PostgreSQL ────────────────────────────────────────────────────────────────
section "Starting PostgreSQL (port 15432)"

PGDATA="$TMPDIR/pgdata"
initdb -D "$PGDATA" --auth=trust -U postgres >/dev/null 2>&1
echo "port = 15432" >> "$PGDATA/postgresql.conf"
echo "unix_socket_directories = '$TMPDIR'" >> "$PGDATA/postgresql.conf"
echo "listen_addresses = '127.0.0.1'" >> "$PGDATA/postgresql.conf"

pg_ctl -D "$PGDATA" -l "$TMPDIR/pg.log" start >/dev/null 2>&1

# Wait for postgres
for i in $(seq 1 30); do
  if psql -h 127.0.0.1 -p 15432 -U postgres -c "SELECT 1" >/dev/null 2>&1; then
    break
  fi
  sleep 0.3
done

psql -h 127.0.0.1 -p 15432 -U postgres -c "CREATE USER changala WITH SUPERUSER;" >/dev/null 2>&1 || true
psql -h 127.0.0.1 -p 15432 -U postgres -c "CREATE DATABASE changala_e2e OWNER changala;" >/dev/null 2>&1

echo "  PostgreSQL ready on port 15432"

# ── Garage S3 ─────────────────────────────────────────────────────────────────
section "Starting Garage S3 (port 13900)"

GARAGE_DIR="$TMPDIR/garage"
mkdir -p "$GARAGE_DIR/data" "$GARAGE_DIR/meta"

GARAGE_CONF="$GARAGE_DIR/garage.toml"
cat > "$GARAGE_CONF" <<EOF
metadata_dir = "$GARAGE_DIR/meta"
data_dir = "$GARAGE_DIR/data"
db_engine = "sqlite"
replication_factor = 1
rpc_bind_addr = "127.0.0.1:13901"
rpc_secret = "$(openssl rand -hex 32)"

[s3_api]
s3_region = "us-east-1"
api_bind_addr = "127.0.0.1:13900"
root_domain = ".s3.garage.localhost"

[admin]
api_bind_addr = "127.0.0.1:13903"
admin_token = "e2e-admin"
EOF

garage -c "$GARAGE_CONF" server &
GARAGE_SERVER_PID=$!
echo "$GARAGE_SERVER_PID" > "$GARAGE_DIR/garage.pid"

# Wait for Garage to be ready
for i in $(seq 1 30); do
  if garage -c "$GARAGE_CONF" status >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done

# Layout setup
NODE_ID=$(garage -c "$GARAGE_CONF" node id -q | head -1 | cut -d@ -f1 | tr -d ' ')
SHORT_ID="${NODE_ID:0:16}"

garage -c "$GARAGE_CONF" layout assign "$SHORT_ID" -z dc1 -c 1G 2>/dev/null || true
garage -c "$GARAGE_CONF" layout apply --version 1 2>/dev/null || true

echo "  Garage node: $SHORT_ID"

# Create key and bucket
garage -c "$GARAGE_CONF" key create changala-e2e-key >/dev/null 2>&1 || true

KEY_INFO=$(garage -c "$GARAGE_CONF" key info changala-e2e-key 2>&1)
S3_ACCESS_KEY=$(echo "$KEY_INFO" | grep -i "Key ID" | awk '{print $NF}' | tr -d ' ')
S3_SECRET_KEY=$(echo "$KEY_INFO" | grep -i "Secret key" | awk '{print $NF}' | tr -d ' ')

if [ -z "$S3_ACCESS_KEY" ] || [ -z "$S3_SECRET_KEY" ]; then
  echo -e "${RED}ERROR: Failed to parse Garage key credentials${RESET}"
  echo "  Key info output:"
  echo "$KEY_INFO"
  exit 1
fi

garage -c "$GARAGE_CONF" bucket create changala-blobs 2>/dev/null || true
garage -c "$GARAGE_CONF" bucket allow --read --write --owner changala-blobs --key changala-e2e-key 2>/dev/null || true

echo "  Garage ready — bucket: changala-blobs"
echo "  Access key: $S3_ACCESS_KEY"

# ── Generate atrg.toml ───────────────────────────────────────────────────────
section "Generating atrg.toml"

cat > "$TMPDIR/atrg.toml" <<EOF
[app]
name = "changala"
host = "127.0.0.1"
port = 13000
environment = "test"
secret_key = "e2e-test-secret-key-0000000000000000000000000000000000000000"

[auth]
client_id = "http://127.0.0.1:13000"
redirect_uri = "http://127.0.0.1:13000/auth/callback"

[database]
url = "postgres://changala@127.0.0.1:15432/changala_e2e"

[jetstream]
host = "jetstream1.us-east.bsky.network"
collections = [
  "app.changala.membership",
  "app.changala.course",
  "app.changala.session",
  "app.changala.keyword",
  "app.changala.note",
  "app.changala.vote",
  "app.changala.collectivenote.proposal",
  "app.changala.label",
  "app.changala.archive",
  "app.changala.brain.node",
  "app.changala.brain.link",
]

[changala]
database_url = "postgres://changala@127.0.0.1:15432/changala_e2e"

[changala.s3]
endpoint = "http://127.0.0.1:13900"
bucket = "changala-blobs"
region = "us-east-1"
path_style = true
access_key = "$S3_ACCESS_KEY"
secret_key = "$S3_SECRET_KEY"
EOF

echo "  Written to $TMPDIR/atrg.toml"

# Copy Ring migrations so the binary can find them
cp -r "$SCRIPT_DIR/../../crates/changala-ring/ring_migrations" "$TMPDIR/ring_migrations"
echo "  Copied ring_migrations to $TMPDIR/ring_migrations"

# ── Copy and start changala-ring ─────────────────────────────────────────────
section "Starting changala-ring server"

cp "$CHANGALA_BIN" "$TMPDIR/changala-ring"

cd "$TMPDIR"
./changala-ring > "$TMPDIR/changala.log" 2>&1 &
CHANGALA_PID=$!
echo "  PID: $CHANGALA_PID"

# Health check — wait up to 15 seconds
for i in $(seq 1 30); do
  if curl -sf http://127.0.0.1:13000/api/health >/dev/null 2>&1; then
    echo "  Server healthy"
    break
  fi
  if ! kill -0 "$CHANGALA_PID" 2>/dev/null; then
    echo -e "${RED}ERROR: changala-ring exited prematurely${RESET}"
    echo "  Log tail:"
    tail -20 "$TMPDIR/changala.log"
    exit 1
  fi
  sleep 0.5
done

if ! curl -sf http://127.0.0.1:13000/api/health >/dev/null 2>&1; then
  echo -e "${RED}ERROR: changala-ring failed to become healthy${RESET}"
  echo "  Log tail:"
  tail -20 "$TMPDIR/changala.log"
  exit 1
fi

# ══════════════════════════════════════════════════════════════════════════════
# TEST SUITES
# ══════════════════════════════════════════════════════════════════════════════

section "Health & Index"

run_test "GET /" "GET" "/" "" "200"
run_test "GET /api/health" "GET" "/api/health" "" "200"

# ── Identity ──────────────────────────────────────────────────────────────────
section "Identity (verifyEmail flow)"

run_test "verifyEmail step 1 — request OTP" \
  "POST" "/xrpc/app.changala.ring.verifyEmail" \
  '{"did":"did:plc:e2euser001","email":"test@nitc.ac.in"}' "200"

# Query OTP from postgres
OTP=$(psql -h 127.0.0.1 -p 15432 -U changala changala_e2e -t -c \
  "SELECT code FROM otp_codes WHERE did='did:plc:e2euser001' ORDER BY id DESC LIMIT 1" \
  2>/dev/null | tr -d ' \n')

if [ -n "$OTP" ]; then
  echo "  OTP retrieved: $OTP"
  run_test "verifyEmail step 2 — verify OTP" \
    "POST" "/xrpc/app.changala.ring.verifyEmail" \
    "{\"did\":\"did:plc:e2euser001\",\"email\":\"test@nitc.ac.in\",\"otp\":\"$OTP\"}" "200"
else
  echo -e "  ${YELLOW}⚠  Could not retrieve OTP from database — skipping step 2${RESET}"
fi

run_test "getMemberships" \
  "GET" "/xrpc/app.changala.ring.getMemberships?did=did:plc:e2euser001" "" "200"

run_test "getRole" \
  "GET" "/xrpc/app.changala.ring.getRole?did=did:plc:e2euser001" "" "200"

# ── Unauthenticated Reads ────────────────────────────────────────────────────
section "Unauthenticated Reads (Ring)"

run_test "listCourses" \
  "GET" "/xrpc/app.changala.ring.listCourses" "" "200"

run_test "listSessions (fake course)" \
  "GET" "/xrpc/app.changala.ring.listSessions?courseUri=at://fake/course/1" "" "200"

run_test "isBanned" \
  "GET" "/xrpc/app.changala.ring.isBanned?did=did:plc:nobody" "" "200"

# ── Unauthenticated Global View Reads ────────────────────────────────────────
# NOTE: Global View endpoints are served by changala-aggregator, not changala-ring.
# These tests are skipped in the Ring-only e2e suite.
section "Unauthenticated Global View Reads (SKIPPED — Aggregator not running)"
echo -e "  ${YELLOW}⚠  Global View endpoints are served by changala-aggregator${RESET}"
echo -e "  ${YELLOW}⚠  This e2e suite only tests changala-ring${RESET}"

# ── Seeded Data Tests ────────────────────────────────────────────────────────
section "Seeded Data Tests"

SEED_SQL="$SCRIPT_DIR/seed.sql"
if [ -f "$SEED_SQL" ]; then
  echo "  Running seed.sql..."
  psql -h 127.0.0.1 -p 15432 -U changala changala_e2e -f "$SEED_SQL" >/dev/null 2>&1

  run_test_body "getCourse (seeded Algorithms)" \
    "GET" "/xrpc/app.changala.ring.getCourse?uri=at://did:web:ring.changala.local/app.changala.course/seed001" \
    "" "200" \
    '(.title // "" | test("Algorithms")) // false'

  run_test_body "listCourses (non-empty after seed)" \
    "GET" "/xrpc/app.changala.ring.listCourses" \
    "" "200" \
    '(.courses // [] | length) > 0'

  run_test "getSession (seeded)" \
    "GET" "/xrpc/app.changala.ring.getSession?uri=at://did:web:ring.changala.local/app.changala.session/seed001" "" "200"

  run_test "listSessions (seeded course)" \
    "GET" "/xrpc/app.changala.ring.listSessions?courseUri=at://did:web:ring.changala.local/app.changala.course/seed001" "" "200"

  # NOTE: getNotes, getKeywordHistogram, getBacklinks, getNodeGraph, getNeighbours
  # are globalview endpoints served by changala-aggregator (not running in this suite).
  echo -e "  ${YELLOW}⚠  Skipping globalview seeded tests (Aggregator not running)${RESET}"
else
  echo -e "  ${YELLOW}⚠  seed.sql not found at $SEED_SQL — skipping seeded data tests${RESET}"
fi

# ── Auth-Required Endpoints ──────────────────────────────────────────────────
section "Auth-Required Endpoints (expect 401 without token)"

run_test "createCourse without auth → 401" \
  "POST" "/xrpc/app.changala.ring.createCourse" \
  '{"title":"Unauthorized"}' "401"

run_test "createSession without auth → 401" \
  "POST" "/xrpc/app.changala.ring.createSession" \
  '{"course_uri":"at://fake/course/1"}' "401"

run_test "createNode without auth → 401" \
  "POST" "/xrpc/app.changala.ring.createNode" \
  '{"title":"Unauthorized Node"}' "401"

run_test "banDid without auth → 401" \
  "POST" "/xrpc/app.changala.ring.banDid" \
  '{"did":"did:plc:someone"}' "401"
