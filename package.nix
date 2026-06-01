{ lib
, rustPlatform
, installShellFiles
}:

let
  sourceRoot = toString ./.;

  # Only include files needed for the Rust build; runtime assets are installed
  # explicitly below so consumers do not need the mutable checkout.
  govSrc = lib.cleanSourceWith {
    src = ./.;
    filter = path: _type:
      let
        rel = lib.removePrefix (sourceRoot + "/") (toString path);
        keep = base: rel != base && !(lib.hasPrefix "${base}/" rel);
      in
      builtins.all (base: keep base) [
        ".antigravitycli"
        ".claude"
        ".codex"
        ".cursor"
        ".git"
        "target"
        "plugins"
      ];
  };

in
rustPlatform.buildRustPackage {
  pname = "task-registry-flow";
  version = "2.0.1";

  src = govSrc;

  # The inner crate has its own Cargo.lock. buildRustPackage validates
  # against a root-level copy, so symlink it into place after unpack.
  cargoLock = {
    lockFile = ./rust/task-registry-flow-cli/Cargo.lock;
  };

  postUnpack = ''
    ln -sf rust/task-registry-flow-cli/Cargo.lock source/Cargo.lock
  '';

  # Point buildRustPackage at the inner Cargo.toml
  buildAndTestSubdir = "rust/task-registry-flow-cli";

  nativeBuildInputs = [
    installShellFiles
  ];

  # The Rust binary is the primary artifact. Companion scripts are
  # installed alongside it so hook configurations can reference them at
  # known paths under the Nix store output.
  postInstall = ''
    asset_root=$out/share/agent-governance
    mkdir -p "$asset_root"

    # Install the mutation hook script that agent hooks (Codex, Claude
    # Code, Cursor, Antigravity) invoke before mutation tool calls.
    cp ${./tools/agent-governance/pre-tool-use-gap-closure.sh} $out/bin/pre-tool-use-gap-closure.sh
    chmod +x $out/bin/pre-tool-use-gap-closure.sh

    # Install the posture check script for CI and manual verification.
    cp ${./scripts/status.sh} $out/bin/agent-governance-status
    chmod +x $out/bin/agent-governance-status

    cp -R ${./templates} "$asset_root/templates"
    cp -R ${./skills} "$asset_root/skills"
    cp -R ${./hooks} "$asset_root/hooks"
    cp -R ${./modules} "$asset_root/modules"
    cp ${./MANIFEST.toml} "$asset_root/MANIFEST.toml"
    cp ${./REQUIREMENTS.toml} "$asset_root/REQUIREMENTS.toml"
    cp ${./project.config.example.toml} "$asset_root/project.config.example.toml"
    cp ${./README.md} "$asset_root/README.md"
    mkdir -p "$asset_root/docs/releases"
    cp ${./docs/runtime-schemas.md} "$asset_root/docs/runtime-schemas.md"
    cp ${./docs/releases/v2.md} "$asset_root/docs/releases/v2.md"
  '';

  meta = with lib; {
    description = "Task registry CLI for the agent-governance plugin";
    mainProgram = "task-registry-flow";
    license = lib.licenses.mit;
    platforms = platforms.linux;
  };
}
