{
  description = "Deterministic Rust + WASM + Tailwind dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = with fenix.packages.${system}; combine [
          stable.toolchain
          targets.wasm32-unknown-unknown.stable.rust-std
        ];
        defaultLdLibraryPath = "${pkgs.openssl.out}/lib;${pkgs.aravis.lib}/lib;${pkgs.glib.out}/lib";
        defaultPkgConfigPath = "${pkgs.openssl.dev}/lib/pkgconfig;${pkgs.glib.dev}/lib/pkgconfig";
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.pkg-config
            pkgs.just
            pkgs.trunk
            pkgs.glib
            pkgs.aravis
            pkgs.tailwindcss_4
            pkgs.sass
          ];

          shellHook = ''
            echo "===================================="
            echo " Welcome to the deterministic dev shell! "
            echo "===================================="

            export LD_LIBRARY_PATH="${defaultLdLibraryPath}"
            export PKG_CONFIG_PATH="${defaultPkgConfigPath}"
            
            rustc --version
            cargo --version
            trunk --version
            tailwindcss --version
          '';
        };
      });
}
