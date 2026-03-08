use std::fs;

use atlassy_cli::*;
use atlassy_contracts::{ErrorCode, RUNTIME_LIVE};

mod common;

use common::fixture_path;

#[test]
fn complete_matrix_generates_aggregate_report() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let report = execute_batch_from_manifest_file(
        &fixture_path("batch_complete_manifest.json"),
        temp.path(),
    )
    .expect("batch run should succeed");

    assert_eq!(report.total_runs, 10);
    assert!(report.kpi.is_some());
    assert_eq!(
        report
            .kpi
            .as_ref()
            .expect("kpi should be present")
            .pattern_rollups
            .len(),
        3
    );
    assert!(
        temp.path()
            .join("artifacts")
            .join("batch")
            .join("manifest.normalized.json")
            .exists()
    );
    assert!(
        temp.path()
            .join("artifacts")
            .join("batch")
            .join("artifact-index.json")
            .exists()
    );
    assert!(
        temp.path()
            .join("artifacts")
            .join("batch")
            .join("report.json")
            .exists()
    );
}

#[test]
fn unmatched_pair_manifest_is_rejected() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let err = execute_batch_from_manifest_file(
        &fixture_path("batch_unmatched_pair_manifest.json"),
        temp.path(),
    )
    .expect_err("unmatched pair should fail");

    assert!(
        err.to_string().contains("unmatched pair"),
        "unexpected error: {err}"
    );
}

#[test]
fn incomplete_telemetry_blocks_kpi_report_generation() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let report = execute_batch_from_manifest_file(
        &fixture_path("batch_incomplete_telemetry_manifest.json"),
        temp.path(),
    )
    .expect("batch run should finish with diagnostics");

    assert!(!report.telemetry_complete);
    assert!(report.kpi.is_none());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diag| diag.error_class == Some(ErrorClass::TelemetryIncomplete))
    );
}

#[test]
fn retry_limit_breach_is_reported_as_batch_failure() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let report = execute_batch_from_manifest_file(
        &fixture_path("batch_retry_breach_manifest.json"),
        temp.path(),
    )
    .expect("batch run should complete");

    assert!(!report.retry_policy_ok);
    assert!(report.diagnostics.iter().any(|diag| {
        diag.error_class == Some(ErrorClass::RetryPolicy)
            && diag.error_code == Some(DiagnosticCode::Pipeline(ErrorCode::ConflictRetryExhausted))
    }));
}

#[test]
fn provenance_mismatch_blocks_decision_grade_kpi_claims() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let report = execute_batch_from_manifest_file(
        &fixture_path("batch_complete_manifest.json"),
        temp.path(),
    )
    .expect("batch run should complete");

    let summary_path = temp
        .path()
        .join("artifacts")
        .join("run-a-optimized")
        .join("summary.json");
    let mut summary_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&summary_path).expect("summary should exist"))
            .expect("summary should deserialize");
    summary_json["runtime_mode"] = serde_json::json!(RUNTIME_LIVE);
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary_json).expect("summary serialization should succeed"),
    )
    .expect("summary should be written");

    let manifest: RunManifest = serde_json::from_str(
        &fs::read_to_string(
            temp.path()
                .join("artifacts")
                .join("batch")
                .join("manifest.normalized.json"),
        )
        .expect("normalized manifest should exist"),
    )
    .expect("manifest should deserialize");

    let rebuilt = rebuild_batch_report_from_artifacts(
        &manifest,
        temp.path(),
        "artifacts/batch/artifact-index.json",
        &report.provenance,
    )
    .expect("rebuild should succeed");

    assert!(rebuilt.kpi.is_none());
    assert!(
        rebuilt
            .gate_checks
            .iter()
            .any(|check| { check.name == "provenance_complete" && !check.pass })
    );
    assert!(rebuilt.diagnostics.iter().any(|diag| {
        diag.error_class == Some(ErrorClass::ProvenanceIncomplete)
            && diag.error_code == Some(DiagnosticCode::ProvenanceMismatch)
    }));
}

#[test]
fn unmapped_live_runtime_hard_error_is_reported_deterministically() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let report = execute_batch_from_manifest_file(
        &fixture_path("batch_complete_manifest.json"),
        temp.path(),
    )
    .expect("batch run should complete");

    let summary_path = temp
        .path()
        .join("artifacts")
        .join("run-b-optimized")
        .join("summary.json");
    let mut summary_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&summary_path).expect("summary should exist"))
            .expect("summary should deserialize");
    summary_json["success"] = serde_json::json!(false);
    summary_json["error_codes"] = serde_json::json!([ErrorCode::RuntimeUnmappedHard.as_str()]);
    summary_json["failure_state"] = serde_json::json!("publish");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary_json).expect("summary serialization should succeed"),
    )
    .expect("summary should be written");

    let manifest: RunManifest = serde_json::from_str(
        &fs::read_to_string(
            temp.path()
                .join("artifacts")
                .join("batch")
                .join("manifest.normalized.json"),
        )
        .expect("normalized manifest should exist"),
    )
    .expect("manifest should deserialize");

    let rebuilt = rebuild_batch_report_from_artifacts(
        &manifest,
        temp.path(),
        "artifacts/batch/artifact-index.json",
        &report.provenance,
    )
    .expect("rebuild should succeed");

    assert!(rebuilt.diagnostics.iter().any(|diag| {
        diag.error_class == Some(ErrorClass::RuntimeUnmappedHard)
            && diag.error_code == Some(DiagnosticCode::Pipeline(ErrorCode::RuntimeUnmappedHard))
    }));
}

#[test]
fn drift_and_coverage_gate_failures_are_reflected_in_report() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let drift_report = execute_batch_from_manifest_file(
        &fixture_path("batch_drift_failure_manifest.json"),
        temp.path(),
    )
    .expect("batch run should complete");
    assert!(drift_report.drift.unresolved_material_drift);
    assert_eq!(drift_report.recommendation.decision, "stop");

    let coverage_report = execute_batch_from_manifest_file(
        &fixture_path("batch_coverage_failure_manifest.json"),
        temp.path(),
    )
    .expect("batch run should complete");
    assert!(!coverage_report.scenario_coverage.complete);
    assert_eq!(coverage_report.recommendation.decision, "iterate");
}

#[test]
fn report_is_reproducible_from_stored_run_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let manifest_path = fixture_path("batch_complete_manifest.json");
    let report = execute_batch_from_manifest_file(&manifest_path, temp.path())
        .expect("batch run should complete");

    let manifest_text = fs::read_to_string(
        temp.path()
            .join("artifacts")
            .join("batch")
            .join("manifest.normalized.json"),
    )
    .expect("normalized manifest should exist");
    let manifest: RunManifest =
        serde_json::from_str(&manifest_text).expect("manifest should deserialize");

    let rebuilt = rebuild_batch_report_from_artifacts(
        &manifest,
        temp.path(),
        "artifacts/batch/artifact-index.json",
        &report.provenance,
    )
    .expect("rebuild from artifacts should succeed");

    let stored_text = fs::read_to_string(
        temp.path()
            .join("artifacts")
            .join("batch")
            .join("report.json"),
    )
    .expect("stored report should exist");
    let stored: BatchReport =
        serde_json::from_str(&stored_text).expect("report should deserialize");

    assert_eq!(report.provenance, rebuilt.provenance);
    assert_eq!(report.diagnostics, rebuilt.diagnostics);
    assert_eq!(stored, rebuilt);
}
