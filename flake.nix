{
  description = "Changala — federated social learning platform on AT Protocol";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "clippy"
            "rustfmt"
          ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Source filter: include .rs, .json, .sql, .toml files
        srcFilter =
          path: type:
          (craneLib.filterCargoSources path type)
          || (builtins.match ".*\\.json$" path != null)
          || (builtins.match ".*\\.sql$" path != null)
          || (builtins.match ".*\\.toml$" path != null);

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = srcFilter;
        };

        isDarwin = pkgs.stdenv.isDarwin;

        # macOS runtime libraries (no apple_sdk frameworks — use system SDK)
        darwinBuildInputs = pkgs.lib.optionals isDarwin [
          pkgs.libiconv
          pkgs.darwin.cctools
        ];

        # Dev tools needed on the build machine
        rustDevTools = with pkgs; [
          openssl
          pkg-config
        ];

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = [
            pkgs.openssl
          ]
          ++ darwinBuildInputs;
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
        };

        # Build dependencies (cached separately)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # The main package
        changala = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        # --- Service scripts ---

        serviceBinPath = pkgs.lib.makeBinPath [
          pkgs.postgresql
          pkgs.garage
          pkgs.curl
          pkgs.jq
          pkgs.coreutils
          pkgs.gnugrep
          pkgs.gawk
          pkgs.bash
        ];

        changala-services-start = pkgs.writeShellScriptBin "changala-services-start" ''
          export PATH="${serviceBinPath}:$PATH"

          WORK_DIR="/tmp/changala-dev"
          PG_PORT="''${CHANGALA_PG_PORT:-5432}"
          S3_PORT="''${CHANGALA_S3_PORT:-9000}"
          RPC_PORT="3901"
          ADMIN_PORT="3903"

          mkdir -p "$WORK_DIR"

          echo "==> Starting Changala dev services..."

          # ── PostgreSQL ──────────────────────────────────────────────
          PG_DATA="$WORK_DIR/pgdata"
          PG_LOG="$WORK_DIR/pg.log"

          if [ ! -d "$PG_DATA" ]; then
            echo "  Initialising PostgreSQL data directory..."
            initdb -D "$PG_DATA" --no-locale --encoding=UTF8 -A trust
          fi

          if ! pg_isready -h 127.0.0.1 -p "$PG_PORT" -q 2>/dev/null; then
            echo "  Starting PostgreSQL on port $PG_PORT..."
            pg_ctl -D "$PG_DATA" -l "$PG_LOG" -o "-p $PG_PORT -k /tmp" start
            sleep 2
          else
            echo "  PostgreSQL already running on port $PG_PORT."
          fi

          # Create role and database (idempotent)
          psql -h 127.0.0.1 -p "$PG_PORT" -d postgres -c "CREATE ROLE changala WITH LOGIN SUPERUSER;" 2>/dev/null || true
          psql -h 127.0.0.1 -p "$PG_PORT" -d postgres -c "CREATE DATABASE changala OWNER changala;" 2>/dev/null || true

          # ── Garage ──────────────────────────────────────────────────
          GARAGE_DATA="$WORK_DIR/garage"
          GARAGE_META="$GARAGE_DATA/meta"
          GARAGE_BLOBS="$GARAGE_DATA/data"
          GARAGE_CFG="$WORK_DIR/garage.toml"
          GARAGE_PID="$WORK_DIR/garage.pid"

          mkdir -p "$GARAGE_META" "$GARAGE_BLOBS"

          # Generate a 64 hex-char RPC secret deterministically for dev
          RPC_SECRET="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

          cat > "$GARAGE_CFG" <<EOF
          metadata_dir = "$GARAGE_META"
          data_dir = "$GARAGE_BLOBS"
          db_engine = "sqlite"
          replication_factor = 1

          [rpc]
          bind_addr = "127.0.0.1:$RPC_PORT"
          secret = "$RPC_SECRET"

          [s3_api]
          s3_region = "us-east-1"
          api_bind_addr = "127.0.0.1:$S3_PORT"

          [s3_web]
          bind_addr = "127.0.0.1:3902"

          [admin]
          api_bind_addr = "127.0.0.1:$ADMIN_PORT"
          admin_token = "changala-dev-admin"
          EOF

          if [ -f "$GARAGE_PID" ] && kill -0 $(cat "$GARAGE_PID") 2>/dev/null; then
            echo "  Garage already running."
          else
            echo "  Starting Garage (S3 on port $S3_PORT)..."
            garage -c "$GARAGE_CFG" server &
            GARAGE_PROC=$!
            echo "$GARAGE_PROC" > "$GARAGE_PID"
            sleep 3
          fi

          ADMIN_URL="http://127.0.0.1:$ADMIN_PORT"
          GARAGE_CMD="garage -c $GARAGE_CFG"

          # Get node ID and configure layout
          NODE_ID=$($GARAGE_CMD node id 2>/dev/null | head -1 | awk '{print $1}')
          if [ -n "$NODE_ID" ]; then
            $GARAGE_CMD layout assign "$NODE_ID" -z dc1 -c 1G || true
            $GARAGE_CMD layout apply --version 1 || true
          fi

          # Create key and bucket (idempotent)
          $GARAGE_CMD key create changala-dev-key || true
          $GARAGE_CMD bucket create changala-blobs || true
          $GARAGE_CMD bucket allow --read --write --owner changala-blobs --key changala-dev-key || true

          # Extract key info
          KEY_INFO=$($GARAGE_CMD key info changala-dev-key 2>/dev/null)
          ACCESS_KEY=$(echo "$KEY_INFO" | grep "Key ID" | awk '{print $NF}')
          SECRET_KEY=$(echo "$KEY_INFO" | grep "Secret key" | awk '{print $NF}')

          echo ""
          echo "╔══════════════════════════════════════════════════════════════╗"
          echo "║             Changala Dev Services Running                   ║"
          echo "╠══════════════════════════════════════════════════════════════╣"
          echo "║                                                            ║"
          echo "║  PostgreSQL:                                               ║"
          echo "║    URL: postgresql://changala@127.0.0.1:$PG_PORT/changala           ║"
          echo "║                                                            ║"
          echo "║  Garage S3:                                                ║"
          echo "║    Endpoint:   http://127.0.0.1:$S3_PORT                        ║"
          echo "║    Access Key: $ACCESS_KEY"
          echo "║    Secret Key: $SECRET_KEY"
          echo "║                                                            ║"
          echo "╚══════════════════════════════════════════════════════════════╝"
        '';

        changala-services-stop = pkgs.writeShellScriptBin "changala-services-stop" ''
          export PATH="${serviceBinPath}:$PATH"

          WORK_DIR="/tmp/changala-dev"
          GARAGE_PID="$WORK_DIR/garage.pid"
          PG_DATA="$WORK_DIR/pgdata"

          echo "==> Stopping Changala dev services..."

          # Stop Garage
          if [ -f "$GARAGE_PID" ]; then
            PID=$(cat "$GARAGE_PID")
            if kill -0 "$PID" 2>/dev/null; then
              echo "  Stopping Garage (PID $PID)..."
              kill "$PID"
            fi
            rm -f "$GARAGE_PID"
          else
            echo "  Garage not running (no pidfile)."
          fi

          # Stop PostgreSQL
          if [ -d "$PG_DATA" ]; then
            echo "  Stopping PostgreSQL..."
            pg_ctl -D "$PG_DATA" stop 2>/dev/null || true
          else
            echo "  PostgreSQL not running (no data dir)."
          fi

          echo "  Done."
        '';

        changala-services-clean = pkgs.writeShellScriptBin "changala-services-clean" ''
          export PATH="${serviceBinPath}:$PATH"

          WORK_DIR="/tmp/changala-dev"

          echo "==> Cleaning Changala dev environment..."

          # Stop services first
          ${changala-services-stop}/bin/changala-services-stop

          # Remove working directory
          if [ -d "$WORK_DIR" ]; then
            echo "  Removing $WORK_DIR..."
            rm -rf "$WORK_DIR"
          fi

          echo "  Clean complete."
        '';

      in
      {
        packages = {
          default = changala;
          inherit
            changala
            changala-services-start
            changala-services-stop
            changala-services-clean
            ;
        };

        checks = {
          fmt = craneLib.cargoFmt {
            inherit src;
          };

          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "-- --deny warnings";
            }
          );
        };

        apps = {
          changala-services-start = flake-utils.lib.mkApp {
            drv = changala-services-start;
          };
          changala-services-stop = flake-utils.lib.mkApp {
            drv = changala-services-stop;
          };
          changala-services-clean = flake-utils.lib.mkApp {
            drv = changala-services-clean;
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = rustDevTools ++ [ rustToolchain ];
          buildInputs = [ pkgs.openssl ] ++ darwinBuildInputs;

          packages = with pkgs; [
            postgresql
            garage
            curl
            jq
          ];

          env =
            pkgs.lib.optionalAttrs isDarwin {
              # Pin the SDK and developer dir so cargo/cc/xcrun all agree
              # on the same SDK regardless of what `xcode-select -s` was
              # last pointed at. Avoids "ld: library not found for -liconv".
              SDKROOT = "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk";
              DEVELOPER_DIR = "/Library/Developer/CommandLineTools";
              LIBRARY_PATH = "${pkgs.libiconv}/lib";
              RUSTFLAGS = "-L ${pkgs.libiconv}/lib";
            }
            // pkgs.lib.optionalAttrs (!isDarwin) {
              # Use mold linker on Linux for faster builds (optional)
              # RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
            };

          shellHook = ''
            echo ""
            echo "╔══════════════════════════════════════════════════════════════╗"
            echo "║              🔗 Changala Dev Shell                         ║"
            echo "╠══════════════════════════════════════════════════════════════╣"
            echo "║                                                            ║"
            echo "║  Commands:                                                 ║"
            echo "║    changala-services-start   Start PG + Garage             ║"
            echo "║    changala-services-stop    Stop all services             ║"
            echo "║    changala-services-clean   Stop + wipe /tmp/changala-dev ║"
            echo "║                                                            ║"
            echo "║  Environment:                                              ║"
            echo "║    CHANGALA_PG_PORT  PostgreSQL port (default: 5432)       ║"
            echo "║    CHANGALA_S3_PORT  Garage S3 port  (default: 9000)       ║"
            echo "║                                                            ║"
            echo "╚══════════════════════════════════════════════════════════════╝"
            echo ""
            export PATH="${changala-services-start}/bin:${changala-services-stop}/bin:${changala-services-clean}/bin:$PATH"
          '';
        };
      }
    );
}
