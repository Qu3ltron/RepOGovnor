{ config, lib, pkgs, ... }:

let
  cfg = config.services.agent-governance-auto-update;

  # The flake input name that points to the RepOGovnor repo.
  # Consumers add this repo as a flake input; the timer updates it.
  flakeInput = cfg.flakeInput;
  flakeInputNameSafe = builtins.match "[A-Za-z0-9._-]+" flakeInput != null;

  # Path to the flake whose lock file contains the governance-plugin input.
  flakeDir = cfg.flakeDir;

  validationCommand =
    if cfg.validationCommand == ""
    then "${pkgs.nix}/bin/nix flake check --no-build ${lib.escapeShellArg flakeDir}"
    else cfg.validationCommand;

  rebuildCommand =
    if cfg.rebuildCommand == ""
    then "${pkgs.nixos-rebuild}/bin/nixos-rebuild switch --flake ${lib.escapeShellArg flakeDir}"
    else cfg.rebuildCommand;

  healthCommand =
    if cfg.healthCommand == ""
    then "true"
    else cfg.healthCommand;

  # One-shot service script: update the flake lock and rebuild if changed.
  updateScript = pkgs.writeShellScriptBin "agent-governance-update" ''
    set -euo pipefail

    LOCK_FILE="${flakeDir}/flake.lock"
    flake_input=${lib.escapeShellArg flakeInput}
    backup_lock="$(mktemp)"
    validation_cmd=${lib.escapeShellArg validationCommand}
    rebuild_cmd=${lib.escapeShellArg rebuildCommand}
    health_cmd=${lib.escapeShellArg healthCommand}

    cleanup() {
      rm -f "$backup_lock"
    }
    trap cleanup EXIT

    rollback_lock() {
      local reason="$1"
      echo "agent-governance-update: rolling back lock after $reason" >&2
      cp "$backup_lock" "$LOCK_FILE"
      if ! bash -lc "$rebuild_cmd"; then
        echo "agent-governance-update: rollback rebuild failed" >&2
      fi
    }

    if [[ ! -f "$LOCK_FILE" ]]; then
      echo "agent-governance-update: no flake.lock at $LOCK_FILE" >&2
      exit 1
    fi

    cp "$LOCK_FILE" "$backup_lock"

    # Snapshot the current lock hash for the governance-plugin input.
    old_rev="$(${pkgs.jq}/bin/jq -r \
      --arg input "$flake_input" \
      '.nodes[$input].locked.rev // empty' \
      "$LOCK_FILE")"

    if [[ -z "$old_rev" ]]; then
      echo "agent-governance-update: input $flake_input not found in lock" >&2
      exit 1
    fi

    # Update only the governance-plugin input.
    if ! ${pkgs.nix}/bin/nix flake lock \
      --update-input "$flake_input" \
      "$flakeDir" 2>&1; then
      rollback_lock "flake lock failure"
      exit 1
    fi

    # Snapshot the new lock hash.
    new_rev="$(${pkgs.jq}/bin/jq -r \
      --arg input "$flake_input" \
      '.nodes[$input].locked.rev // empty' \
      "$LOCK_FILE")"

    if [[ "$old_rev" == "$new_rev" ]]; then
      echo "agent-governance-update: $flake_input unchanged at $old_rev"
      exit 0
    fi

    echo "agent-governance-update: $flake_input updated $old_rev -> $new_rev"

    if ! bash -lc "$validation_cmd"; then
      rollback_lock "validation failure"
      exit 1
    fi

    if ! bash -lc "$rebuild_cmd"; then
      rollback_lock "rebuild failure"
      exit 1
    fi

    if ! bash -lc "$health_cmd"; then
      rollback_lock "health check failure"
      exit 1
    fi
  '';

in
{
  options.services.agent-governance-auto-update = {
    enable = lib.mkEnableOption "automatic updates for the agent-governance plugin flake input";

    flakeDir = lib.mkOption {
      type = lib.types.str;
      default = "/etc/nixos";
      description = "Path to the flake whose lock file contains the governance-plugin input";
    };

    flakeInput = lib.mkOption {
      type = lib.types.str;
      default = "governance-plugin";
      description = "Name of the flake input pointing to the RepOGovnor repo";
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
      default = "root";
      description = "User to run the update service as; root is required for nixos-rebuild switch";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "root";
      description = "Group to run the update service as";
    };

    validationCommand = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Command run after lock update and before rebuild; default runs nix flake check --no-build";
    };

    rebuildCommand = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Root-safe rebuild command; default runs nixos-rebuild switch for flakeDir";
    };

    healthCommand = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Post-rebuild health command; default is true";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = flakeInputNameSafe;
        message = "services.agent-governance-auto-update.flakeInput must match [A-Za-z0-9._-]+";
      }
    ];

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
