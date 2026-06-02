use super::*;
use crate::schema::{
    CostAmount, CostAttributionKind, CostAttributionTarget, CostEvidence, CostEvidenceStatus,
    CostPricingRates, CostPricingSnapshot, EventOutcome, ModelIdentityStatus,
    MutationAttributionDecision, ReportSurface, TokenUsage, UsageContribution,
};
use sha2::{Digest, Sha256};

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

fn cost_event(summary: &str, cost_evidence: CostEvidence) -> EventRecord {
    let mut event = EventRecord::new(
        timestamp(),
        crate::schema::CliCommand::Metrics,
        EventOutcome::Ok,
        0,
        summary.to_string(),
    );
    event.cost_evidence = Some(cost_evidence);
    event
}

fn cost_target(kind: CostAttributionKind, id: &str) -> CostAttributionTarget {
    CostAttributionTarget {
        kind,
        id: id.to_string(),
    }
}

fn measured_cost_evidence(root: &Path) -> CostEvidence {
    let pricing_path = write_pricing_snapshot(root, "gpt-5-codex");
    let transcript_line = r#"{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}"#;
    let transcript = write_cost_transcript(
        root,
        &format!(
            "{}\n{}\n{}\n",
            r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5-codex"}}"#,
            r#"{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5-codex"}}"#,
            transcript_line
        ),
    );
    CostEvidence {
        status: CostEvidenceStatus::Measured,
        evidence_source: "provider-usage-receipt".to_string(),
        attribution_target: cost_target(CostAttributionKind::Commit, "abc1234"),
        provider: Some("openai".to_string()),
        model_slug: Some("gpt-5-codex".to_string()),
        usage: Some(TokenUsage {
            input_tokens: 1000,
            output_tokens: 10,
            cached_input_tokens: Some(100),
            reasoning_tokens: None,
        }),
        pricing: Some(CostPricingSnapshot {
            source: "pricing-snapshot".to_string(),
            version: "2026-06-02".to_string(),
            currency: "CREDITS".to_string(),
            service_tier: "codex-cloud".to_string(),
            snapshot_path: pricing_path.clone(),
            snapshot_sha256: test_file_sha256(&root.join(&pricing_path)),
        }),
        amount: Some(CostAmount {
            currency: "CREDITS".to_string(),
            amount_micros: 121250,
        }),
        pricing_rates: Some(CostPricingRates {
            input_micros_per_million: 125_000_000,
            cached_input_micros_per_million: 12_500_000,
            output_micros_per_million: 750_000_000,
        }),
        usage_contributions: vec![UsageContribution {
            source_kind: "codex-transcript-token-count".to_string(),
            source_path: transcript.display().to_string(),
            source_sha256: test_selected_digest(&[(3, transcript_line)]),
            start_line: 3,
            end_line: 3,
            event_count: 1,
            model_slug: "gpt-5-codex".to_string(),
            model_context_line: 2,
            session_id: "session-cost-tests".to_string(),
            selected_event_digest_sha256: test_selected_digest(&[(3, transcript_line)]),
            turn_ids: vec!["turn-1".to_string()],
        }],
        measurement_timestamp: Some("2026-06-02T00:00:00Z".to_string()),
        estimation_method: None,
        unmeasured_reason: None,
    }
}

fn unmeasured_cost_evidence(model_slug: &str) -> CostEvidence {
    CostEvidence {
        status: CostEvidenceStatus::Unmeasured,
        evidence_source: "provider-adapter-contract".to_string(),
        attribution_target: cost_target(CostAttributionKind::Commit, "abc1234"),
        provider: Some("unknown".to_string()),
        model_slug: Some(model_slug.to_string()),
        usage: None,
        pricing: None,
        amount: None,
        pricing_rates: None,
        usage_contributions: Vec::new(),
        measurement_timestamp: None,
        estimation_method: None,
        unmeasured_reason: Some("provider adapter or pricing source unavailable".to_string()),
    }
}

fn test_file_sha256(path: &Path) -> String {
    let body = fs::read(path).unwrap();
    format!("{:x}", Sha256::digest(&body))
}

fn test_selected_digest(lines: &[(usize, &str)]) -> String {
    let mut hasher = Sha256::new();
    for (line_number, line) in lines {
        hasher.update(line_number.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
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

#[test]
fn cost_evidence_check_reports_measured_estimated_and_unmeasured() {
    let root = active_root("cost-evidence-check");
    append_event(
        &root,
        cost_event("measured commit cost", measured_cost_evidence(&root)),
    )
    .unwrap();
    append_event(
        &root,
        cost_event(
            "estimated verifier cost",
            CostEvidence {
                status: CostEvidenceStatus::Estimated,
                evidence_source: "manual-estimate".to_string(),
                attribution_target: cost_target(CostAttributionKind::VerifierRun, "B-001"),
                provider: Some("openai".to_string()),
                model_slug: Some("gpt-5-codex".to_string()),
                usage: None,
                pricing: None,
                amount: Some(CostAmount {
                    currency: "USD".to_string(),
                    amount_micros: 10,
                }),
                pricing_rates: None,
                usage_contributions: Vec::new(),
                measurement_timestamp: None,
                estimation_method: Some("operator-entered estimate".to_string()),
                unmeasured_reason: None,
            },
        ),
    )
    .unwrap();
    append_event(
        &root,
        cost_event(
            "unmeasured session cost",
            CostEvidence {
                status: CostEvidenceStatus::Unmeasured,
                evidence_source: "local-session".to_string(),
                attribution_target: cost_target(CostAttributionKind::Session, "session-1"),
                provider: Some("codex".to_string()),
                model_slug: Some("gpt-5-codex".to_string()),
                usage: None,
                pricing: None,
                amount: None,
                pricing_rates: None,
                usage_contributions: Vec::new(),
                measurement_timestamp: None,
                estimation_method: None,
                unmeasured_reason: Some("provider usage receipt unavailable".to_string()),
            },
        ),
    )
    .unwrap();

    let report = crate::cost_evidence::check(&root).unwrap();
    assert_eq!(report.surface, ReportSurface::CostEvidence);
    assert_eq!(report.summary.fail, 0);
    assert_eq!(report.summary.pass, 1);
    assert_eq!(report.summary.warn, 2);

    let metrics_report = metrics(&root).unwrap();
    assert_eq!(metrics_report.cost_measured_events, 1);
    assert_eq!(metrics_report.cost_estimated_events, 1);
    assert_eq!(metrics_report.cost_unmeasured_events, 1);
    assert!(format_metrics(&metrics_report).contains("cost_measured_events=1"));
}

#[test]
fn cost_evidence_check_rejects_false_measured_receipt() {
    let root = active_root("cost-evidence-bad-measured");
    let mut evidence = measured_cost_evidence(&root);
    evidence.provider = None;
    evidence.usage = None;
    evidence.pricing = None;

    append_event(&root, cost_event("bad measured cost", evidence)).unwrap();

    let report = crate::cost_evidence::check(&root).unwrap();
    assert_eq!(report.summary.fail, 1);
}

#[test]
fn cost_capture_records_transcript_and_event_digest() {
    let root = temp_root("cost-capture-digest");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let token_line = r#"{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}"#;
    let transcript = write_cost_transcript(
        &root,
        &format!(
            "{}\n{}\n{}\n",
            r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}"#,
            r#"{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}"#,
            token_line
        ),
    );

    let request = crate::cost_ingest::request_for_test(transcript.clone(), pricing, commit);
    let report = crate::cost_ingest::ingest_for_test(&root, &request).unwrap();
    let contribution = &report.measured[0].contribution;

    assert_eq!(contribution.session_id, "session-cost-tests");
    assert_eq!(
        contribution.source_sha256,
        test_selected_digest(&[(3, token_line)])
    );
    assert_eq!(
        contribution.selected_event_digest_sha256,
        test_selected_digest(&[(3, token_line)])
    );
    assert_eq!(contribution.model_context_line, 2);
}

#[test]
fn cost_capture_rejects_tampered_transcript_digest() {
    let root = active_root("cost-capture-tampered-transcript");
    let evidence = measured_cost_evidence(&root);
    let source_path = evidence.usage_contributions[0].source_path.clone();
    append_event(&root, cost_event("measured commit cost", evidence)).unwrap();
    fs::write(source_path, "tampered\n").unwrap();

    let report = crate::cost_evidence::check(&root).unwrap();
    assert_eq!(report.summary.fail, 1);
}

#[test]
fn cost_capture_accepts_explicit_session_and_transcript() {
    let root = temp_root("cost-capture-explicit-session");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, commit);
    let report = crate::cost_ingest::ingest_for_test(&root, &request).unwrap();

    assert_eq!(report.session_id, "session-cost-tests");
    assert_eq!(report.measured.len(), 1);
}

#[test]
fn cost_capture_links_measured_usage_to_commit_target() {
    let root = temp_root("cost-capture-commit-target");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, "HEAD".to_string());
    let report = crate::cost_ingest::ingest_for_test(&root, &request).unwrap();

    assert_eq!(report.target.kind, CostAttributionKind::Commit);
    assert_eq!(report.target.id, commit);
}

#[test]
fn pricing_snapshot_accepts_valid_codex_rate_card() {
    let root = temp_root("pricing-valid-codex");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, commit);
    let report = crate::cost_ingest::ingest_for_test(&root, &request).unwrap();

    assert_eq!(report.measured[0].amount.currency, "CREDITS");
}

#[test]
fn pricing_snapshot_rejects_zero_rates() {
    let root = temp_root("pricing-zero-rate");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let pricing_path = root.join(&pricing);
    let body = fs::read_to_string(&pricing_path).unwrap();
    fs::write(
        &pricing_path,
        body.replace(
            "input_micros_per_million = 125000000",
            "input_micros_per_million = 0",
        ),
    )
    .unwrap();
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, commit);
    let error = crate::cost_ingest::ingest_for_test(&root, &request).unwrap_err();

    assert!(error.contains("zero rate"), "{error}");
}

#[test]
fn cost_evidence_accepts_non_overlapping_ranges() {
    let root = active_root("cost-evidence-non-overlap");
    append_event(
        &root,
        cost_event("measured commit cost", measured_cost_evidence(&root)),
    )
    .unwrap();

    let report = crate::cost_evidence::check(&root).unwrap();
    assert_eq!(report.summary.fail, 0);
}

#[test]
fn cost_evidence_rejects_duplicate_selected_event_digest() {
    let root = active_root("cost-evidence-duplicate-digest");
    let evidence = measured_cost_evidence(&root);
    append_event(&root, cost_event("measured commit cost", evidence.clone())).unwrap();
    append_event(
        &root,
        cost_event("duplicate measured commit cost", evidence),
    )
    .unwrap();

    let report = crate::cost_evidence::check(&root).unwrap();
    assert_eq!(report.summary.fail, 1);
}

#[test]
fn cost_migration_preserves_receipt_chain_integrity() {
    let root = active_root("cost-migration-chain");
    append_event(
        &root,
        cost_event("measured commit cost", measured_cost_evidence(&root)),
    )
    .unwrap();

    crate::verify_chain::run_verify_chain(&root, &["--format".to_string(), "json".to_string()])
        .unwrap();
}

#[test]
fn cost_evidence_rejects_legacy_measured_receipt_after_migration() {
    let root = active_root("cost-evidence-legacy-reject");
    let mut evidence = measured_cost_evidence(&root);
    evidence.usage_contributions[0].source_sha256.clear();
    evidence.usage_contributions[0]
        .selected_event_digest_sha256
        .clear();
    append_event(&root, cost_event("legacy measured commit cost", evidence)).unwrap();

    let report = crate::cost_evidence::check(&root).unwrap();
    assert_eq!(report.summary.fail, 1);
}

#[test]
fn provider_adapter_records_unknown_model_as_unmeasured() {
    let root = active_root("provider-unmeasured");
    append_event(
        &root,
        cost_event(
            "unmeasured provider cost",
            unmeasured_cost_evidence("deepseek-v4-flash:cloud"),
        ),
    )
    .unwrap();

    let report = crate::cost_evidence::check(&root).unwrap();
    assert_eq!(report.summary.fail, 0);
    assert_eq!(report.summary.warn, 1);
}

#[test]
fn provider_adapter_rejects_model_name_pricing_inference() {
    let root = temp_root("provider-no-name-pricing");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"deepseek-v4-flash:cloud"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"deepseek-v4-flash:cloud"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, commit);
    let error = crate::cost_ingest::ingest_for_test(&root, &request).unwrap_err();

    assert!(error.contains("pricing snapshot missing model"), "{error}");
}

#[test]
fn cost_report_emits_measured_cost_per_commit() {
    let root = active_root("cost-report-measured");
    append_event(
        &root,
        cost_event("measured commit cost", measured_cost_evidence(&root)),
    )
    .unwrap();

    let report = crate::cost_report::report(&root).unwrap();
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.status == "measured")
        .unwrap();

    assert_eq!(entry.target_kind, "commit");
    assert_eq!(entry.measured_amount_micros, Some(121250));
}

#[test]
fn cost_report_does_not_emit_zero_for_unmeasured_commit() {
    let root = active_root("cost-report-unmeasured");
    append_event(
        &root,
        cost_event("unmeasured commit cost", unmeasured_cost_evidence("gemma4")),
    )
    .unwrap();

    let report = crate::cost_report::report(&root).unwrap();
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.status == "unmeasured")
        .unwrap();

    assert_eq!(entry.target_kind, "commit");
    assert_eq!(entry.measured_amount_micros, None);
}

fn write_pricing_snapshot(root: &Path, model_slug: &str) -> String {
    let path = "pricing.toml";
    fs::write(
        root.join(path),
        format!(
            r#"
schema_version = 1
provider = "openai"
surface = "codex"
currency = "CREDITS"
source_url = "https://help.openai.com/en/articles/20001106-codex-rate-card"
retrieved_at = "2026-06-02T00:00:00Z"
effective_from = "2026-06-02"
version = "2026-06-02"
service_tier = "codex-cloud"

[[models]]
model_slug = "{model_slug}"
input_micros_per_million = 125000000
cached_input_micros_per_million = 12500000
output_micros_per_million = 750000000
"#
        ),
    )
    .unwrap();
    path.to_string()
}

fn write_cost_transcript(root: &Path, body: &str) -> PathBuf {
    let path = root.join("transcript.jsonl");
    fs::write(&path, body).unwrap();
    path
}

fn init_cost_git_commit(root: &Path) -> String {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(root)
        .output()
        .unwrap();
    fs::write(root.join("commit-target.txt"), "commit target").unwrap();
    std::process::Command::new("git")
        .args(["add", "commit-target.txt"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "seed"])
        .current_dir(root)
        .output()
        .unwrap();
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn cost_ingest_computes_credit_amount_from_pricing_snapshot() {
    let root = temp_root("cost-ingest-compute");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000000,"cached_input_tokens":200000,"output_tokens":1000,"reasoning_output_tokens":100}}}}
"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, commit);
    let report = crate::cost_ingest::ingest_for_test(&root, &request).unwrap();

    assert_eq!(report.measured.len(), 1);
    assert_eq!(report.measured[0].model_slug, "gpt-5.5");
    assert_eq!(report.measured[0].usage.input_tokens, 1_000_000);
    assert_eq!(report.measured[0].usage.cached_input_tokens, Some(200_000));
    assert_eq!(report.measured[0].amount.currency, "CREDITS");
    assert_eq!(report.measured[0].amount.amount_micros, 103_250_000);
    assert_eq!(report.measured[0].contribution.start_line, 3);
    assert_eq!(report.measured[0].contribution.event_count, 1);
}

#[test]
fn cost_ingest_append_receipt_is_idempotent_for_same_contribution() {
    let root = temp_root("cost-ingest-idempotent-append");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let request = crate::cost_ingest::append_request_for_test(
        transcript.clone(),
        pricing.clone(),
        commit.clone(),
    );
    let first = crate::cost_ingest::ingest_for_test(&root, &request).unwrap();
    let second = crate::cost_ingest::ingest_for_test(&root, &request).unwrap();

    assert_eq!(first.receipts_appended, 1);
    assert_eq!(second.receipts_appended, 0);
    let cost_receipts = receipt_events(&root)
        .into_iter()
        .filter(|event| event.cost_evidence.is_some())
        .count();
    assert_eq!(cost_receipts, 1);
}

#[test]
fn cost_ingest_rejects_missing_usage() {
    let root = temp_root("cost-ingest-missing-usage");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, commit);
    let error = crate::cost_ingest::ingest_for_test(&root, &request).unwrap_err();

    assert!(
        error.contains("no Codex token_count usage events"),
        "{error}"
    );
}

#[test]
fn cost_ingest_rejects_unknown_pricing_model() {
    let root = temp_root("cost-ingest-unknown-model");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.4");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":0,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let request = crate::cost_ingest::request_for_test(transcript, pricing, commit);
    let error = crate::cost_ingest::ingest_for_test(&root, &request).unwrap_err();

    assert!(
        error.contains("pricing snapshot missing model gpt-5.5"),
        "{error}"
    );
}

#[test]
fn cost_capture_rejects_latest_selector() {
    let error = crate::cost_ingest::run_command(
        Path::new("."),
        &[
            "codex-transcript".to_string(),
            "--latest".to_string(),
            "--pricing-snapshot".to_string(),
            "pricing.toml".to_string(),
        ],
    )
    .unwrap_err();

    assert!(
        error.summary().contains("unsupported selector --latest"),
        "{error:?}"
    );
}

#[test]
fn cost_capture_rejects_legacy_commit_flag() {
    let error = crate::cost_ingest::run_command(
        Path::new("."),
        &[
            "codex-transcript".to_string(),
            "--transcript-path".to_string(),
            "transcript.jsonl".to_string(),
            "--session-id".to_string(),
            "session-cost-tests".to_string(),
            "--since-line".to_string(),
            "1".to_string(),
            "--until-line".to_string(),
            "2".to_string(),
            "--pricing-snapshot".to_string(),
            "pricing.toml".to_string(),
            "--commit".to_string(),
            "HEAD".to_string(),
        ],
    )
    .unwrap_err();

    assert!(
        error.summary().contains("unsupported legacy flag --commit"),
        "{error:?}"
    );
}
