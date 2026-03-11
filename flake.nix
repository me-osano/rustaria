{
  description = "rustaria – Rust download manager powered by aria2";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          sqlite
          aria2
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          shellHook = ''
            echo "🦀 rustaria development shell"
            echo "   Rust: $(rustc --version)"
            echo "   aria2: $(aria2c --version | head -1)"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rustaria";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          inherit nativeBuildInputs buildInputs;

          meta = with pkgs.lib; {
            description = "Rust download manager powered by aria2";
            homepage = "https://github.com/me-osano/rustaria";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      }
    ) // {
      homeManagerModules.default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.programs.rustaria;
          tomlFormat = pkgs.formats.toml { };
        in
        {
          options.programs.rustaria = {
            enable = mkEnableOption "rustaria download manager";

            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.default;
              description = "The rustaria package to use.";
            };

            settings = mkOption {
              type = tomlFormat.type;
              default = { };
              description = "Configuration for rustaria (written to config.toml).";
            };
          };

          config = mkIf cfg.enable {
            home.packages = [ cfg.package pkgs.aria2 ];

            xdg.configFile."rustaria/config.toml" = mkIf (cfg.settings != { }) {
              source = tomlFormat.generate "rustaria-config" cfg.settings;
            };
          };
        };
    };
}
