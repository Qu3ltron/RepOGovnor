{ config, lib, pkgs, ... }:

let
  cfg = config.services.agent-governance-auto-update;

  # The flake input name that points to the Governance-plugin repo.
  # Consumers add this repo as a flake input; the timer updates it.
  flakeInput = cfg.flakeInput;

  # Path to the flake whose lock file contains the governance-plugin input.
  flakeDir = cfg.flakeDir;

  # One-shot service script: update the flake lock and rebuild if changed.
  updateScript = pkgs.writeShellScriptBin "agent-governance-update" ''
    set -euo pipefail

    LOCK_FILE="${flakeDir}/flake.lock"

    if [[ ! -f "$LOCK_FILE" ]]; then
      echo "agent-governance-update: no flake.lock at $LOCK_FILE" >&2
      exit 1
    fi

    # Snapshot the current lock hash for the governance-plugin input.
    old_rev="$(${pkgs.jq}/bin/jq -r \
      ".nodes.\"${flakeInput}\".locked.rev // empty" \
      "$LOCK_FILE")"

    if [[ -z "$old_rev" ]]; then
      echo "agent-governance-update: input ${flakeInput} not found in lock" >&2
      exit 1
    fi

    # Update only the governance-plugin input.
    ${pkgs.nix}/bin/nix flake lock \
      --update-input "${flakeInput}" \
      "$flakeDir" 2>&1

    # Snapshot the new lock hash.
    new_rev="$(${pkgs.jq}/bin/jq -r \
      ".nodes.\"${flakeInput}\".locked.rev // empty" \
      "$LOCK_FILE")"

    if [[ "$old_rev" == "$new_rev" ]]; then
      echo "agent-governance-update: ${flakeInput} unchanged at $old_rev"
      exit 0
    fi

    echo "agent-governance-update: ${flakeInput} updated $old_rev -> $new_rev"

    # Rebuild the system so dependent services pick up the new plugin.
    ${pkgs.nixos-rebuild}/bin/nixos-rebuild switch --flake "$flakeDir" 2>&1
  '';

in
{
  options.services.agent-governance-auto-update = {
    enable = lib.mkEnableOption "automatic updates for the agent-governance plugin flake input";

    flakeDir = lib.mkOption {
      type = lib.types.str;
      default = "/home/hasnamuss";
      description = "Path to the flake whose lock file contains the governance-plugin input";
    };

    flakeInput = lib.mkOption {
      type = lib.types.str;
      default = "governance-plugin";
      description = "Name of the flake input pointing to the Governance-plugin repo";
    };

    devLockFile = lib.mkOption {
      type = lib.types.str;
      default = "/run/agent-governance/no-auto-update";
      description = "When this file exists, auto-update is suppressed (active development guard)";
    };

    interval = lib.mkOption {
      type = lib.types.str;
      default = "30min";
      description = "How often to check for updates (systemd OnUnitActiveSec)";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "hasnamuss";
      description = "User to run the update service as";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "users";
      description = "Group to run the update service as";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.agent-governance-update = {
      description = "Update agent-governance plugin flake input and rebuild";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];

      # Skip when the dev lock file exists (active development).
      unitConfig.ConditionPathExists = "!${cfg.devLockFile}";

      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${updateScript}/bin/agent-governance-update";
        LockPersonality = true;
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectHome = false;
        ProtectSystem = "strict";
        ReadWritePaths = [
          "${cfg.flakeDir}"
          "/nix/var/nix"
          "/etc/nixos"
        ];
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
      };
    };

    systemd.timers.agent-governance-update = {
      description = "Periodically check for agent-governance plugin updates";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnBootSec = "5min";
        OnUnitActiveSec = cfg.interval;
        AccuracySec = "60s";
        Persistent = true;
        Unit = "agent-governance-update.service";
      };
    };
  };
}
