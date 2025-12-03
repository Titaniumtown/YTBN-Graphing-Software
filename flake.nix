{
  description = "YTBN Graphing Software - Web-compatible graphing calculator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    simon-egui = {
      url = "github:Titaniumtown/egui/b63c21d70150f1b414370f0f9a8af56e886662f4";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, simon-egui }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Use nightly rust with wasm32 target
        rustToolchain = pkgs.rust-bin.nightly."2025-05-01".default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        # Build wasm-bindgen-cli matching the version in Cargo.lock (0.2.106)
        wasm-bindgen-cli = rustPlatform.buildRustPackage rec {
          pname = "wasm-bindgen-cli";
          version = "0.2.106";

          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-M6WuGl7EruNopHZbqBpucu4RWz44/MSdv6f0zkYw+44=";
          };

          cargoHash = "sha256-ElDatyOwdKwHg3bNH/1pcxKI7LXkhsotlDPQjiLHBwA=";

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.curl
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          # Tests require network access
          doCheck = false;
        };

        # Create a combined source with the main project and dependencies
        combinedSrc = pkgs.stdenv.mkDerivation {
          name = "ytbn-combined-src";
          phases = [ "installPhase" ];
          installPhase = ''
            mkdir -p $out/integral_site_rust
            mkdir -p $out/simon-egui

            cp -r ${./.}/* $out/integral_site_rust/
            cp -r ${simon-egui}/* $out/simon-egui/

            chmod -R u+w $out
          '';
        };

        # Build the wasm library using rustPlatform
        wasmLib = rustPlatform.buildRustPackage {
          pname = "ytbn-graphing-software-wasm";
          version = "0.1.0";

          src = combinedSrc;
          sourceRoot = "${combinedSrc.name}/integral_site_rust";

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "egui_plot-0.34.0" = "sha256-lk0yeljsvkHzF0eLD5llQ+05DycPqG2jGzhBvQ0X6Qw=";
            };
          };

          nativeBuildInputs = with pkgs; [
            python3Packages.fonttools
            pkg-config
            clang
          ];

          buildInputs = with pkgs; [
            openssl
            zstd
          ];

          buildPhase = ''
            runHook preBuild

            export HOME=$TMPDIR

            cargo build \
              --release \
              --lib \
              --target wasm32-unknown-unknown

            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out/lib
            cp target/wasm32-unknown-unknown/release/*.wasm $out/lib/
            runHook postInstall
          '';

          doCheck = false;
        };

        # Final web package with wasm-bindgen processing
        ytbn-graphing-software-web = pkgs.stdenv.mkDerivation {
          pname = "ytbn-graphing-software-web";
          version = "0.1.0";

          src = ./.;

          nativeBuildInputs = [
            wasm-bindgen-cli
            pkgs.binaryen
          ];

          buildPhase = ''
            runHook preBuild

            # Generate JS bindings
            wasm-bindgen ${wasmLib}/lib/ytbn_graphing_software.wasm \
              --out-dir out \
              --out-name ytbn_graphing_software \
              --target web \
              --no-typescript

            # Optimize wasm (enable features used by modern rust wasm targets)
            wasm-opt out/ytbn_graphing_software_bg.wasm \
              -O2 --fast-math \
              --enable-bulk-memory \
              --enable-nontrapping-float-to-int \
              --enable-sign-ext \
              --enable-mutable-globals \
              -o out/ytbn_graphing_software_bg.wasm

            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall

            mkdir -p $out

            # Copy wasm and js files
            cp out/ytbn_graphing_software_bg.wasm $out/
            cp out/ytbn_graphing_software.js $out/

            # Copy static web assets
            cp www/index.html $out/
            cp www/manifest.json $out/
            cp www/sw.js $out/

            # Copy logo
            cp assets/logo.svg $out/

            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "Web-compatible graphing calculator similar to Desmos";
            homepage = "https://github.com/Titaniumtown/YTBN-Graphing-Software";
            license = licenses.agpl3Only;
            platforms = platforms.all;
          };
        };
      in
      {
        packages = {
          default = ytbn-graphing-software-web;
          web = ytbn-graphing-software-web;
          wasm = wasmLib;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            wasm-bindgen-cli
            binaryen
            python3Packages.fonttools
            rust-analyzer
            pkg-config
            clang

            # Runtime deps for native builds
            libxkbcommon
            libGL
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
          ];

          buildInputs = with pkgs; [
            openssl
            zstd
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            libxkbcommon
            libGL
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
          ]);
        };
      }
    );
}
