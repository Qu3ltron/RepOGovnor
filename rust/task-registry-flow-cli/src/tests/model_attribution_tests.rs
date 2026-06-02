use super::*;
use crate::schema::{
    CostAmount, CostAttributionKind, CostAttributionTarget, CostEvidence, CostEvidenceStatus,
    CostPricingRates, CostPricingSnapshot, EventOutcome, ModelIdentityStatus,
    MutationAttributionDecision, ReportSurface, TokenUsage, UsageContribution,
};
use sha2::{Digest, Sha256};

#[path = "cost_capture_review_tests.rs"]
mod cost_capture_review_tests;

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
            reasoning_token_policy: None,
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
            tool_use_ids: Vec::new(),
        }],
        measurement_timestamp: Some("2026-06-02T00:00:00Z".to_string()),
        estimation_method: None,
        unmeasured_reason: None,
        boundary_session_id: None,
        boundary_turn_id: None,
        boundary_tool_use_id: None,
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
        boundary_session_id: None,
        boundary_turn_id: None,
        boundary_tool_use_id: None,
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
                transcript_path: None,
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
                boundary_session_id: None,
                boundary_turn_id: None,
                boundary_tool_use_id: None,
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
                boundary_session_id: None,
                boundary_turn_id: None,
                boundary_tool_use_id: None,
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
fn cost_report_totals_only_replay_valid_receipts() {
    let root = active_root("cost-report-valid-replay-only");
    append_event(
        &root,
        cost_event("measured commit cost", measured_cost_evidence(&root)),
    )
    .unwrap();

    let report = crate::cost_report::report(&root).unwrap();

    assert_eq!(report.invalid_events, 0);
    assert_eq!(report.measured_targets, 1);
    assert_eq!(report.entries[0].measured_amount_micros, Some(121250));
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
reasoning_token_policy = "reasoning_tokens_not_billed_separately"

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
fn cost_ingest_help_exposes_canonical_interface() {
    let output = crate::cost_ingest::run_command(Path::new("."), &["--help".to_string()]).unwrap();

    assert!(output.contains("cost-ingest codex-transcript"), "{output}");
    for flag in [
        "--transcript-path",
        "--session-id",
        "--since-line",
        "--until-line",
        "--pricing-snapshot",
        "--target-kind",
        "--target-id",
    ] {
        assert!(output.contains(flag), "missing {flag} in {output}");
    }
    assert!(output.contains("verifier-run"), "{output}");
    assert!(output.contains("release-cycle"), "{output}");
}

#[test]
fn cost_ingest_codex_transcript_help_excludes_legacy_selectors() {
    let output = crate::cost_ingest::run_command(
        Path::new("."),
        &["codex-transcript".to_string(), "--help".to_string()],
    )
    .unwrap();

    assert!(output.contains("--append-receipt"), "{output}");
    assert!(!output.contains("--latest"), "{output}");
    assert!(!output.contains("--commit"), "{output}");
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

#[test]
fn cost_record_unmeasured_appends_reasoned_receipt() {
    let root = active_root("cost-record-unmeasured");

    crate::cost_record::run_command(
        &root,
        &[
            "unmeasured".to_string(),
            "--target-kind".to_string(),
            "plan".to_string(),
            "--target-id".to_string(),
            "PLAN-2026-05-30-sample".to_string(),
            "--reason".to_string(),
            "private transcript cannot be published".to_string(),
            "--boundary-session-id".to_string(),
            "session-model-attribution-tests".to_string(),
            "--boundary-turn-id".to_string(),
            "turn-model-attribution-tests".to_string(),
            "--boundary-tool-use-id".to_string(),
            "tool-use-model-attribution-tests".to_string(),
        ],
    )
    .unwrap();

    let event = receipt_events(&root)
        .into_iter()
        .find(|event| {
            event
                .cost_evidence
                .as_ref()
                .is_some_and(|cost| cost.status == CostEvidenceStatus::Unmeasured)
        })
        .unwrap();
    let cost = event.cost_evidence.unwrap();
    assert_eq!(
        cost.boundary_turn_id.as_deref(),
        Some("turn-model-attribution-tests")
    );
    assert!(cost.amount.is_none());
}

#[test]
fn cost_record_unmeasured_rejects_amount_and_missing_reason() {
    let root = active_root("cost-record-unmeasured-negative");
    let missing_reason = crate::cost_record::run_command(
        &root,
        &[
            "unmeasured".to_string(),
            "--target-kind".to_string(),
            "plan".to_string(),
            "--target-id".to_string(),
            "PLAN-2026-05-30-sample".to_string(),
        ],
    )
    .unwrap_err();
    assert!(missing_reason.summary().contains("missing --reason"));

    let amount = crate::cost_record::run_command(
        &root,
        &[
            "unmeasured".to_string(),
            "--target-kind".to_string(),
            "plan".to_string(),
            "--target-id".to_string(),
            "PLAN-2026-05-30-sample".to_string(),
            "--reason".to_string(),
            "unavailable".to_string(),
            "--amount".to_string(),
            "1".to_string(),
        ],
    )
    .unwrap_err();
    assert!(amount.summary().contains("must not include --amount"));
}

#[test]
fn cost_targets_resolve_governed_objects() {
    let root = active_root("cost-targets-positive");

    let plan = crate::cost_targets::resolve(
        &root,
        Some(CostAttributionKind::Plan),
        Some("PLAN-2026-05-30-sample"),
    )
    .unwrap();
    let task = crate::cost_targets::resolve(
        &root,
        Some(CostAttributionKind::Task),
        Some("TASK-2026-05-30-sample-001"),
    )
    .unwrap();
    let verifier = crate::cost_targets::resolve(
        &root,
        Some(CostAttributionKind::VerifierRun),
        Some("B-001-sample"),
    )
    .unwrap();

    assert_eq!(plan.id, "PLAN-2026-05-30-sample");
    assert_eq!(task.id, "TASK-2026-05-30-sample-001");
    assert_eq!(verifier.id, "B-001-sample");
}

#[test]
fn cost_targets_reject_unknown_governed_objects() {
    let root = active_root("cost-targets-negative");

    for (kind, id) in [
        (CostAttributionKind::Plan, "PLAN-missing"),
        (CostAttributionKind::Task, "TASK-missing"),
        (CostAttributionKind::VerifierRun, "B-missing"),
    ] {
        let error = crate::cost_targets::resolve(&root, Some(kind), Some(id)).unwrap_err();
        assert!(error.contains("unknown governed cost target"), "{error}");
    }
}

#[test]
fn cost_capture_boundary_ingest_records_measured_evidence() {
    let root = temp_root("cost-boundary-records");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-boundary","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let output = crate::cost_ingest::run_command(
        &root,
        &[
            "codex-transcript".to_string(),
            "--transcript-path".to_string(),
            transcript.display().to_string(),
            "--session-id".to_string(),
            "session-cost-tests".to_string(),
            "--since-line".to_string(),
            "2".to_string(),
            "--until-line".to_string(),
            "3".to_string(),
            "--pricing-snapshot".to_string(),
            pricing,
            "--target-kind".to_string(),
            "commit".to_string(),
            "--target-id".to_string(),
            commit,
            "--boundary-turn-id".to_string(),
            "turn-boundary".to_string(),
            "--boundary-tool-use-id".to_string(),
            "tool-boundary".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
    )
    .unwrap();

    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        value["measured"][0]["contribution"]["tool_use_ids"][0],
        "tool-boundary"
    );
}

#[test]
fn cost_capture_rejects_boundary_mismatch() {
    let root = temp_root("cost-boundary-mismatch");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-present","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":10,"reasoning_output_tokens":0}}}}
"#,
    );

    let error = crate::cost_ingest::run_command(
        &root,
        &[
            "codex-transcript".to_string(),
            "--transcript-path".to_string(),
            transcript.display().to_string(),
            "--session-id".to_string(),
            "session-cost-tests".to_string(),
            "--since-line".to_string(),
            "2".to_string(),
            "--until-line".to_string(),
            "3".to_string(),
            "--pricing-snapshot".to_string(),
            pricing,
            "--target-kind".to_string(),
            "commit".to_string(),
            "--target-id".to_string(),
            commit,
            "--boundary-turn-id".to_string(),
            "turn-missing".to_string(),
        ],
    )
    .unwrap_err();

    assert!(error.summary().contains("boundary turn id turn-missing"));
}

#[test]
fn cost_pricing_calculates_standard_and_fast_tiers() {
    let root = temp_root("cost-pricing-tiers");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_tiered_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":0,"output_tokens":0,"reasoning_output_tokens":0}}}}
"#,
    );

    let standard = cost_ingest_amount_for_tier(&root, &transcript, &pricing, &commit, "standard");
    let fast = cost_ingest_amount_for_tier(&root, &transcript, &pricing, &commit, "fast");

    assert_eq!(standard, 1000);
    assert_eq!(fast, 2000);
}

#[test]
fn cost_pricing_rejects_unknown_tier_and_ambiguous_reasoning() {
    let root = temp_root("cost-pricing-negative");
    seed_repo(&root);
    let commit = init_cost_git_commit(&root);
    let pricing = write_tiered_pricing_snapshot(&root, "gpt-5.5");
    let transcript = write_cost_transcript(
        &root,
        r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}
{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":1000,"cached_input_tokens":0,"output_tokens":0,"reasoning_output_tokens":10}}}}
"#,
    );

    let unknown = crate::cost_ingest::run_command(
        &root,
        &cost_ingest_args(&transcript, &pricing, &commit, "unknown"),
    )
    .unwrap_err();
    assert!(unknown.summary().contains("missing service tier unknown"));

    let no_policy = crate::cost_ingest::run_command(
        &root,
        &cost_ingest_args(&transcript, &pricing, &commit, "standard"),
    )
    .unwrap_err();
    assert!(no_policy.summary().contains("reasoning_token_policy"));
}

#[test]
fn cost_coverage_accepts_measured_or_unmeasured_mutation_cost() {
    let root = active_root("cost-coverage-positive");
    verify_mutation_payload_for_format(
        &root,
        HookFormat::Codex,
        &codex_payload("printf x >src/lib.rs", "PreToolUse"),
    )
    .unwrap();
    let mut evidence = unmeasured_cost_evidence("gpt-5-codex");
    evidence.boundary_session_id = Some("session-model-attribution-tests".to_string());
    evidence.boundary_turn_id = Some("turn-model-attribution-tests".to_string());
    evidence.boundary_tool_use_id = Some("tool-use-model-attribution-tests".to_string());
    append_event(&root, cost_event("covered mutation cost", evidence)).unwrap();

    let report = crate::cost_coverage::check(&root).unwrap();

    assert_eq!(report.summary.fail, 0);
    assert!(report.summary.pass >= 1);
}

#[test]
fn cost_coverage_rejects_uncovered_or_mismatched_mutation() {
    let root = active_root("cost-coverage-negative");
    verify_mutation_payload_for_format(
        &root,
        HookFormat::Codex,
        &codex_payload("printf x >src/lib.rs", "PreToolUse"),
    )
    .unwrap();

    let report = crate::cost_coverage::check(&root).unwrap();

    assert_eq!(report.summary.fail, 1);
}

#[test]
fn cost_report_excludes_tampered_measured_receipts() {
    let root = active_root("cost-report-tampered");
    let mut evidence = measured_cost_evidence(&root);
    evidence.amount.as_mut().unwrap().amount_micros += 1;
    append_event(&root, cost_event("tampered measured commit cost", evidence)).unwrap();

    let report = crate::cost_report::report(&root).unwrap();

    assert_eq!(report.invalid_events, 1);
    assert_eq!(report.measured_targets, 0);
    assert!(report.entries.is_empty());
}

fn write_tiered_pricing_snapshot(root: &Path, model_slug: &str) -> String {
    let path = "tiered-pricing.toml";
    fs::write(
        root.join(path),
        format!(
            r#"
schema_version = 1
provider = "openai"
surface = "codex"
currency = "CREDITS"
source_url = "https://example.test/rate-card"
retrieved_at = "2026-06-02T00:00:00Z"
effective_from = "2026-06-02"
version = "2026-06-02"

[[service_tiers]]
name = "standard"
[[service_tiers.models]]
model_slug = "{model_slug}"
input_micros_per_million = 1000000
cached_input_micros_per_million = 1000000
output_micros_per_million = 1000000

[[service_tiers]]
name = "fast"
[[service_tiers.models]]
model_slug = "{model_slug}"
input_micros_per_million = 2000000
cached_input_micros_per_million = 2000000
output_micros_per_million = 2000000
"#
        ),
    )
    .unwrap();
    path.to_string()
}

fn cost_ingest_args(
    transcript: &Path,
    pricing: &str,
    commit: &str,
    service_tier: &str,
) -> Vec<String> {
    vec![
        "codex-transcript".to_string(),
        "--transcript-path".to_string(),
        transcript.display().to_string(),
        "--session-id".to_string(),
        "session-cost-tests".to_string(),
        "--since-line".to_string(),
        "2".to_string(),
        "--until-line".to_string(),
        "3".to_string(),
        "--pricing-snapshot".to_string(),
        pricing.to_string(),
        "--service-tier".to_string(),
        service_tier.to_string(),
        "--target-kind".to_string(),
        "commit".to_string(),
        "--target-id".to_string(),
        commit.to_string(),
        "--format".to_string(),
        "json".to_string(),
    ]
}

fn cost_ingest_amount_for_tier(
    root: &Path,
    transcript: &Path,
    pricing: &str,
    commit: &str,
    service_tier: &str,
) -> u64 {
    let output = crate::cost_ingest::run_command(
        root,
        &cost_ingest_args(transcript, pricing, commit, service_tier),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    value["measured"][0]["amount"]["amount_micros"]
        .as_u64()
        .unwrap()
}
