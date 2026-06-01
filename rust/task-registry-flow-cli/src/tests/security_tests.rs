use super::*;
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
}

fn write_executable(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
    set_executable(path, true);
}

#[test]
fn security_hook_accepts_canonical_env() {
    let root = temp_root("security-hook-canonical");
    fs::create_dir_all(root.join(".codex/scripts")).unwrap();
    fs::write(
        root.join(".codex/governance-cli.env"),
        "GOVERNANCE_VERIFY_HOOK_CMD=\".codex/scripts/task-registry verify-mutation-hook\"\n",
    )
    .unwrap();
    write_executable(
        &root.join(".codex/scripts/task-registry"),
        "#!/bin/sh\nprintf '%s\\n' \"$*\" > hook-args.txt\n",
    );

    let output = Command::new("bash")
        .arg(repo_root().join("tools/agent-governance/pre-tool-use-gap-closure.sh"))
        .arg("--format")
        .arg("cursor")
        .current_dir(&root)
        .env_remove("GOVERNANCE_VERIFY_HOOK_CMD")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(root.join("hook-args.txt")).unwrap(),
        "verify-mutation-hook --format cursor\n"
    );
}

#[test]
fn security_hook_rejects_malicious_env() {
    let root = temp_root("security-hook-malicious");
    let marker = root.join("pwned");
    fs::create_dir_all(root.join(".codex/scripts")).unwrap();
    fs::write(
        root.join(".codex/governance-cli.env"),
        format!(
            "GOVERNANCE_VERIFY_HOOK_CMD=\".codex/scripts/task-registry verify-mutation-hook\"; touch {}\n",
            marker.display()
        ),
    )
    .unwrap();
    write_executable(
        &root.join(".codex/scripts/task-registry"),
        "#!/bin/sh\ntouch wrapper-ran\n",
    );

    let output = Command::new("bash")
        .arg(repo_root().join("tools/agent-governance/pre-tool-use-gap-closure.sh"))
        .arg("--format")
        .arg("antigravity")
        .current_dir(&root)
        .env_remove("GOVERNANCE_VERIFY_HOOK_CMD")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("noncanonical GOVERNANCE_VERIFY_HOOK_CMD"),
        "{stdout}"
    );
    assert!(!marker.exists());
    assert!(!root.join("wrapper-ran").exists());
}

#[test]
fn security_installer_rejects_unsafe_hook_paths() {
    let repo = repo_root();
    if !command_exists("python3") {
        let script = fs::read_to_string(repo.join("scripts/render-from-config.sh")).unwrap();
        assert!(script.contains("validate_hook_script_path"));
        assert!(script.contains("tools/agent-governance/"));
        return;
    }
    for unsafe_path in [
        "/tmp/agent-governance-hook.sh",
        "../outside.sh",
        "tools/agent-governance/hook.sh;touch-pwned",
        ".github/workflows/pwn.yml",
    ] {
        let target = temp_root("security-installer-target");
        let config = target.join("project.config.toml");
        let body = fs::read_to_string(repo.join("project.config.example.toml"))
            .unwrap()
            .replace(
                "repo_root = \"{{AUTO_REPO_ROOT}}\"",
                &format!("repo_root = \"{}\"", target.display()),
            )
            .replace(
                "hook_script_path = \"tools/agent-governance/pre-tool-use-gap-closure.sh\"",
                &format!("hook_script_path = \"{unsafe_path}\""),
            );
        fs::write(&config, body).unwrap();

        let output = Command::new("bash")
            .arg(repo.join("scripts/render-from-config.sh"))
            .arg(&config)
            .arg(&target)
            .env("MODE", "merge")
            .current_dir(&repo)
            .output()
            .unwrap();

        assert!(
            !output.status.success(),
            "{unsafe_path} unexpectedly passed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("unsafe project.config.toml"), "{stderr}");
    }
}

fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

#[test]
fn security_status_uses_plugin_manifest_not_target_wrapper() {
    let root = temp_root("security-status-wrapper");
    write_executable(
        &root.join(".codex/scripts/task-registry"),
        "#!/bin/sh\ntouch status-wrapper-ran\nexit 1\n",
    );

    let _ = Command::new("bash")
        .arg(repo_root().join("scripts/status.sh"))
        .arg("--env")
        .arg("codex")
        .current_dir(&root)
        .output()
        .unwrap();

    assert!(!root.join("status-wrapper-ran").exists());
}

#[test]
fn security_workflows_pin_actions_and_permissions() {
    let repo = repo_root();
    for path in [
        ".github/workflows/ci.yml",
        ".github/workflows/agent-governance.yml",
        "templates/.github/workflows/agent-governance.yml.template",
    ] {
        let body = fs::read_to_string(repo.join(path)).unwrap();
        assert_workflow_hardened(path, &body);
    }

    let mutable = "permissions:\n  contents: read\nsteps:\n  - uses: actions/checkout@v6.0.2\n";
    assert!(workflow_hardening_error(mutable).is_some());
    let permissive =
        "steps:\n  - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd\n";
    assert!(workflow_hardening_error(permissive).is_some());
}

#[test]
fn security_nix_auto_update_uses_safe_flake_input() {
    let body =
        fs::read_to_string(repo_root().join("modules/nixos/agent-governance-auto-update.nix"))
            .unwrap();
    assert!(body.contains("builtins.match \"[A-Za-z0-9._-]+\" flakeInput"));
    assert!(body.contains("flake_input=${lib.escapeShellArg flakeInput}"));
    assert!(body.contains("--arg input \"$flake_input\""));
    assert!(body.contains("'.nodes[$input].locked.rev // empty'"));
    assert!(!body.contains(".nodes.\\\"${flakeInput}\\\""));
    assert!(!body.contains("--update-input \"${flakeInput}\""));
}

fn assert_workflow_hardened(path: &str, body: &str) {
    if let Some(error) = workflow_hardening_error(body) {
        panic!("{path}: {error}");
    }
}

fn workflow_hardening_error(body: &str) -> Option<String> {
    if !body.contains("permissions:\n  contents: read\n") {
        return Some("missing read-only contents permission".to_string());
    }
    for line in body.lines() {
        let Some((_, reference)) = line.trim().split_once("uses: ") else {
            continue;
        };
        let Some((_, version)) = reference.split_once('@') else {
            return Some(format!("action ref missing @: {reference}"));
        };
        let sha = version.split('#').next().unwrap_or_default().trim();
        if sha.len() != 40 || !sha.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Some(format!("action ref is not pinned by SHA: {reference}"));
        }
    }
    None
}
