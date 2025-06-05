{
  description = "LLiMinal - A TUI tool for interfacing with LLMs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem ( system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.stable.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            # Rust toolchain
            rustVersion
            rust-analyzer
            clippy
            rustfmt

            # Development tools
            # pkg-config
            # openssl

            # Terminal UI libraries dependencies
            # ncurses

            # Optional but helpful tools
            cargo-edit     # For cargo add, cargo rm, etc.
            cargo-watch    # For auto-recompilation during development
          ];
        };
      }
    );
}
