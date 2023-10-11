{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust =
          (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain).override {
            extensions = [ "rust-src" ];
          };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rust ] ++ (with pkgs; [ rust-analyzer ]);
          RUST_BACKTRACE = 1;
        };
      });
}
