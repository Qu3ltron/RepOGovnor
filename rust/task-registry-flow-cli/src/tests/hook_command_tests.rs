use super::*;

fn bash_payload(command: &str) -> String {
    serde_json::json!({
        "model": "gpt-5-codex",
        "session_id": "session-hook-command-tests",
        "turn_id": "turn-hook-command-tests",
        "tool_name": "Bash",
        "tool_use_id": "tool-use-hook-command-tests",
        "tool_input": {
            "command": command
        }
    })
    .to_string()
}

#[test]
fn hook_command_allows_read_only_without_path() {
    let root = temp_root("hook-command-read-only");
    let payload =
        bash_payload("cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml");

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload).unwrap();
}

#[test]
fn hook_command_allows_extracted_path_with_active_target() {
    let root = temp_root("hook-command-active-target");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let payload =
        bash_payload("python3 -c \"from pathlib import Path; Path('src/lib.rs').write_text('x')\"");

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload).unwrap();
}

#[test]
fn hook_command_allows_compact_redirection_with_active_target() {
    let root = temp_root("hook-command-compact-active-target");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let payload = bash_payload("printf x >src/lib.rs");

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload).unwrap();
}

#[test]
fn hook_command_denies_compact_redirection_to_unbound_path() {
    let root = temp_root("hook-command-compact-unbound");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let payload = bash_payload("printf x >src/other.rs");

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("compact redirection to an unbound path must fail");
    assert!(
        error.contains("src/other.rs is not bound to an active registry task target"),
        "{error}"
    );
}

#[test]
fn hook_command_denies_compact_variable_redirection_without_path() {
    let root = temp_root("hook-command-compact-variable");
    let payload = bash_payload("target=src/lib.rs; printf x >$target");

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("compact variable redirection must fail closed");
    assert!(error.contains("deterministic target path"), "{error}");
}

#[test]
fn hook_command_denies_python_inline_write_without_path() {
    let root = temp_root("hook-command-python-write");
    let payload = bash_payload("python3 -c \"target='src/lib.rs'; open(target, 'w').write('x')\"");

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("ambiguous Python write must fail closed");
    assert!(error.contains("deterministic target path"), "{error}");
}

#[test]
fn hook_command_denies_mixed_open_read_and_variable_write() {
    let root = temp_root("hook-command-mixed-open-write");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let payload = bash_payload(
        "python3 -c \"target='src/other.rs'; open('src/lib.rs').read(); open(target, 'w').write('x')\"",
    );

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("variable write target must not be masked by a deterministic read target");
    assert!(error.contains("deterministic target path"), "{error}");
}

#[test]
fn hook_command_denies_node_inline_write_without_path() {
    let root = temp_root("hook-command-node-write");
    let payload =
        bash_payload("node -e \"const p='src/lib.rs'; require('fs').writeFileSync(p, 'x')\"");

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("ambiguous Node write must fail closed");
    assert!(error.contains("deterministic target path"), "{error}");
}

#[test]
fn hook_command_denies_heredoc_write_without_path() {
    let root = temp_root("hook-command-heredoc-write");
    let payload =
        bash_payload("python3 - <<'PY'\ntarget='src/lib.rs'\nopen(target, 'w').write('x')\nPY");

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("ambiguous heredoc write must fail closed");
    assert!(error.contains("deterministic target path"), "{error}");
}

#[test]
fn hook_command_denies_nested_shell_write_without_path() {
    let root = temp_root("hook-command-nested-shell-write");
    let payload = bash_payload("bash -lc \"target=src/lib.rs; echo x > $target\"");

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("ambiguous nested shell write must fail closed");
    assert!(error.contains("deterministic target path"), "{error}");
}
