{
  description = "Deterministic Rust + WASM + Tailwind dev shell";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { system, pkgs, ... }:
        let
          rust = with inputs.fenix.packages.${system}; combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
          ];
          env = {
            LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib:${pkgs.aravis.lib}/lib:${pkgs.glib.out}/lib";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig";
            CC = "${pkgs.clang}/bin/clang";
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          };
          packages = [
            rust
            pkgs.clang
            pkgs.pkg-config
            pkgs.just
            pkgs.trunk
            pkgs.glib
            pkgs.aravis
            pkgs.tailwindcss_4
            pkgs.sass
          ];
          greet = ''
            echo "===================================="
            echo " Welcome to the deterministic dev shell! "
            echo "===================================="
            rustc --version && cargo --version && trunk --version && tailwindcss --version
          '';
          policy = pkgs.writeText "policy.json" ''{"default":[{"type":"insecureAcceptAnything"}]}'';
          envSetup = pkgs.lib.concatStringsSep "\n"
            (pkgs.lib.mapAttrsToList (k: v: "export ${k}=${v}") env);
        in
        {
          devShells.default = pkgs.mkShell {
            buildInputs = packages;
            inherit (env) LD_LIBRARY_PATH PKG_CONFIG_PATH;
            shellHook = greet;
          };
          packages.isolated = pkgs.dockerTools.buildImage {
            name = "isolated-dev";
            tag = "latest";
            copyToRoot = pkgs.buildEnv {
              name = "isolated-env";
              paths = packages ++ [
                pkgs.bashInteractive
                pkgs.opencode
                pkgs.coreutils
                (pkgs.writeScriptBin "entrypoint.sh" ''
                  #!${pkgs.bashInteractive}/bin/bash
                  ${envSetup}
                  ${greet}
                  exec ${pkgs.bashInteractive}/bin/bash
                '')
              ];
              pathsToLink = [ "/bin" "/lib" "/include" "/share" ];
            };
            config = {
              Env = pkgs.lib.mapAttrsToList (k: v: "${k}=${v}") env ++ [ "HOME=/root" ];
              Cmd = [ "/bin/entrypoint.sh" ];
              WorkingDir = "/workspace";
            };
          };
          apps.isolated = {
            type = "app";
            program = toString (pkgs.writeShellScript "run-isolated" ''

              ${pkgs.podman}/bin/podman load \
                --signature-policy ${policy} \
                --input ${inputs.self.packages.${system}.isolated}
              ${pkgs.podman}/bin/podman run --rm -it \
                --network=slirp4netns \
                --tmpfs /tmp \
                -v "..:/workspace:z" \
                -e HOME=/root \
                isolated-dev:latest /bin/entrypoint.sh
            '');
          };
        };
    };
}
