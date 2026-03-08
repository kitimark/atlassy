use std::fs;

use atlassy_cli::*;

mod common;

use common::fixture_path;

#[test]
fn readiness_checklist_is_deterministic_and_blocks_on_mandatory_failures() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(&fixture_path("batch_complete_manifest.json"), temp.path())
        .expect("batch run should complete");

    let readiness = generate_readiness_outputs_from_artifacts(temp.path())
        .expect("readiness outputs should be generated");

    let gate_ids = readiness
        .checklist
        .gates
        .iter()
        .map(|gate| gate.gate_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        gate_ids,
        vec![
            "gate_1_design_and_contract_freeze",
            "gate_2_environment_and_access",
            "gate_3_pipeline_integrity",
            "gate_4_test_and_fixture_coverage",
            "gate_5_metrics_and_reporting",
            "gate_6_risk_control_activation",
            "gate_7_lifecycle_enablement_validation",
        ]
    );
    assert!(readiness.checklist.gates.iter().all(|gate| gate.mandatory));
    assert!(!readiness.checklist.blocked);
    assert!(
        temp.path()
            .join("artifacts")
            .join("batch")
            .join("readiness.checklist.json")
            .exists()
    );

    let blocked_temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_coverage_failure_manifest.json"),
        blocked_temp.path(),
    )
    .expect("batch run should complete");
    let blocked = generate_readiness_outputs_from_artifacts(blocked_temp.path())
        .expect("readiness outputs should be generated");
    assert!(blocked.checklist.blocked);
    assert!(
        blocked
            .checklist
            .gates
            .iter()
            .any(|gate| { gate.gate_id == "gate_4_test_and_fixture_coverage" && !gate.pass })
    );
}

#[test]
fn runbook_generation_includes_metadata_and_rejects_unknown_failure_class() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_retry_breach_manifest.json"),
        temp.path(),
    )
    .expect("batch run should complete");

    let readiness = generate_readiness_outputs_from_artifacts(temp.path())
        .expect("readiness outputs should be generated");
    let retry_section = readiness
        .runbook_bundle
        .sections
        .iter()
        .find(|section| section.failure_class == "retry_exhaustion")
        .expect("retry runbook section should exist");
    assert_eq!(retry_section.severity, "high");
    assert!(!retry_section.primary_owner_role.is_empty());
    assert!(!retry_section.escalation_owner_role.is_empty());
    assert!(!retry_section.escalation_trigger.is_empty());

    let unknown_temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_complete_manifest.json"),
        unknown_temp.path(),
    )
    .expect("batch run should complete");

    let report_path = unknown_temp
        .path()
        .join("artifacts")
        .join("batch")
        .join("report.json");
    let report_text = fs::read_to_string(&report_path).expect("report should exist");
    let mut report_json: serde_json::Value =
        serde_json::from_str(&report_text).expect("report should deserialize");
    report_json
        .get_mut("diagnostics")
        .and_then(|value| value.as_array_mut())
        .expect("diagnostics array should exist")
        .push(serde_json::json!({
            "run_id": "run-a-baseline",
            "page_id": "18841604",
            "pattern": "A",
            "flow": "baseline",
            "status": "failed",
            "error_class": "mystery_class",
            "error_code": "ERR_MYSTERY",
            "message": "unexpected class"
        }));
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&report_json).expect("report serialization should succeed"),
    )
    .expect("report should be written");

    let err = generate_readiness_outputs_from_artifacts(unknown_temp.path())
        .expect_err("unknown failure classes should fail deserialization");
    assert!(
        err.to_string().contains("mystery_class"),
        "unexpected error: {err}"
    );
}

#[test]
fn decision_packet_contains_required_sections_and_precedence() {
    let drift_temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_drift_failure_manifest.json"),
        drift_temp.path(),
    )
    .expect("batch run should complete");
    let drift_readiness = generate_readiness_outputs_from_artifacts(drift_temp.path())
        .expect("readiness outputs should be generated");

    assert_eq!(drift_readiness.decision_packet.recommendation, "stop");
    assert_eq!(
        drift_readiness
            .decision_packet
            .blocking_condition
            .as_deref(),
        Some("unresolved_material_drift")
    );
    assert!(!drift_readiness.decision_packet.gate_outcomes.is_empty());
    assert!(
        !drift_readiness
            .decision_packet
            .risk_status_deltas
            .is_empty()
    );
    assert!(
        !drift_readiness
            .decision_packet
            .top_failure_classes
            .is_empty()
    );

    let coverage_temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_coverage_failure_manifest.json"),
        coverage_temp.path(),
    )
    .expect("batch run should complete");
    let coverage_readiness = generate_readiness_outputs_from_artifacts(coverage_temp.path())
        .expect("readiness outputs should be generated");
    assert_eq!(coverage_readiness.decision_packet.recommendation, "iterate");
    assert_eq!(
        coverage_readiness
            .decision_packet
            .blocking_condition
            .as_deref(),
        Some("gate_4_test_and_fixture_coverage")
    );
}

#[test]
fn readiness_replay_verification_detects_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(&fixture_path("batch_complete_manifest.json"), temp.path())
        .expect("batch run should complete");
    let _ = generate_readiness_outputs_from_artifacts(temp.path())
        .expect("readiness outputs should be generated");

    verify_decision_packet_replay(temp.path()).expect("replay should match before mutation");

    let packet_path = temp
        .path()
        .join("artifacts")
        .join("batch")
        .join("decision.packet.json");
    let packet_text = fs::read_to_string(&packet_path).expect("packet should exist");
    let mut packet_json: serde_json::Value =
        serde_json::from_str(&packet_text).expect("packet should deserialize");
    packet_json["recommendation"] = serde_json::json!("stop");
    fs::write(
        &packet_path,
        serde_json::to_string_pretty(&packet_json).expect("packet serialization should succeed"),
    )
    .expect("packet should be written");

    let err = verify_decision_packet_replay(temp.path())
        .expect_err("replay mismatch should fail verification");
    assert!(
        err.to_string().contains("readiness replay mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn gate_7_lifecycle_evidence_controls_pass_and_iterate_recommendation() {
    let no_lifecycle_temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_coverage_failure_manifest.json"),
        no_lifecycle_temp.path(),
    )
    .expect("batch run should complete");
    let no_lifecycle = generate_readiness_outputs_from_artifacts(no_lifecycle_temp.path())
        .expect("readiness outputs should be generated");
    let gate_7 = no_lifecycle
        .checklist
        .gates
        .iter()
        .find(|gate| gate.gate_id == "gate_7_lifecycle_enablement_validation")
        .expect("gate 7 should be present");
    assert!(
        !gate_7.pass,
        "gate 7 should fail without lifecycle evidence"
    );
    assert!(gate_7.mandatory);
    assert_eq!(gate_7.owner_role, "qa_owner");
    assert!(
        gate_7
            .blocking_reason
            .as_ref()
            .is_some_and(|reason| reason.contains("lifecycle evidence is incomplete"))
    );
    assert!(no_lifecycle.checklist.blocked);
    assert_eq!(no_lifecycle.decision_packet.recommendation, "iterate");

    let with_lifecycle_temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_complete_manifest.json"),
        with_lifecycle_temp.path(),
    )
    .expect("batch run should complete");
    let with_lifecycle = generate_readiness_outputs_from_artifacts(with_lifecycle_temp.path())
        .expect("readiness outputs should be generated");
    let gate_7 = with_lifecycle
        .checklist
        .gates
        .iter()
        .find(|gate| gate.gate_id == "gate_7_lifecycle_enablement_validation")
        .expect("gate 7 should be present");
    assert!(gate_7.pass, "gate 7 should pass with lifecycle evidence");
    assert!(gate_7.blocking_reason.is_none());
}

#[test]
fn readiness_errors_are_operator_facing() {
    let empty = tempfile::tempdir().expect("tempdir should be created");
    let missing_err = generate_readiness_outputs_from_artifacts(empty.path())
        .expect_err("missing evidence should fail");
    assert!(
        missing_err
            .to_string()
            .contains("missing readiness evidence"),
        "unexpected error: {missing_err}"
    );

    let blocked_temp = tempfile::tempdir().expect("tempdir should be created");
    execute_batch_from_manifest_file(
        &fixture_path("batch_coverage_failure_manifest.json"),
        blocked_temp.path(),
    )
    .expect("batch run should complete");
    let blocked = generate_readiness_outputs_from_artifacts(blocked_temp.path())
        .expect("readiness outputs should be generated");
    let blocked_err = ensure_readiness_unblocked(&blocked.decision_packet)
        .expect_err("blocked readiness should return operator-facing error");
    assert!(
        blocked_err.to_string().contains("readiness blocked"),
        "unexpected error: {blocked_err}"
    );
}
