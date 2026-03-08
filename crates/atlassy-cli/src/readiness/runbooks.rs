use std::collections::BTreeSet;

use crate::{BatchReport, ErrorClass, ReadinessChecklist, RunbookBundle, RunbookSection};

pub(crate) fn build_operator_runbooks(
    report: &BatchReport,
    checklist: &ReadinessChecklist,
) -> RunbookBundle {
    let mut sections = Vec::new();
    let mut diagnostic_sections = BTreeSet::new();

    for diag in &report.diagnostics {
        match diag.error_class {
            Some(ErrorClass::PipelineHard) => {
                if diag
                    .message
                    .as_deref()
                    .is_some_and(|message| message.contains("`verify`"))
                    && diagnostic_sections.insert("verify_hard_failure")
                {
                    sections.push(known_runbook_section(
                        "verify_hard_failure",
                        "high",
                        "qa_owner",
                        "engineering_owner",
                        "recurs for two consecutive batches or blocks all optimized runs",
                        vec![
                            "inspect verify diagnostics for schema, scope, and route violations",
                            "capture offending paths and create regression fixtures",
                            "rerun affected scenario IDs before resuming batch",
                        ],
                        vec![
                            "artifacts/<run_id>/verify/diagnostics.json",
                            "artifacts/<run_id>/summary.json",
                        ],
                    ));
                }
            }
            Some(ErrorClass::RetryPolicy) => {
                if diagnostic_sections.insert("retry_exhaustion") {
                    sections.push(known_runbook_section(
                        "retry_exhaustion",
                        "high",
                        "engineering_owner",
                        "release_reviewer",
                        "any run exceeds one scoped retry",
                        vec![
                            "stop the batch and preserve publish diagnostics",
                            "review scoped rebase behavior and conflict surface",
                            "resume only after retry policy compliance is restored",
                        ],
                        vec![
                            "artifacts/<run_id>/publish/diagnostics.json",
                            "artifacts/batch/report.json",
                        ],
                    ));
                }
            }
            Some(ErrorClass::TelemetryIncomplete) => {
                if diagnostic_sections.insert("telemetry_incomplete") {
                    sections.push(known_runbook_section(
                        "telemetry_incomplete",
                        "high",
                        "data_metrics_owner",
                        "engineering_owner",
                        "required telemetry fields are missing from any run",
                        vec![
                            "mark affected runs non-evaluable",
                            "repair telemetry emission and rerun paired keys",
                            "rebuild aggregate report and readiness outputs",
                        ],
                        vec![
                            "artifacts/<run_id>/summary.json",
                            "artifacts/batch/report.json",
                        ],
                    ));
                }
            }
            Some(ErrorClass::Io)
            | Some(ErrorClass::ProvenanceIncomplete)
            | Some(ErrorClass::RuntimeUnmappedHard)
            | None => {}
        }
    }

    if report.safety.safety_failed {
        sections.push(known_runbook_section(
            "safety_gate_violation",
            "critical",
            "qa_owner",
            "release_reviewer",
            "any locked-node, out-of-scope, or table-shape violation is detected",
            vec![
                "pause release-readiness decision flow immediately",
                "isolate violating runs and classify by error code",
                "create targeted regression tests before rerun",
            ],
            vec![
                "artifacts/batch/report.json",
                "artifacts/<run_id>/summary.json",
            ],
        ));
    }

    if report.drift.unresolved_material_drift {
        sections.push(known_runbook_section(
            "unresolved_drift",
            "critical",
            "engineering_owner",
            "release_reviewer",
            "live-vs-stub parity check fails on key behavior",
            vec![
                "suspend sign-off and document drift scope",
                "update stub scenarios and rerun affected suites",
                "record drift resolution before next recommendation",
            ],
            vec!["artifacts/batch/report.json"],
        ));
    }

    if sections.is_empty() {
        sections.push(known_runbook_section(
            "no_active_failures",
            "low",
            "release_reviewer",
            "release_reviewer",
            "no failure-class triage required",
            vec!["confirm checklist and KPI gates remain passing"],
            vec!["artifacts/batch/report.json"],
        ));
    }

    sections.sort_by(|left, right| left.failure_class.cmp(&right.failure_class));
    let blocked = checklist.blocked || sections.iter().any(|section| section.blocks_signoff);

    RunbookBundle {
        schema_version: "v1".to_string(),
        generated_ts: checklist.generated_ts.clone(),
        provenance: checklist.provenance.clone(),
        sections,
        blocked,
    }
}

fn known_runbook_section(
    failure_class: &str,
    severity: &str,
    primary_owner_role: &str,
    escalation_owner_role: &str,
    escalation_trigger: &str,
    triage_steps: Vec<&str>,
    evidence_checks: Vec<&str>,
) -> RunbookSection {
    RunbookSection {
        failure_class: failure_class.to_string(),
        severity: severity.to_string(),
        primary_owner_role: primary_owner_role.to_string(),
        escalation_owner_role: escalation_owner_role.to_string(),
        escalation_trigger: escalation_trigger.to_string(),
        triage_steps: triage_steps
            .into_iter()
            .map(|step| step.to_string())
            .collect(),
        evidence_checks: evidence_checks
            .into_iter()
            .map(|check| check.to_string())
            .collect(),
        fallback: false,
        blocks_signoff: matches!(severity, "critical" | "high")
            && failure_class != "no_active_failures",
    }
}
