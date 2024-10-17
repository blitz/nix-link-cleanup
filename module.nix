{ config, lib, ... }:
let
  cfg = config.programs.nix-link-cleanup;
in
{
  options.programs.nix-link-cleanup = {
    enable = lib.mkEnableOption "nix-link-cleanup";

    package = lib.mkOption {
      defaultText = "The package to use";
      type = lib.types.package;
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      cfg.package
    ];
  };
}
