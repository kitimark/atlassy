use std::fs;
use std::path::Path;

use crate::batch::summarize_failure_classes;
use crate::io::load_required_json;
use crate::readiness::load_readiness_evidence;
use crate::{
    BatchReport, DecisionPacket, DynError, ReadinessChecklist, ReadinessEvidence, ReadinessOutputs,
    RiskStatusDelta, RunbookBundle,
};

pub(crate) fn build_risk_status_deltas(report: &BatchReport) -> Vec<RiskStatusDelta> {
    let mut deltas = vec![
        risk_delta(
            "R-001",
            "Out-of-scope mutation",
            "high",
            report.safety.out_of_scope_violation_runs.is_empty(),
            "out-of-scope mutation diagnostics",
        ),
        risk_delta(
            "R-002",
            "Locked structural node mutation",
            "high",
            report.safety.locked_node_violation_runs.is_empty(),
            "locked-node mutation diagnostics",
        ),
        risk_delta(
            "R-003",
            "Table shape drift",
            "high",
            report.safety.table_shape_violation_runs.is_empty(),
            "table-shape violation diagnostics",
        ),
        risk_delta(
            "R-004",
            "Insufficient context reduction",
            "medium",
            report
                .kpi
                .as_ref()
                .and_then(|kpi| {
                    kpi.checks
                        .iter()
                        .find(|check| check.name == "context_reduction_ratio")
                })
                .is_none_or(|check| check.pass),
            "context reduction KPI result",
        ),
        risk_delta(
            "R-005",
            "Conflict retry token waste",
            "medium",
            report.retry_policy_ok,
            "retry policy gate outcome",
        ),
        risk_delta(
            "R-006",
            "Schema-invalid candidate payloads",
            "high",
            !report
                .diagnostics
                .iter()
                .any(|diag| diag.error_code.as_deref() == Some("ERR_SCHEMA_INVALID")),
            "verify diagnostics error codes",
        ),
        risk_delta(
            "R-007",
            "Metrics instrumentation gaps",
            "medium",
            report.telemetry_complete,
            "telemetry completeness gate",
        ),
        risk_delta(
            "R-008",
            "External service variance masking regressions",
            "medium",
            !report.drift.unresolved_material_drift,
            "live-vs-stub drift status",
        ),
    ];

    deltas.sort_by(|left, right| left.risk_id.cmp(&right.risk_id));
    deltas
}

fn risk_delta(
    risk_id: &str,
    title: &str,
    priority: &str,
    mitigated: bool,
    reason: &str,
) -> RiskStatusDelta {
    RiskStatusDelta {
        risk_id: risk_id.to_string(),
        title: title.to_string(),
        priority: priority.to_string(),
        previous_status: "watch".to_string(),
        current_status: if mitigated {
            "mitigated".to_string()
        } else {
            "open".to_string()
        },
        reason: reason.to_string(),
    }
}

pub(crate) fn assemble_decision_packet(
    evidence: &ReadinessEvidence,
    checklist: &ReadinessChecklist,
    runbook_bundle: &RunbookBundle,
    risk_status_deltas: Vec<RiskStatusDelta>,
) -> DecisionPacket {
    let kpi_failed = evidence
        .report
        .kpi
        .as_ref()
        .is_some_and(|kpi| kpi.checks.iter().any(|check| !check.pass));

    let mut blocking_condition = None;
    let recommendation = if evidence.report.safety.safety_failed {
        blocking_condition = Some("safety_gates_failed".to_string());
        "stop"
    } else if evidence.report.drift.unresolved_material_drift {
        blocking_condition = Some("unresolved_material_drift".to_string());
        "stop"
    } else if checklist.blocked || runbook_bundle.blocked {
        blocking_condition = checklist
            .gates
            .iter()
            .find(|gate| !gate.pass)
            .map(|gate| gate.gate_id.clone())
            .or_else(|| {
                runbook_bundle
                    .sections
                    .iter()
                    .find(|section| section.blocks_signoff)
                    .map(|section| section.failure_class.clone())
            });
        "iterate"
    } else if kpi_failed {
        blocking_condition = Some("kpi_target_miss".to_string());
        "iterate"
    } else {
        "go"
    }
    .to_string();

    let mut rationale = Vec::new();
    rationale.extend(
        checklist
            .gates
            .iter()
            .filter(|gate| !gate.pass)
            .map(|gate| {
                format!(
                    "{} failed: {}",
                    gate.gate_name,
                    gate.blocking_reason
                        .clone()
                        .unwrap_or_else(|| "blocking condition detected".to_string())
                )
            }),
    );
    rationale.extend(
        runbook_bundle
            .sections
            .iter()
            .filter(|section| section.blocks_signoff)
            .map(|section| {
                format!(
                    "runbook class {} requires escalation ({})",
                    section.failure_class, section.severity
                )
            }),
    );
    if let Some(kpi) = &evidence.report.kpi {
        rationale.extend(
            kpi.checks
                .iter()
                .filter(|check| !check.pass)
                .map(|check| format!("kpi target miss: {} ({})", check.name, check.target)),
        );
    }
    rationale.sort();
    rationale.dedup();

    let top_failure_classes = summarize_failure_classes(&evidence.report, runbook_bundle);

    DecisionPacket {
        schema_version: "v1".to_string(),
        generated_ts: checklist.generated_ts.clone(),
        recommendation,
        provenance: checklist.provenance.clone(),
        blocking_condition,
        rationale,
        gate_outcomes: checklist.gates.clone(),
        kpi_summary: evidence.report.kpi.clone(),
        risk_status_deltas,
        top_failure_classes,
        checklist_path: "artifacts/batch/readiness.checklist.json".to_string(),
        runbook_path: "artifacts/batch/runbook.bundle.json".to_string(),
        report_path: "artifacts/batch/report.json".to_string(),
    }
}

pub(crate) fn persist_decision_packet(
    artifacts_dir: &Path,
    outputs: &ReadinessOutputs,
) -> Result<(), DynError> {
    let batch_dir = artifacts_dir.join("artifacts").join("batch");
    fs::create_dir_all(&batch_dir)?;
    fs::write(
        batch_dir.join("readiness.checklist.json"),
        serde_json::to_string_pretty(&outputs.checklist)?,
    )?;
    fs::write(
        batch_dir.join("runbook.bundle.json"),
        serde_json::to_string_pretty(&outputs.runbook_bundle)?,
    )?;
    fs::write(
        batch_dir.join("decision.packet.json"),
        serde_json::to_string_pretty(&outputs.decision_packet)?,
    )?;
    Ok(())
}

fn replay_decision_packet(artifacts_dir: &Path) -> Result<DecisionPacket, DynError> {
    let evidence = load_readiness_evidence(artifacts_dir)?;
    let checklist_path = artifacts_dir
        .join("artifacts")
        .join("batch")
        .join("readiness.checklist.json");
    let runbook_path = artifacts_dir
        .join("artifacts")
        .join("batch")
        .join("runbook.bundle.json");

    let checklist: ReadinessChecklist = load_required_json(&checklist_path)?;
    let runbook_bundle: RunbookBundle = load_required_json(&runbook_path)?;
    let risk_status_deltas = build_risk_status_deltas(&evidence.report);

    Ok(assemble_decision_packet(
        &evidence,
        &checklist,
        &runbook_bundle,
        risk_status_deltas,
    ))
}

pub fn verify_decision_packet_replay(artifacts_dir: &Path) -> Result<(), DynError> {
    let stored_path = artifacts_dir
        .join("artifacts")
        .join("batch")
        .join("decision.packet.json");
    let stored: DecisionPacket = load_required_json(&stored_path)?;
    let rebuilt = replay_decision_packet(artifacts_dir)?;
    if stored != rebuilt {
        return Err(
            "readiness replay mismatch: rebuilt decision packet diverges from stored output"
                .to_string()
                .into(),
        );
    }
    Ok(())
}
