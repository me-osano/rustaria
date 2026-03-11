# Home Manager module for rustaria
# Import this module in your home.nix or flake-based Home Manager config.
#
# Usage (flake):
#   inputs.rustaria.url = "github:me-osano/rustaria";
#   imports = [ inputs.rustaria.homeManagerModules.default ];
#
# Usage (standalone):
#   imports = [ ./path/to/rustaria/home-manager/rustaria.nix ];
#
# Then configure:
#   programs.rustaria = {
#     enable = true;
#     settings = {
#       general.download_dir = "~/Downloads";
#       general.max_concurrent = 5;
#       aria2.rpc_secret = "your-secret";
#     };
#   };

{ config, lib, pkgs, ... }:

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
      default = pkgs.callPackage ../default.nix { };
      defaultText = literalExpression "pkgs.rustaria";
      description = "The rustaria package to use.";
    };

    settings = mkOption {
      type = tomlFormat.type;
      default = { };
      example = literalExpression ''
        {
          general = {
            download_dir = "~/Downloads";
            max_concurrent = 5;
          };
          aria2 = {
            rpc_url = "http://127.0.0.1:6800/jsonrpc";
          };
        }
      '';
      description = ''
        Configuration for rustaria. Will be serialized to TOML and
        written to <filename>~/.config/rustaria/config.toml</filename>.
      '';
    };

    aria2 = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Whether to ensure aria2 is installed alongside rustaria.";
      };
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ] ++ optional cfg.aria2.enable pkgs.aria2;

    xdg.configFile."rustaria/config.toml" = mkIf (cfg.settings != { }) {
      source = tomlFormat.generate "rustaria-config" cfg.settings;
    };
  };
}
