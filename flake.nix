{
  description = "Agentic Ops Platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem
    (system: let
      overlays = [
        (import rust-overlay)
      ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      rustVersion = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustToolchain = rustVersion.override {
        extensions = [
          "rust-analyzer"
          "rust-src"
          "rustfmt"
          "clippy"
        ];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      rustSource = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          craneLib.filterCargoSources path type;
      };

      commonArgs = {
        src = rustSource;
        strictDeps = true;
        nativeBuildInputs =
          pkgs.lib.optionals pkgs.stdenv.isLinux [pkgs.clang pkgs.lld]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.libiconv pkgs.darwin.cctools];
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      individualCrateArgs =
        commonArgs
        // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml {src = rustSource;}) version;
          doCheck = false;
        };

      agentic-ops-server = craneLib.buildPackage (
        individualCrateArgs
        // {
          pname = "agentic-ops-server";
          cargoExtraArgs = "-p server";
        }
      );

      nativeBuildInputs = with pkgs;
        [
          rustToolchain
          cargo-nextest
          cargo-watch
          bats
          jq
          curl
        ]
        ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          clang
          lld
        ]
        ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          libiconv
          darwin.cctools
        ];
    in
      with pkgs; {
        packages = {
          default = agentic-ops-server;
          inherit agentic-ops-server;

          bats-runner = let
            binPath = pkgs.lib.makeBinPath [
              pkgs.bats
              pkgs.curl
              pkgs.jq
              pkgs.coreutils
              agentic-ops-server
            ];
          in
            pkgs.writeShellScriptBin "bats-runner" ''
              set -e
              export PATH="${binPath}:$PATH"
              export SERVER_BIN="${agentic-ops-server}/bin/agentic-ops-server"

              cleanup() {
                if [ -f .server.pid ]; then
                  kill "$(cat .server.pid)" 2>/dev/null || true
                  rm -f .server.pid
                fi
              }
              trap cleanup EXIT

              # Set test OAuth credentials so auth routes are mounted
              export GITHUB_CLIENT_ID="''${GITHUB_CLIENT_ID:-test-client-id}"
              export GITHUB_CLIENT_SECRET="''${GITHUB_CLIENT_SECRET:-test-client-secret}"

              echo "Starting agentic-ops-server..."
              $SERVER_BIN &
              echo "$!" > .server.pid

              echo "Waiting for server to be ready..."
              for i in $(seq 1 30); do
                if curl -sf http://localhost:4200/graphql -X POST \
                  -H "Content-Type: application/json" \
                  -d '{"query":"{ health }"}' > /dev/null 2>&1; then
                  echo "Server is ready"
                  break
                fi
                if [ "$i" -eq 30 ]; then
                  echo "Server failed to start"
                  exit 1
                fi
                sleep 1
              done

              export TERM="''${TERM:-dumb}"
              echo "Running bats tests..."
              bats bats/*.bats

              echo "Tests completed successfully!"
            '';

          bats = pkgs.writeShellScriptBin "bats" ''
            exec ${self.packages.${system}.bats-runner}/bin/bats-runner "$@"
          '';
        };

        checks = {
          workspace-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          workspace-fmt = craneLib.cargoFmt {
            src = rustSource;
          };
        };

        devShells.default = mkShell {
          inherit nativeBuildInputs;
        };
      });
}
