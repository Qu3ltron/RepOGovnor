use super::*;

#[test]
fn metrics_counts_local_receipts() {
    let root = temp_root("metrics");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Validate,
            EventOutcome::Ok,
            1,
            "test".to_string(),
        ),
    )
    .unwrap();

    let report = metrics(&root).unwrap();

    assert_eq!(report.events, 1);
    assert_eq!(report.chained_events, 1);
    assert_eq!(report.receipt_chain_breaks, 0);
}

#[test]
fn metrics_validates_chained_receipts() {
    let root = temp_root("metrics-chain");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Activate,
            EventOutcome::Ok,
            1,
            "first".to_string(),
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
            "second".to_string(),
        ),
    )
    .unwrap();

    let report = metrics(&root).unwrap();

    assert_eq!(report.events, 2);
    assert_eq!(report.chained_events, 2);
    assert_eq!(report.receipt_chain_breaks, 0);
    assert_eq!(report.failed_events, 0);
}

#[test]
fn metrics_rejects_tampered_receipt_chain() {
    let root = temp_root("metrics-chain-tamper");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord::new(
            timestamp(),
            CliCommand::Activate,
            EventOutcome::Ok,
            1,
            "first".to_string(),
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
            "second".to_string(),
        ),
    )
    .unwrap();
    let path = root.join(EVENTS_PATH);
    let body = fs::read_to_string(&path)
        .unwrap()
        .replace("first", "edited");
    fs::write(&path, body).unwrap();

    let report = metrics(&root).unwrap();

    assert_eq!(report.events, 2);
    assert_eq!(report.receipt_chain_breaks, 2);
    assert!(report.failed_events >= 2);
}

#[test]
fn metrics_counts_malformed_receipts_as_failures() {
    let root = temp_root("metrics-malformed");
    seed_repo(&root);
    fs::create_dir_all(root.join("docs/task-registry")).unwrap();
    fs::write(root.join(EVENTS_PATH), "{not json}\n").unwrap();

    let report = metrics(&root).unwrap();

    assert_eq!(report.events, 1);
    assert_eq!(report.failed_events, 1);
    assert_eq!(report.malformed_events, 1);
}

#[test]
fn metrics_rejects_schema_v1_receipts() {
    let root = temp_root("metrics-v1-receipt");
    seed_repo(&root);
    fs::create_dir_all(root.join("docs/task-registry")).unwrap();
    fs::write(
        root.join(EVENTS_PATH),
        r#"{"schema_version":1,"timestamp":"2026-05-30T00:00:00Z","command":"validate","outcome":"ok","duration_ms":1,"detail":"legacy"}"#,
    )
    .unwrap();

    let report = metrics(&root).unwrap();

    assert_eq!(report.events, 1);
    assert_eq!(report.failed_events, 1);
    assert_eq!(report.malformed_events, 1);
}

#[test]
fn metrics_reports_legacy_unchained_v2_receipts() {
    let root = temp_root("metrics-v2-unchained");
    seed_repo(&root);
    fs::create_dir_all(root.join("docs/task-registry")).unwrap();
    fs::write(
        root.join(EVENTS_PATH),
        serde_json::to_string(&EventRecord::new(
            timestamp(),
            CliCommand::Validate,
            EventOutcome::Ok,
            1,
            "legacy".to_string(),
        ))
        .unwrap(),
    )
    .unwrap();

    let report = metrics(&root).unwrap();

    assert_eq!(report.events, 1);
    assert_eq!(report.unchained_events, 1);
    assert_eq!(report.receipt_chain_breaks, 0);
}
