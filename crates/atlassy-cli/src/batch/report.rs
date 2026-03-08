use std::collections::BTreeMap;
use std::path::Path;

use atlassy_contracts::{
    ErrorCode, PipelineState, ProvenanceStamp, RunSummary, validate_run_summary_telemetry,
};

use crate::batch::{
    assess_drift, assess_safety, assess_scenario_coverage, build_kpi_report, build_recommendation,
    collect_flow_groups,
};
use crate::io::load_run_summary;
use crate::manifest::observed_scenario_ids;
use crate::provenance::{provenance_matches, summary_telemetry_complete};
use crate::{
    BatchArtifactIndex, BatchArtifactMetadata, BatchReport, BatchRunDiagnostic, DynError,
    FailureClassSummary, GateCheck, ManifestRunEntry, RunArtifactIndexEntry, RunManifest,
    RunbookBundle, StateArtifactIndexEntry,
};

pub fn rebuild_batch_report_from_artifacts(
    manifest: &RunManifest,
    artifacts_dir: &Path,
    artifact_index_path: &str,
    provenance: &ProvenanceStamp,
) -> Result<BatchReport, DynError> {
    let mut summaries = BTreeMap::new();
    for run in &manifest.runs {
        if let Some(summary) = load_run_summary(artifacts_dir, &run.run_id)? {
            summaries.insert(run.run_id.clone(), summary);
        }
    }

    let diagnostics = manifest
        .runs
        .iter()
        .map(|run| classify_run_from_summary(run, summaries.get(&run.run_id), provenance))
        .collect::<Vec<_>>();

    let failed_runs = diagnostics
        .iter()
        .filter(|diag| diag.status == "failed")
        .count();
    let telemetry_complete = manifest.runs.iter().all(|run| {
        summaries.get(&run.run_id).is_some_and(|summary| {
            let early_failure = summary.failure_state.is_some()
                && summary.failure_state != Some(PipelineState::Verify)
                && summary.failure_state != Some(PipelineState::Publish);
            early_failure || summary_telemetry_complete(summary)
        })
    });
    let provenance_complete = manifest.runs.iter().all(|run| {
        summaries.get(&run.run_id).is_some_and(|summary| {
            let early_failure = summary.failure_state.is_some()
                && summary.failure_state != Some(PipelineState::Verify)
                && summary.failure_state != Some(PipelineState::Publish);
            early_failure
                || (summary_telemetry_complete(summary) && provenance_matches(summary, provenance))
        })
    });
    let retry_policy_ok = manifest.runs.iter().all(|run| {
        summaries
            .get(&run.run_id)
            .is_some_and(|summary| summary.retry_count <= 1)
    });

    let drift = assess_drift(&manifest.batch.live_smoke);
    let scenario_coverage = assess_scenario_coverage(manifest);
    let safety = assess_safety(&summaries);
    let flow_groups = collect_flow_groups(manifest, &summaries);
    let paired_matrix_complete = flow_groups
        .iter()
        .all(|group| !group.baseline.is_empty() && !group.optimized.is_empty());

    let kpi = if telemetry_complete && provenance_complete && paired_matrix_complete {
        Some(build_kpi_report(&flow_groups))
    } else {
        None
    };

    let kpi_checks_pass = kpi
        .as_ref()
        .is_some_and(|report| report.checks.iter().all(|check| check.pass));

    let gate_checks = vec![
        GateCheck {
            name: "telemetry_complete".to_string(),
            target: "all run summaries include required KPI telemetry".to_string(),
            pass: telemetry_complete,
        },
        GateCheck {
            name: "provenance_complete".to_string(),
            target: "all decision-grade outputs include valid provenance".to_string(),
            pass: provenance_complete,
        },
        GateCheck {
            name: "paired_matrix_complete".to_string(),
            target:
                "all (page_id, pattern, edit_intent_hash) keys have baseline and optimized runs"
                    .to_string(),
            pass: paired_matrix_complete,
        },
        GateCheck {
            name: "retry_policy".to_string(),
            target: "no run exceeds one scoped retry".to_string(),
            pass: retry_policy_ok,
        },
        GateCheck {
            name: "drift_resolved".to_string(),
            target: "live-vs-stub parity checks are resolved".to_string(),
            pass: !drift.unresolved_material_drift,
        },
        GateCheck {
            name: "scenario_coverage".to_string(),
            target: "required scenario IDs are covered".to_string(),
            pass: scenario_coverage.complete,
        },
        GateCheck {
            name: "safety_gates".to_string(),
            target: "no locked-node, out-of-scope, or table-shape violations".to_string(),
            pass: !safety.safety_failed,
        },
    ];

    let valid_summaries = manifest
        .runs
        .iter()
        .filter_map(|run| summaries.get(&run.run_id))
        .filter(|summary| {
            summary_telemetry_complete(summary) && provenance_matches(summary, provenance)
        })
        .cloned()
        .collect::<Vec<_>>();

    let recommendation = build_recommendation(
        &gate_checks,
        kpi.as_ref(),
        &valid_summaries,
        &flow_groups,
        safety.safety_failed,
        drift.unresolved_material_drift,
    );
    let status = if recommendation.decision == "go" {
        "passed".to_string()
    } else {
        "failed".to_string()
    };

    let _ = kpi_checks_pass;

    Ok(BatchReport {
        total_runs: diagnostics.len(),
        succeeded_runs: diagnostics.len().saturating_sub(failed_runs),
        failed_runs,
        status,
        telemetry_complete,
        retry_policy_ok,
        provenance: provenance.clone(),
        diagnostics,
        artifact_index_path: artifact_index_path.to_string(),
        drift,
        scenario_coverage,
        safety,
        gate_checks,
        kpi,
        recommendation,
    })
}

pub(crate) fn build_artifact_index(
    manifest: &RunManifest,
    provenance: &ProvenanceStamp,
) -> BatchArtifactIndex {
    let observed_scenarios = observed_scenario_ids(manifest);
    let runs = manifest
        .runs
        .iter()
        .map(|run| {
            let state_artifacts = PipelineState::ORDER
                .iter()
                .map(|state| {
                    let state_name = state.as_str().to_string();
                    StateArtifactIndexEntry {
                        state: state_name.clone(),
                        input_path: format!(
                            "artifacts/{}/{state_name}/state_input.json",
                            run.run_id
                        ),
                        output_path: format!(
                            "artifacts/{}/{state_name}/state_output.json",
                            run.run_id
                        ),
                        diagnostics_path: format!(
                            "artifacts/{}/{state_name}/diagnostics.json",
                            run.run_id
                        ),
                    }
                })
                .collect::<Vec<_>>();

            RunArtifactIndexEntry {
                run_id: run.run_id.clone(),
                page_id: run.page_id.clone(),
                pattern: run.pattern.clone(),
                flow: run.flow.clone(),
                edit_intent_hash: run.edit_intent_hash.clone(),
                summary_path: format!("artifacts/{}/summary.json", run.run_id),
                state_artifacts,
            }
        })
        .collect::<Vec<_>>();

    BatchArtifactIndex {
        batch: BatchArtifactMetadata {
            manifest_path: "artifacts/batch/manifest.input.json".to_string(),
            normalized_manifest_path: "artifacts/batch/manifest.normalized.json".to_string(),
            report_path: "artifacts/batch/report.json".to_string(),
            required_scenario_ids: manifest.batch.required_scenario_ids.clone(),
            observed_scenario_ids: observed_scenarios,
            live_smoke: manifest.batch.live_smoke.clone(),
            provenance: provenance.clone(),
        },
        runs,
    }
}

pub(crate) fn classify_run_from_summary(
    run: &ManifestRunEntry,
    summary: Option<&RunSummary>,
    provenance: &ProvenanceStamp,
) -> BatchRunDiagnostic {
    match summary {
        None => BatchRunDiagnostic {
            run_id: run.run_id.clone(),
            page_id: run.page_id.clone(),
            pattern: run.pattern.clone(),
            flow: run.flow.clone(),
            status: "failed".to_string(),
            error_class: Some("io".to_string()),
            error_code: Some("ERR_SUMMARY_MISSING".to_string()),
            message: Some("run summary artifact is missing".to_string()),
        },
        Some(summary) => {
            if let Err(error) = validate_run_summary_telemetry(summary) {
                return BatchRunDiagnostic {
                    run_id: run.run_id.clone(),
                    page_id: run.page_id.clone(),
                    pattern: run.pattern.clone(),
                    flow: run.flow.clone(),
                    status: "failed".to_string(),
                    error_class: Some("telemetry_incomplete".to_string()),
                    error_code: Some("ERR_TELEMETRY_INCOMPLETE".to_string()),
                    message: Some(error.to_string()),
                };
            }

            if !provenance_matches(summary, provenance) {
                return BatchRunDiagnostic {
                    run_id: run.run_id.clone(),
                    page_id: run.page_id.clone(),
                    pattern: run.pattern.clone(),
                    flow: run.flow.clone(),
                    status: "failed".to_string(),
                    error_class: Some("provenance_incomplete".to_string()),
                    error_code: Some("ERR_PROVENANCE_MISMATCH".to_string()),
                    message: Some(
                        "run summary provenance does not match batch provenance".to_string(),
                    ),
                };
            }

            if summary.retry_count > 1 {
                return BatchRunDiagnostic {
                    run_id: run.run_id.clone(),
                    page_id: run.page_id.clone(),
                    pattern: run.pattern.clone(),
                    flow: run.flow.clone(),
                    status: "failed".to_string(),
                    error_class: Some("retry_policy".to_string()),
                    error_code: Some("ERR_CONFLICT_RETRY_EXHAUSTED".to_string()),
                    message: Some("retry-count exceeded one scoped retry maximum".to_string()),
                };
            }

            if summary
                .error_codes
                .iter()
                .any(|code| code == ErrorCode::RuntimeUnmappedHard.as_str())
            {
                return BatchRunDiagnostic {
                    run_id: run.run_id.clone(),
                    page_id: run.page_id.clone(),
                    pattern: run.pattern.clone(),
                    flow: run.flow.clone(),
                    status: "failed".to_string(),
                    error_class: Some("runtime_unmapped_hard".to_string()),
                    error_code: Some(ErrorCode::RuntimeUnmappedHard.to_string()),
                    message: Some("unmapped hard failure from live runtime requires explicit taxonomy mapping".to_string()),
                };
            }

            if !summary.telemetry_complete {
                return BatchRunDiagnostic {
                    run_id: run.run_id.clone(),
                    page_id: run.page_id.clone(),
                    pattern: run.pattern.clone(),
                    flow: run.flow.clone(),
                    status: "failed".to_string(),
                    error_class: Some("telemetry_incomplete".to_string()),
                    error_code: Some("ERR_TELEMETRY_INCOMPLETE".to_string()),
                    message: Some("run summary is invalid for KPI aggregation".to_string()),
                };
            }

            if summary.success {
                return BatchRunDiagnostic {
                    run_id: run.run_id.clone(),
                    page_id: run.page_id.clone(),
                    pattern: run.pattern.clone(),
                    flow: run.flow.clone(),
                    status: "ok".to_string(),
                    error_class: None,
                    error_code: None,
                    message: None,
                };
            }

            BatchRunDiagnostic {
                run_id: run.run_id.clone(),
                page_id: run.page_id.clone(),
                pattern: run.pattern.clone(),
                flow: run.flow.clone(),
                status: "failed".to_string(),
                error_class: Some("pipeline_hard".to_string()),
                error_code: summary.error_codes.first().cloned(),
                message: summary
                    .failure_state
                    .map(|state| format!("pipeline failed in state `{}`", state.as_str())),
            }
        }
    }
}

pub(crate) fn summarize_failure_classes(
    report: &BatchReport,
    runbook_bundle: &RunbookBundle,
) -> Vec<FailureClassSummary> {
    let mut counts = BTreeMap::new();
    for diag in &report.diagnostics {
        if let Some(class) = &diag.error_class {
            *counts.entry(class.clone()).or_insert(0usize) += 1;
        }
    }

    let mut summaries = runbook_bundle
        .sections
        .iter()
        .filter(|section| section.failure_class != "no_active_failures")
        .filter_map(|section| {
            let count = counts.get(&section.failure_class).copied().unwrap_or(0);
            if count > 0 || section.blocks_signoff {
                Some(FailureClassSummary {
                    failure_class: section.failure_class.clone(),
                    severity: section.severity.clone(),
                    count,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for (failure_class, count) in counts {
        if summaries
            .iter()
            .any(|summary| summary.failure_class == failure_class)
        {
            continue;
        }
        summaries.push(FailureClassSummary {
            failure_class,
            severity: "medium".to_string(),
            count,
        });
    }

    summaries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.failure_class.cmp(&right.failure_class))
    });
    summaries
}
