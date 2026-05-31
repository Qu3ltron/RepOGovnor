{
  description = "Portable agent-governance plugin (Codex + Claude Code + Cursor + Antigravity)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          f system (import nixpkgs { inherit system; })
        );
    in
    {
      packages = forAllSystems (_system: pkgs:
        let
          task-registry-flow = pkgs.callPackage ./package.nix { };
        in
        {
          default = task-registry-flow;
          task-registry-flow = task-registry-flow;
        });

      apps = forAllSystems (system: _pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/task-registry-flow";
          meta.description = "Governance task registry CLI";
        };
      });

      devShells = forAllSystems (_system: pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.clippy
            pkgs.rustc
            pkgs.rustfmt
            pkgs.python3
          ];
          shellHook = ''
            echo "Governance-plugin dev shell"
            echo "  cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
            echo "  cargo run --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- validate"
          '';
        };
      });

      formatter = forAllSystems (_system: pkgs: pkgs.nixpkgs-fmt);

      nixosModules = {
        agent-governance = import ./modules/nixos/agent-governance.nix;
        auto-update = import ./modules/nixos/agent-governance-auto-update.nix;
      };
    };
}
