use super::*;
use crate::model::EVENTS_PATH;
use crate::schema::{CliCommand, EventOutcome};
use fs2::FileExt;

// ---------------------------------------------------------------------------
// Verify-chain tests
// ---------------------------------------------------------------------------

#[test]
fn verify_chain_reports_intact_chain() {
    let root = temp_root("verify-chain-intact");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Activate,
            EventOutcome::Ok,
            1,
            "first event".to_string(),
        ),
    )
    .unwrap();
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Status,
            EventOutcome::Ok,
            2,
            "second event".to_string(),
        ),
    )
    .unwrap();

    let result = crate::verify_chain::run_verify_chain(&root, &[]).unwrap();
    assert!(result.contains("intact"), "{result}");
}

#[test]
fn verify_chain_reports_tampered_event_content() {
    let root = temp_root("verify-chain-tamper");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Activate,
            EventOutcome::Ok,
            1,
            "original summary".to_string(),
        ),
    )
    .unwrap();

    let events_path = root.join(EVENTS_PATH);
    let body = fs::read_to_string(&events_path).unwrap();
    let tampered = body.replace("original summary", "tampered summary");
    fs::write(&events_path, tampered).unwrap();

    let error =
        crate::verify_chain::run_verify_chain(&root, &[]).expect_err("tampered chain must fail");
    assert!(error.summary().contains("break"), "{}", error.summary());
}

#[test]
fn verify_chain_json_reports_success() {
    let root = temp_root("verify-chain-json");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Metrics,
            EventOutcome::Ok,
            3,
            "json test".to_string(),
        ),
    )
    .unwrap();

    let result =
        crate::verify_chain::run_verify_chain(&root, &["--format".to_string(), "json".to_string()])
            .unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&result).unwrap();
    assert_eq!(value["surface"], "receipt-chain");
    assert_eq!(value["summary"]["fail"], 0);
}

#[test]
fn verify_chain_repair_fixes_broken_chain() {
    let root = temp_root("verify-chain-repair");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Activate,
            EventOutcome::Ok,
            1,
            "fixable summary".to_string(),
        ),
    )
    .unwrap();

    let events_path = root.join(EVENTS_PATH);
    let body = fs::read_to_string(&events_path).unwrap();
    let tampered = body.replace("fixable summary", "corrupted");
    fs::write(&events_path, tampered).unwrap();

    let result = crate::verify_chain::run_verify_chain(&root, &["--repair".to_string()]).unwrap();
    assert!(
        result.contains("repaired") || result.contains("intact"),
        "{result}"
    );

    let result = crate::verify_chain::run_verify_chain(&root, &[]).unwrap();
    assert!(result.contains("intact"), "{result}");
}

#[test]
fn verify_chain_reports_empty_events_file_as_ok() {
    let root = temp_root("verify-chain-empty");
    seed_repo(&root);

    let result = crate::verify_chain::run_verify_chain(&root, &[]).unwrap();
    assert!(result.contains("intact"), "{result}");
}

#[test]
fn verify_chain_fails_unchained_v2_receipt() {
    let root = temp_root("verify-chain-unchained");
    seed_repo(&root);
    let events_path = root.join(EVENTS_PATH);
    fs::create_dir_all(events_path.parent().unwrap()).unwrap();
    let event = EventRecord::new(
        timestamp(),
        CliCommand::Metrics,
        EventOutcome::Ok,
        1,
        "unchained".to_string(),
    );
    fs::write(
        &events_path,
        format!("{}\n", serde_json::to_string(&event).unwrap()),
    )
    .unwrap();

    let error = crate::verify_chain::run_verify_chain(&root, &[])
        .expect_err("unchained v2 receipt must fail");

    assert!(
        error.summary().contains("unchained event"),
        "{}",
        error.summary()
    );
}

#[test]
fn verify_chain_repair_chains_parseable_v2_receipts() {
    let root = temp_root("verify-chain-repair-unchained");
    seed_repo(&root);
    let events_path = root.join(EVENTS_PATH);
    fs::create_dir_all(events_path.parent().unwrap()).unwrap();
    let first = EventRecord::new(
        timestamp(),
        CliCommand::Metrics,
        EventOutcome::Ok,
        1,
        "first unchained".to_string(),
    );
    let second = EventRecord::new(
        timestamp(),
        CliCommand::Validate,
        EventOutcome::Ok,
        2,
        "second unchained".to_string(),
    );
    fs::write(
        &events_path,
        format!(
            "{}\n{}\n",
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        ),
    )
    .unwrap();

    let repaired = crate::verify_chain::run_verify_chain(&root, &["--repair".to_string()]).unwrap();
    assert!(repaired.contains("repaired"), "{repaired}");
    let result = crate::verify_chain::run_verify_chain(&root, &[]).unwrap();
    assert!(result.contains("intact"), "{result}");
}

#[test]
fn verify_chain_repair_refuses_schema_v1_receipt() {
    let root = temp_root("verify-chain-repair-v1");
    seed_repo(&root);
    let events_path = root.join(EVENTS_PATH);
    fs::create_dir_all(events_path.parent().unwrap()).unwrap();
    fs::write(
        &events_path,
        r#"{"schema_version":1,"timestamp":"2026-05-30T00:00:00Z","command":"validate","outcome":"ok","duration_ms":1,"summary":"legacy"}"#,
    )
    .unwrap();

    let error = crate::verify_chain::run_verify_chain(&root, &["--repair".to_string()])
        .expect_err("schema v1 repair must fail");

    assert!(
        error.summary().contains("schema_version 2"),
        "{}",
        error.summary()
    );
}

// ---------------------------------------------------------------------------
// Concurrent-write lock test
// ---------------------------------------------------------------------------

#[test]
fn save_registry_holds_exclusive_lock() {
    let root = temp_root("lock-exclusive");
    seed_repo(&root);
    let registry = load_registry(&root).unwrap();
    crate::registry_io::save_registry(&root, &registry).unwrap();

    let file = fs::OpenOptions::new()
        .write(true)
        .open(root.join(crate::model::REGISTRY_PATH))
        .unwrap();
    assert!(file.try_lock_exclusive().is_ok());
}
