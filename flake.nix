{
  description = "bart-rs — a Rust CLI and library for BART's GTFS schedule and realtime data";

  inputs = {
    # GitHub rather than the (now deprecated) FlakeHub tarball, so that a
    # fresh `git clone` does not depend on FlakeHub still being up.
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default self.overlays.default ];
        };
      });
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [ "rust-src" "rustfmt" "clippy" ];
            };
      };

      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
          ];

          env = {
            # Required by rust-analyzer.
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";

            # Show backtraces by default; a CLI that panics should be debuggable.
            RUST_BACKTRACE = "1";
          };
        };
      });

      packages = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "bart-rs";
          version = "0.1.0";

          # The flake source is the git tree, so `Cargo.lock` must be tracked.
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      });
    };
}
