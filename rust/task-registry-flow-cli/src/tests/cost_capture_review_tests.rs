use super::*;
use std::fs;
use std::path::Path;

#[test]
fn cost_ingest_appends_distinct_receipts_per_service_tier() {
    let root = active_root("cost-review-tier-distinct");
    let commit = init_cost_git_commit(&root);
    let pricing = write_tiered_pricing_snapshot(&root, "gpt-5.5");
    let transcript = review_transcript(&root, 0);

    let standard = run_cost_ingest_json(
        &root,
        &append_args(&transcript, &pricing, &commit, "standard"),
    );
    let fast = run_cost_ingest_json(&root, &append_args(&transcript, &pricing, &commit, "fast"));
    let report = crate::cost_report::report(&root).unwrap();

    assert_eq!(standard["receipts_appended"].as_u64(), Some(1));
    assert_eq!(fast["receipts_appended"].as_u64(), Some(1));
    assert_eq!(report.invalid_events, 0);
    assert_eq!(report.measured_targets, 2);
    assert!(report.entries.iter().any(|entry| {
        entry.service_tier.as_deref() == Some("standard")
            && entry.measured_amount_micros == Some(1000)
    }));
    assert!(report.entries.iter().any(|entry| {
        entry.service_tier.as_deref() == Some("fast") && entry.measured_amount_micros == Some(2000)
    }));
}

#[test]
fn cost_ingest_dedupes_only_identical_pricing_identity() {
    let root = active_root("cost-review-tier-dedupe");
    let commit = init_cost_git_commit(&root);
    let pricing = write_tiered_pricing_snapshot(&root, "gpt-5.5");
    let transcript = review_transcript(&root, 0);

    let first = run_cost_ingest_json(
        &root,
        &append_args(&transcript, &pricing, &commit, "standard"),
    );
    let duplicate = run_cost_ingest_json(
        &root,
        &append_args(&transcript, &pricing, &commit, "standard"),
    );
    let different_tier =
        run_cost_ingest_json(&root, &append_args(&transcript, &pricing, &commit, "fast"));

    assert_eq!(first["receipts_appended"].as_u64(), Some(1));
    assert_eq!(duplicate["receipts_appended"].as_u64(), Some(0));
    assert_eq!(different_tier["receipts_appended"].as_u64(), Some(1));
}

#[test]
fn cost_pricing_prefers_named_default_tier_models() {
    let root = active_root("cost-review-default-tier");
    let commit = init_cost_git_commit(&root);
    let pricing = write_default_tier_with_stale_top_level(&root, "gpt-5.5");
    let transcript = review_transcript(&root, 0);
    let args = append_args_without_service_tier(&transcript, &pricing, &commit);

    let output = run_cost_ingest_json(&root, &args);

    assert_eq!(output["service_tier"].as_str(), Some("standard"));
    assert_eq!(
        output["measured"][0]["amount"]["amount_micros"].as_u64(),
        Some(1000)
    );
}

#[test]
fn cost_pricing_rejects_unsupported_reasoning_policy() {
    let root = active_root("cost-review-reasoning-policy");
    let commit = init_cost_git_commit(&root);
    let supported = write_reasoning_policy_snapshot(
        &root,
        "supported-pricing.toml",
        "reasoning_tokens_not_billed_separately",
    );
    let unsupported = write_reasoning_policy_snapshot(
        &root,
        "unsupported-pricing.toml",
        "reasoning_tokens_priced_as_output_tokens",
    );
    let transcript = review_transcript(&root, 10);

    let accepted = run_cost_ingest_json(
        &root,
        &cost_ingest_args(&transcript, &supported, &commit, "standard"),
    );
    let error = crate::cost_ingest::run_command(
        &root,
        &cost_ingest_args(&transcript, &unsupported, &commit, "standard"),
    )
    .unwrap_err();

    assert_eq!(
        accepted["measured"][0]["usage"]["reasoning_tokens"].as_u64(),
        Some(10)
    );
    assert_eq!(
        accepted["measured"][0]["amount"]["amount_micros"].as_u64(),
        Some(1000)
    );
    assert!(
        error
            .summary()
            .contains("unsupported reasoning_token_policy")
    );
}

#[test]
fn cost_evidence_rejects_unsupported_reasoning_policy_on_replay() {
    let root = active_root("cost-review-reasoning-replay");
    let mut evidence = measured_cost_evidence(&root);
    evidence.usage.as_mut().unwrap().reasoning_tokens = Some(10);
    evidence.pricing.as_mut().unwrap().reasoning_token_policy =
        Some("reasoning_tokens_priced_as_output_tokens".to_string());
    append_event(&root, cost_event("unsupported reasoning policy", evidence)).unwrap();

    let evidence_report = crate::cost_evidence::check(&root).unwrap();
    let cost_report = crate::cost_report::report(&root).unwrap();

    assert_eq!(evidence_report.summary.fail, 1);
    assert!(
        evidence_report
            .checks
            .iter()
            .any(|check| check.actual.contains("unsupported reasoning_token_policy"))
    );
    assert_eq!(cost_report.invalid_events, 1);
    assert!(cost_report.entries.is_empty());
}

#[test]
fn cost_coverage_rejects_tool_bound_mutation_without_tool_bound_cost() {
    let root = active_root("cost-review-tool-negative");
    verify_mutation_payload_for_format(
        &root,
        HookFormat::Codex,
        &codex_payload("printf x >src/lib.rs", "PreToolUse"),
    )
    .unwrap();
    let mut evidence = measured_cost_evidence(&root);
    evidence.model_slug = Some("gpt-5-codex".to_string());
    let contribution = evidence.usage_contributions.first_mut().unwrap();
    contribution.session_id = "session-model-attribution-tests".to_string();
    contribution.turn_ids = vec!["turn-model-attribution-tests".to_string()];
    contribution.tool_use_ids = Vec::new();
    append_event(&root, cost_event("unbound measured tool cost", evidence)).unwrap();

    let report = crate::cost_coverage::check(&root).unwrap();

    assert_eq!(report.summary.fail, 1);
}

#[test]
fn cost_coverage_accepts_exact_tool_bound_cost() {
    let root = active_root("cost-review-tool-positive");
    verify_mutation_payload_for_format(
        &root,
        HookFormat::Codex,
        &codex_payload("printf x >src/lib.rs", "PreToolUse"),
    )
    .unwrap();
    let mut evidence = measured_cost_evidence(&root);
    evidence.model_slug = Some("gpt-5-codex".to_string());
    let contribution = evidence.usage_contributions.first_mut().unwrap();
    contribution.session_id = "session-model-attribution-tests".to_string();
    contribution.turn_ids = vec!["turn-model-attribution-tests".to_string()];
    contribution.tool_use_ids = vec!["tool-use-model-attribution-tests".to_string()];
    append_event(&root, cost_event("bound measured tool cost", evidence)).unwrap();

    let report = crate::cost_coverage::check(&root).unwrap();

    assert_eq!(report.summary.fail, 0);
    assert!(report.summary.pass >= 1);
}

fn review_transcript(root: &Path, reasoning_tokens: u64) -> std::path::PathBuf {
    let token_count = serde_json::json!({
        "type": "event_msg",
        "payload": {
            "type": "token_count",
            "info": {
                "last_token_usage": {
                    "input_tokens": 1000,
                    "cached_input_tokens": 0,
                    "output_tokens": 0,
                    "reasoning_output_tokens": reasoning_tokens,
                }
            }
        }
    })
    .to_string();
    write_cost_transcript(
        root,
        &format!(
            "{}\n{}\n{}\n",
            r#"{"type":"session_meta","payload":{"id":"session-cost-tests","model":"gpt-5.5"}}"#,
            r#"{"type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.5"}}"#,
            token_count
        ),
    )
}

fn append_args(transcript: &Path, pricing: &str, commit: &str, service_tier: &str) -> Vec<String> {
    let mut args = cost_ingest_args(transcript, pricing, commit, service_tier);
    args.push("--append-receipt".to_string());
    args
}

fn append_args_without_service_tier(transcript: &Path, pricing: &str, commit: &str) -> Vec<String> {
    let mut args = vec![
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
        "--target-kind".to_string(),
        "commit".to_string(),
        "--target-id".to_string(),
        commit.to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    args.push("--append-receipt".to_string());
    args
}

fn run_cost_ingest_json(root: &Path, args: &[String]) -> serde_json::Value {
    let output = crate::cost_ingest::run_command(root, args).unwrap();
    serde_json::from_str(&output).unwrap()
}

fn write_default_tier_with_stale_top_level(root: &Path, model_slug: &str) -> String {
    let path = "default-tier-pricing.toml";
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
service_tier = "standard"

[[models]]
model_slug = "{model_slug}"
input_micros_per_million = 9000000
cached_input_micros_per_million = 9000000
output_micros_per_million = 9000000

[[service_tiers]]
name = "standard"
[[service_tiers.models]]
model_slug = "{model_slug}"
input_micros_per_million = 1000000
cached_input_micros_per_million = 1000000
output_micros_per_million = 1000000
"#
        ),
    )
    .unwrap();
    path.to_string()
}

fn write_reasoning_policy_snapshot(root: &Path, path: &str, policy: &str) -> String {
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
service_tier = "standard"
reasoning_token_policy = "{policy}"

[[service_tiers]]
name = "standard"
[[service_tiers.models]]
model_slug = "gpt-5.5"
input_micros_per_million = 1000000
cached_input_micros_per_million = 1000000
output_micros_per_million = 1000000
"#
        ),
    )
    .unwrap();
    path.to_string()
}
