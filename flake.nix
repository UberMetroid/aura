{
  description = "Minimalist Nix-built container for Aura";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustVersion;
          cargo = rustVersion;
        };

        # 1. Build the WASM frontend
        frontend = rustPlatform.buildRustPackage {
          pname = "aura-frontend";
          version = "1.0.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [
            rustVersion
            pkgs.wasm-bindgen-cli
            pkgs.trunk
            pkgs.binaryen
          ];

          buildPhase = ''
            export HOME=$TMPDIR
            cd frontend
            trunk build --release
          '';

          installPhase = ''
            mkdir -p $out/dist
            cp -r dist/* $out/dist/
          '';
        };

        # 2. Build the Axum backend
        backend = rustPlatform.buildRustPackage {
          pname = "aura-backend";
          version = "1.0.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];

          doCheck = false;

          buildPhase = ''
            cargo build --release --bin rust-search
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/rust-search $out/bin/rust-search
          '';
        };

        # 3. Create entrypoint script for launching SearXNG and backend
        auraStart = pkgs.writeShellScriptBin "aura-start" ''
          export PATH="${pkgs.openssl}/bin:$PATH"
          if [ -z "$SEARXNG_SECRET" ]; then
            export SEARXNG_SECRET=$(openssl rand -hex 32)
          fi
          export SEARXNG_SETTINGS_PATH=/app/searxng-settings.yml
          ${pkgs.searxng}/bin/searxng-run > /dev/null 2>&1 &
          exec ${backend}/bin/rust-search
        '';

        # 4. Create the layered Docker container image
        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "aura-nix";
          tag = "latest";
          
          # Run under the nobody user (UID 65534)
          config = {
            Cmd = [ "${auraStart}/bin/aura-start" ];
            WorkingDir = "/app";
            Env = [
              "PORT=4408"
              "STATIC_DIR=/app/frontend/dist"
            ];
            ExposedPorts = {
              "4408/tcp" = {};
            };
            User = "nobody:nobody";
          };

          # Create /app directory structure inside the container
          extraCommands = ''
            mkdir -p app/frontend
            cp -r ${frontend}/dist app/frontend/dist
            cp ${./searxng-settings.yml} app/searxng-settings.yml
          '';
        };

      in {
        packages = {
          inherit frontend backend auraStart dockerImage;
          default = dockerImage;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustVersion
            pkgs.trunk
            pkgs.wasm-bindgen-cli
          ];
        };
      }
    );
}
