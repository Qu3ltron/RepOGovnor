{ config, lib, pkgs, ... }:

let
  cfg = config.services.agent-governance;
in
{
  options.services.agent-governance = {
    enable = lib.mkEnableOption "agent-governance runtime package";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ../../package.nix { };
      defaultText = lib.literalExpression "pkgs.callPackage <Governance-plugin>/package.nix { }";
      description = "Agent-governance package that provides the CLI and runtime assets.";
    };

    assetRootVariable = lib.mkOption {
      type = lib.types.str;
      default = "AGENT_GOVERNANCE_ASSET_ROOT";
      description = "Environment variable exposing the packaged runtime asset root.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    environment.variables.${cfg.assetRootVariable} = "${cfg.package}/share/agent-governance";
  };
}
