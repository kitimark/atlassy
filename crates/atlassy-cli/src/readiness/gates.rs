use atlassy_contracts::ErrorCode;

use crate::{
    LifecycleClaims, ReadinessChecklist, ReadinessEvidence, ReadinessGateResult,
    ReadinessOwnerRoles,
};

pub(crate) fn evaluate_readiness_gates(evidence: &ReadinessEvidence) -> ReadinessChecklist {
    let generated_ts = deterministic_generated_ts(evidence);
    let gate_1_pass = !evidence.summaries.is_empty()
        && evidence
            .summaries
            .values()
            .all(|summary| !summary.pipeline_version.is_empty());
    let gate_2_pass = !evidence.manifest.runs.is_empty();
    let gate_3_pass = evidence.report.retry_policy_ok && !evidence.report.safety.safety_failed;
    let gate_4_pass = evidence.report.scenario_coverage.complete;
    let gate_5_pass = evidence.report.telemetry_complete && evidence.report.kpi.is_some();
    let gate_6_pass = !evidence.report.drift.unresolved_material_drift;

    let lifecycle_claims = lifecycle_claims_from_attestations(evidence);
    let summary_has_bootstrap_required_failure = evidence.summaries.values().any(|s| {
        s.empty_page_detected
            && !s.success
            && s.error_codes
                .iter()
                .any(|c| c == ErrorCode::BootstrapRequired.as_str())
    });
    let summary_has_bootstrap_success = evidence
        .summaries
        .values()
        .any(|s| s.bootstrap_applied && s.success);
    let summary_has_bootstrap_invalid_state = evidence.summaries.values().any(|s| {
        !s.empty_page_detected
            && !s.success
            && s.error_codes
                .iter()
                .any(|c| c == ErrorCode::BootstrapInvalidState.as_str())
    });
    let summary_has_create_subpage_evidence =
        evidence.manifest.batch.lifecycle_create_subpage_validated;

    let has_bootstrap_required_failure = lifecycle_claims
        .as_ref()
        .is_some_and(|claims| claims.bootstrap_required_failure)
        || summary_has_bootstrap_required_failure;
    let has_bootstrap_success = lifecycle_claims
        .as_ref()
        .is_some_and(|claims| claims.bootstrap_success)
        || summary_has_bootstrap_success;
    let has_bootstrap_invalid_state = lifecycle_claims
        .as_ref()
        .is_some_and(|claims| claims.bootstrap_on_non_empty_failure)
        || summary_has_bootstrap_invalid_state;
    let has_create_subpage_evidence = lifecycle_claims
        .as_ref()
        .is_some_and(|claims| claims.create_subpage_validated)
        || summary_has_create_subpage_evidence;

    let attestation_contributes = lifecycle_claims.as_ref().is_some_and(|claims| {
        (claims.bootstrap_required_failure && !summary_has_bootstrap_required_failure)
            || (claims.bootstrap_success && !summary_has_bootstrap_success)
            || (claims.bootstrap_on_non_empty_failure && !summary_has_bootstrap_invalid_state)
            || (claims.create_subpage_validated && !summary_has_create_subpage_evidence)
    });

    let mut gate_7_evidence_refs = vec![
        "artifacts/batch/manifest.normalized.json".to_string(),
        "artifacts/batch/report.json".to_string(),
    ];
    if attestation_contributes {
        gate_7_evidence_refs.push("artifacts/batch/attestations.json".to_string());
    }
    let gate_7_pass = has_bootstrap_required_failure
        && has_bootstrap_success
        && has_bootstrap_invalid_state
        && has_create_subpage_evidence;

    let gates = vec![
        readiness_gate_result(
            "gate_1_design_and_contract_freeze",
            "Design and Contract Freeze",
            "v1 route matrix and contract evidence are present",
            gate_1_pass,
            "engineering_owner",
            vec![
                "artifacts/batch/manifest.normalized.json".to_string(),
                "artifacts/batch/artifact-index.json".to_string(),
            ],
            if gate_1_pass {
                None
            } else {
                Some("pipeline version metadata is incomplete".to_string())
            },
        ),
        readiness_gate_result(
            "gate_2_environment_and_access",
            "Environment and Access",
            "batch evidence exists and run set is non-empty",
            gate_2_pass,
            "engineering_owner",
            vec!["artifacts/batch/manifest.normalized.json".to_string()],
            if gate_2_pass {
                None
            } else {
                Some("no runs found in normalized manifest".to_string())
            },
        ),
        readiness_gate_result(
            "gate_3_pipeline_integrity",
            "Pipeline Integrity",
            "safety gates hold and retry policy remains bounded",
            gate_3_pass,
            "qa_owner",
            vec!["artifacts/batch/report.json".to_string()],
            if gate_3_pass {
                None
            } else {
                Some("safety violation or retry-policy breach detected".to_string())
            },
        ),
        readiness_gate_result(
            "gate_4_test_and_fixture_coverage",
            "Test and Fixture Coverage",
            "required scenario IDs are covered by batch evidence",
            gate_4_pass,
            "qa_owner",
            vec!["artifacts/batch/report.json".to_string()],
            if gate_4_pass {
                None
            } else {
                Some("required scenario coverage is incomplete".to_string())
            },
        ),
        readiness_gate_result(
            "gate_5_metrics_and_reporting",
            "Metrics and Reporting",
            "telemetry is complete and aggregate KPI report is present",
            gate_5_pass,
            "data_metrics_owner",
            vec!["artifacts/batch/report.json".to_string()],
            if gate_5_pass {
                None
            } else {
                Some("telemetry completeness or KPI aggregation is missing".to_string())
            },
        ),
        readiness_gate_result(
            "gate_6_risk_control_activation",
            "Risk Control Activation",
            "no unresolved material drift remains",
            gate_6_pass,
            "release_reviewer",
            vec!["artifacts/batch/report.json".to_string()],
            if gate_6_pass {
                None
            } else {
                Some("live-vs-stub material drift remains unresolved".to_string())
            },
        ),
        readiness_gate_result(
            "gate_7_lifecycle_enablement_validation",
            "Lifecycle Enablement Validation",
            "lifecycle matrix evidence covers bootstrap and create-subpage paths",
            gate_7_pass,
            "qa_owner",
            gate_7_evidence_refs,
            if gate_7_pass {
                None
            } else {
                Some("lifecycle evidence is incomplete: requires bootstrap-required failure, bootstrap success, bootstrap-on-non-empty failure, and create-subpage validation".to_string())
            },
        ),
    ];

    let blocked = gates.iter().any(|gate| gate.mandatory && !gate.pass);

    ReadinessChecklist {
        schema_version: "v1".to_string(),
        generated_ts,
        owner_roles: readiness_owner_roles(),
        source_artifacts: evidence.source_artifacts.clone(),
        provenance: evidence.provenance.clone(),
        gates,
        blocked,
    }
}

fn readiness_gate_result(
    gate_id: &str,
    gate_name: &str,
    target: &str,
    pass: bool,
    owner_role: &str,
    evidence_refs: Vec<String>,
    blocking_reason: Option<String>,
) -> ReadinessGateResult {
    ReadinessGateResult {
        gate_id: gate_id.to_string(),
        gate_name: gate_name.to_string(),
        target: target.to_string(),
        pass,
        mandatory: true,
        owner_role: owner_role.to_string(),
        blocking_reason,
        evidence_refs,
    }
}

fn readiness_owner_roles() -> ReadinessOwnerRoles {
    ReadinessOwnerRoles {
        product_owner: "product_owner".to_string(),
        engineering_owner: "engineering_owner".to_string(),
        qa_owner: "qa_owner".to_string(),
        data_metrics_owner: "data_metrics_owner".to_string(),
        release_reviewer: "release_reviewer".to_string(),
    }
}

fn deterministic_generated_ts(evidence: &ReadinessEvidence) -> String {
    evidence
        .manifest
        .runs
        .iter()
        .map(|run| run.timestamp.as_str())
        .max()
        .unwrap_or("1970-01-01T00:00:00Z")
        .to_string()
}

fn lifecycle_claims_from_attestations(evidence: &ReadinessEvidence) -> Option<LifecycleClaims> {
    let entry = evidence
        .attestations
        .entries
        .iter()
        .find(|entry| entry.attestation_id == "lifecycle_validation")?;
    serde_json::from_value::<LifecycleClaims>(entry.claims.clone()).ok()
}
