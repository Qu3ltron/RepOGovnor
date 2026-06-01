use super::*;
use crate::reports::RuntimeFailure;

#[test]
fn backlog_check_accepts_current_gap_pipeline() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report = crate::backlog_check::report(&root).unwrap();

    assert_eq!(report.summary.fail, 0);
    assert!(report.checks.iter().any(|check| {
        check.check_id == "backlog-negative-nonclaim" && check.status == CheckStatus::Pass
    }));
}

#[test]
fn backlog_check_rejects_missing_fields_and_overclaims() {
    let report = crate::backlog_check::report_from_body(
        r#"# Gap Pipeline

## Current Evidence

### GP-001: Bad gap
- Claim pressure: public claim.
- Current evidence: weak.

## Negative Non-Claims

No product correctness proof.

## Drain protocol

Governance proves product correctness.
"#,
    )
    .unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "backlog-gap-field" && check.status == CheckStatus::Fail
    }));
    assert!(report.checks.iter().any(|check| {
        check.check_id == "backlog-forbidden-claim" && check.status == CheckStatus::Fail
    }));
}

#[test]
fn backlog_check_rejects_missing_negative_nonclaims() {
    let body = include_str!("../../../../docs/gap-pipeline.md").replace(
        "No remote receipt sync",
        "Remote receipt sync is not discussed",
    );

    let report = crate::backlog_check::report_from_body(&body).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "backlog-negative-nonclaim"
            && check.expected == "No remote receipt sync"
            && check.status == CheckStatus::Fail
    }));
}

#[test]
fn backlog_check_json_failure_preserves_report() {
    let root = temp_root("backlog-json-failure");
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join("docs/gap-pipeline.md"),
        "# Gap Pipeline\n\n## Remaining Gaps\n\nthere are no remaining gaps\n",
    )
    .unwrap();

    let error =
        crate::backlog_check::run_command(&root, &["--format".to_string(), "json".to_string()])
            .expect_err("invalid backlog must fail");

    let RuntimeFailure::Json { payload, .. } = error else {
        panic!("backlog-check failure should preserve JSON report");
    };
    let value = serde_json::from_str::<serde_json::Value>(&payload).unwrap();
    assert_eq!(value["surface"], "backlog");
    assert!(value["summary"]["fail"].as_u64().unwrap() > 0);
}
