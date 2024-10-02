{
  description = "A basic flake with a shell.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs ={ self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem
    (system:
      let
        overlays = [ ( import rust-overlay )];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [ 
            (rust-bin.stable.latest.default.override {
              extensions = ["rust-src"];
            })

            pkgs.openssl
            pkgs.pkg-config

            pkgs.go

            nodejs nodePackages.pnpm
          ];
        };
      }
    );
}
