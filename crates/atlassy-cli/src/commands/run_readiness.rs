use std::path::Path;

use crate::readiness::{
    assemble_decision_packet, build_operator_runbooks, build_risk_status_deltas,
    evaluate_readiness_gates, load_readiness_evidence, persist_decision_packet,
};
use crate::{DecisionPacket, DynError, ReadinessOutputs};

pub fn generate_readiness_outputs_from_artifacts(
    artifacts_dir: &Path,
) -> Result<ReadinessOutputs, DynError> {
    let evidence = load_readiness_evidence(artifacts_dir)?;
    let checklist = evaluate_readiness_gates(&evidence);
    let runbook_bundle = build_operator_runbooks(&evidence.report, &checklist);
    let risk_status_deltas = build_risk_status_deltas(&evidence.report);
    let decision_packet =
        assemble_decision_packet(&evidence, &checklist, &runbook_bundle, risk_status_deltas);

    let outputs = ReadinessOutputs {
        checklist,
        runbook_bundle,
        decision_packet,
    };

    persist_decision_packet(artifacts_dir, &outputs)?;
    Ok(outputs)
}

pub fn ensure_readiness_unblocked(decision_packet: &DecisionPacket) -> Result<(), DynError> {
    if decision_packet.recommendation == "go" {
        return Ok(());
    }

    let condition = decision_packet
        .blocking_condition
        .clone()
        .unwrap_or_else(|| "unspecified blocking condition".to_string());
    Err(format!(
        "readiness blocked: recommendation={} due to {}",
        decision_packet.recommendation, condition
    )
    .into())
}
