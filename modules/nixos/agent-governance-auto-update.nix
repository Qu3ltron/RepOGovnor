{ config, lib, pkgs, ... }:

let
  cfg = config.services.agent-governance-auto-update;

  # ---- flake mode -------------------------------------------------------
  flakeScript = pkgs.writeShellScriptBin "agent-governance-update" ''
    set -euo pipefail
    LOCK_FILE="${cfg.flakeDir}/flake.lock"
    if [[ ! -f "$LOCK_FILE" ]]; then
      echo "agent-governance-update: no flake.lock at $LOCK_FILE" >&2
      exit 1
    fi
    old_rev="$(${pkgs.jq}/bin/jq -r \
      ".nodes.\"${cfg.flakeInput}\".locked.rev // empty" "$LOCK_FILE")"
    if [[ -z "$old_rev" ]]; then
      echo "agent-governance-update: input ${cfg.flakeInput} not in lock" >&2
      exit 1
    fi
    ${pkgs.nix}/bin/nix flake lock --update-input "${cfg.flakeInput}" "${cfg.flakeDir}"
    new_rev="$(${pkgs.jq}/bin/jq -r \
      ".nodes.\"${cfg.flakeInput}\".locked.rev // empty" "$LOCK_FILE")"
    if [[ "$old_rev" == "$new_rev" ]]; then
      echo "agent-governance-update: ${cfg.flakeInput} unchanged at $old_rev"
      exit 0
    fi
    echo "agent-governance-update: ${cfg.flakeInput} $old_rev -> $new_rev"
    exec ${pkgs.nixos-rebuild}/bin/nixos-rebuild switch --flake "${cfg.flakeDir}"
  '';

  # ---- fetch-tree mode --------------------------------------------------
  fetchTreeScript = pkgs.writeShellScriptBin "agent-governance-update" ''
    set -euo pipefail
    NIX_FILE="${cfg.nixFile}"
    REV_VAR="${cfg.revVariable}"
    REMOTE="${cfg.remoteUrl}"

    old_rev="$(${pkgs.gawk}/bin/awk \
      "/^[[:space:]]*$REV_VAR[[:space:]]*=/ { gsub(/[^a-f0-9]/, \"\", \$NF); print \$NF; exit }" \
      "$NIX_FILE")"
    if [[ -z "$old_rev" ]]; then
      echo "agent-governance-update: could not read $REV_VAR from $NIX_FILE" >&2
      exit 1
    fi

    new_rev="$(${pkgs.git}/bin/git ls-remote "$REMOTE" refs/heads/main \
      | ${pkgs.gawk}/bin/awk '{print $1}')"
    if [[ -z "$new_rev" ]]; then
      echo "agent-governance-update: could not fetch HEAD from $REMOTE" >&2
      exit 1
    fi

    if [[ "$old_rev" == "$new_rev" ]]; then
      echo "agent-governance-update: $REV_VAR unchanged at $old_rev"
      exit 0
    fi

    echo "agent-governance-update: $REV_VAR $old_rev -> $new_rev"
    ${pkgs.gnused}/bin/sed -i \
      "s/$REV_VAR = \"[^\"]*\"/$REV_VAR = \"$new_rev\"/" \
      "$NIX_FILE"
    exec ${pkgs.nixos-rebuild}/bin/nixos-rebuild switch
  '';

  updateScript =
    if cfg.mode == "fetch-tree" then fetchTreeScript
    else flakeScript;

in
{
  options.services.agent-governance-auto-update = {
    enable = lib.mkEnableOption "automatic updates for the agent-governance plugin";

    mode = lib.mkOption {
      type = lib.types.enum [ "flake" "fetch-tree" ];
      default = "flake";
      description = "Update strategy: flake (nix flake lock) or fetch-tree (git ls-remote + sed bump)";
    };

    flakeDir = lib.mkOption {
      type = lib.types.str;
      default = "/home/hasnamuss";
      description = "Path to flake (flake mode) or directory (fetch-tree mode)";
    };

    flakeInput = lib.mkOption {
      type = lib.types.str;
      default = "governance-plugin";
      description = "Flake input name pointing to the Governance-plugin repo (flake mode)";
    };

    nixFile = lib.mkOption {
      type = lib.types.str;
      default = "/home/hasnamuss/reclaimed/system/reclaimed-resources.nix";
      description = "Path to .nix file containing the pinned rev (fetch-tree mode)";
    };

    revVariable = lib.mkOption {
      type = lib.types.str;
      default = "governancePluginRev";
      description = "Nix variable name holding the pinned git rev (fetch-tree mode)";
    };

    remoteUrl = lib.mkOption {
      type = lib.types.str;
      default = "ssh://git@github.com/Qu3ltron/Governance-plugin.git";
      description = "Git remote URL to fetch latest commit from (fetch-tree mode)";
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
      description = "Update agent-governance plugin and rebuild";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
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
