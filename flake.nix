{
  description = "Deterministic Rust + WASM + Tailwind dev shell";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devtools.url = "github:mineichen/nix-devtools";
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { system, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            config.allowUnfreePredicate = pkg: builtins.elem (inputs.nixpkgs.lib.getName pkg) [ "claude-code" ];
          };
          rust = with inputs.fenix.packages.${system}; combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
          ];
          envVars = {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.openssl
              pkgs.aravis
              pkgs.glib
              pkgs.clang
            ];
            PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
              pkgs.openssl.dev
              pkgs.glib.dev
              pkgs.aravis.dev
            ];
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          };
          commonBuildInputs = [
            rust
            pkgs.clang
            pkgs.pkg-config
            pkgs.just
            pkgs.trunk
            pkgs.git
            pkgs.glib
            pkgs.aravis
            pkgs.tailwindcss_4
            pkgs.sass
            pkgs.mdbook
	  ];
          greet = ''
            echo "===================================="
            echo " Welcome to the deterministic dev shell! "
            echo "===================================="
            rustc --version && cargo --version && trunk --version
          '';
          isolated = inputs.devtools.lib.isolated {
            inherit pkgs inputs;
            extraMounts = [ "../imbuf" "../imask" "../pilatus" "../pilatus-aravis" "../imanot" ];
            name = "pilatus-leptos-isolated-dev";
            packages = commonBuildInputs ++ [
              pkgs.busybox
              pkgs.ripgrep
              pkgs.cargo-shear
              pkgs.opencode
            ];
            claude.pluginDirs = [
              (inputs.devtools.lib.claudeRustAnalyzer { inherit pkgs; rust-analyzer = rust; })
            ];
            env = envVars;
            shellHook = greet;
          };
        in
        {
          devShells.default = pkgs.mkShell({
            buildInputs = commonBuildInputs;
            shellHook = greet;
          } // envVars);
          apps.isolated = {
            type = "app";
            program = pkgs.lib.getExe isolated;
          };
        };
    };
}
