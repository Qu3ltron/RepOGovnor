use super::*;
use crate::schema::{
    EventOutcome, ModelIdentityStatus, MutationAttributionDecision, ReportSurface,
};

fn codex_payload(command: &str, hook_event_name: &str) -> String {
    serde_json::json!({
        "hook_event_name": hook_event_name,
        "model": "gpt-5-codex",
        "session_id": "session-model-attribution-tests",
        "turn_id": "turn-model-attribution-tests",
        "tool_name": "Bash",
        "tool_use_id": "tool-use-model-attribution-tests",
        "tool_input": {
            "command": command
        }
    })
    .to_string()
}

fn codex_payload_without_model(command: &str) -> String {
    serde_json::json!({
        "hook_event_name": "PreToolUse",
        "session_id": "session-model-attribution-tests",
        "turn_id": "turn-model-attribution-tests",
        "tool_name": "Bash",
        "tool_use_id": "tool-use-model-attribution-tests",
        "tool_input": {
            "command": command
        }
    })
    .to_string()
}

fn active_root(name: &str) -> std::path::PathBuf {
    let root = temp_root(name);
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    root
}

fn receipt_events(root: &Path) -> Vec<EventRecord> {
    let body = fs::read_to_string(root.join(EVENTS_PATH)).unwrap();
    body.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<EventRecord>(line).unwrap())
        .collect()
}

#[test]
fn model_attribution_codex_write_records_measured_receipt() {
    let root = active_root("model-attribution-codex-measured");
    let payload = codex_payload("printf x >src/lib.rs", "PreToolUse");

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload).unwrap();

    let events = receipt_events(&root);
    let event = events
        .iter()
        .find(|event| event.mutation_attribution.is_some())
        .expect("mutation attribution receipt");
    assert_eq!(event.outcome, EventOutcome::Ok);
    let agent = event.agent_model_attribution.as_ref().unwrap();
    assert_eq!(agent.identity_status, ModelIdentityStatus::Measured);
    assert_eq!(agent.model_slug.as_deref(), Some("gpt-5-codex"));
    assert_eq!(
        agent.tool_use_id.as_deref(),
        Some("tool-use-model-attribution-tests")
    );
    let mutation = event.mutation_attribution.as_ref().unwrap();
    assert_eq!(mutation.decision, MutationAttributionDecision::Allowed);
    assert_eq!(mutation.target_paths, vec!["src/lib.rs"]);
}

#[test]
fn model_attribution_codex_write_missing_model_fails_closed() {
    let root = active_root("model-attribution-codex-missing-model");
    let payload = codex_payload_without_model("printf x >src/lib.rs");

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload)
        .expect_err("missing Codex model identity must fail");

    assert!(
        error.contains("Codex mutation model identity missing"),
        "{error}"
    );
    let events = receipt_events(&root);
    let event = events
        .iter()
        .find(|event| event.mutation_attribution.is_some())
        .expect("denied mutation attribution receipt");
    assert_eq!(event.outcome, EventOutcome::MutationDenied);
    assert_eq!(
        event
            .agent_model_attribution
            .as_ref()
            .unwrap()
            .identity_status,
        ModelIdentityStatus::Unmeasured
    );
}

#[test]
fn model_attribution_codex_post_tool_records_observed_receipt() {
    let root = active_root("model-attribution-codex-posttooluse");
    let payload = codex_payload("printf x >src/lib.rs", "PostToolUse");

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload).unwrap();

    let events = receipt_events(&root);
    let event = events
        .iter()
        .find(|event| event.mutation_attribution.is_some())
        .expect("observed mutation attribution receipt");
    assert_eq!(
        event.mutation_attribution.as_ref().unwrap().decision,
        MutationAttributionDecision::Observed
    );
    assert_eq!(
        event
            .agent_model_attribution
            .as_ref()
            .unwrap()
            .identity_status,
        ModelIdentityStatus::Measured
    );
}

#[test]
fn model_attribution_check_reports_measured_and_unmeasured_posture() {
    let root = active_root("model-attribution-check");
    verify_mutation_payload_for_format(
        &root,
        HookFormat::Codex,
        &codex_payload("printf x >src/lib.rs", "PreToolUse"),
    )
    .unwrap();
    verify_mutation_payload_for_format(
        &root,
        HookFormat::Codex,
        &codex_payload_without_model("printf x >src/lib.rs"),
    )
    .unwrap_err();

    let report = crate::model_attribution::check(&root).unwrap();
    assert_eq!(report.surface, ReportSurface::ModelAttribution);
    assert!(report.summary.pass >= 1);
    assert!(report.summary.warn >= 1);
    assert_eq!(report.summary.fail, 0);

    let metrics_report = metrics(&root).unwrap();
    assert_eq!(metrics_report.model_attributed_mutation_events, 1);
    assert_eq!(metrics_report.model_unmeasured_mutation_events, 1);
    assert!(format_metrics(&metrics_report).contains("model_attributed_mutation_events=1"));
}

#[test]
fn model_attribution_check_rejects_false_measured_codex_receipt() {
    let root = active_root("model-attribution-bad-measured");
    append_event(
        &root,
        EventRecord::mutation_attribution(
            timestamp(),
            0,
            "bad measured attribution".to_string(),
            EventOutcome::Ok,
            "src/lib.rs".to_string(),
            crate::schema::AgentModelAttribution {
                provider: "codex".to_string(),
                adapter: "codex".to_string(),
                identity_status: ModelIdentityStatus::Measured,
                evidence_source: "hook-payload".to_string(),
                model_slug: None,
                session_id: Some("session".to_string()),
                turn_id: Some("turn".to_string()),
                tool_use_id: Some("tool".to_string()),
                hook_event_name: Some("PreToolUse".to_string()),
            },
            crate::schema::MutationAttribution {
                decision: MutationAttributionDecision::Allowed,
                hook_format: HookFormat::Codex,
                target_paths: vec!["src/lib.rs".to_string()],
            },
        ),
    )
    .unwrap();

    let report = crate::model_attribution::check(&root).unwrap();
    assert_eq!(report.summary.fail, 1);
}
