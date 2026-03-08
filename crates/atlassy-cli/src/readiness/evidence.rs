use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use atlassy_contracts::{
    PipelineState, RunSummary, validate_provenance_stamp, validate_run_summary_telemetry,
};

use crate::io::{load_required_json, load_run_summary};
use crate::manifest::normalize_manifest;
use crate::provenance::provenance_matches;
use crate::{BatchArtifactIndex, BatchReport, DynError, ReadinessEvidence, RunManifest};

pub(crate) fn load_readiness_evidence(artifacts_dir: &Path) -> Result<ReadinessEvidence, DynError> {
    let batch_dir = artifacts_dir.join("artifacts").join("batch");
    let manifest_path = batch_dir.join("manifest.normalized.json");
    let artifact_index_path = batch_dir.join("artifact-index.json");
    let report_path = batch_dir.join("report.json");

    let mut manifest: RunManifest = load_required_json(&manifest_path)?;
    normalize_manifest(&mut manifest);

    let mut artifact_index: BatchArtifactIndex = load_required_json(&artifact_index_path)?;
    canonicalize_artifact_index(&mut artifact_index);

    let mut report: BatchReport = load_required_json(&report_path)?;
    report.diagnostics.sort_by(|left, right| {
        (
            left.page_id.as_str(),
            left.pattern.as_str(),
            left.flow.as_str(),
            left.run_id.as_str(),
        )
            .cmp(&(
                right.page_id.as_str(),
                right.pattern.as_str(),
                right.flow.as_str(),
                right.run_id.as_str(),
            ))
    });

    let mut summaries = BTreeMap::new();
    for run in &manifest.runs {
        let summary = load_run_summary(artifacts_dir, &run.run_id)?.ok_or_else(|| {
            format!(
                "missing readiness evidence: artifacts/{}/summary.json",
                run.run_id
            )
        })?;
        summaries.insert(run.run_id.clone(), summary);
    }

    validate_readiness_evidence(&manifest, &artifact_index, &report, &summaries)?;

    let mut source_artifacts = vec![
        "artifacts/batch/manifest.normalized.json".to_string(),
        "artifacts/batch/artifact-index.json".to_string(),
        "artifacts/batch/report.json".to_string(),
    ];
    source_artifacts.extend(
        manifest
            .runs
            .iter()
            .map(|run| format!("artifacts/{}/summary.json", run.run_id)),
    );
    source_artifacts.sort();

    Ok(ReadinessEvidence {
        manifest,
        provenance: report.provenance.clone(),
        report,
        summaries,
        source_artifacts,
    })
}

fn canonicalize_artifact_index(index: &mut BatchArtifactIndex) {
    index.runs.sort_by(|left, right| {
        (
            left.page_id.as_str(),
            left.pattern.as_str(),
            left.edit_intent_hash.as_str(),
            left.flow.as_str(),
            left.run_id.as_str(),
        )
            .cmp(&(
                right.page_id.as_str(),
                right.pattern.as_str(),
                right.edit_intent_hash.as_str(),
                right.flow.as_str(),
                right.run_id.as_str(),
            ))
    });

    for run in &mut index.runs {
        run.state_artifacts.sort_by(|left, right| {
            PipelineState::ORDER
                .iter()
                .position(|state| state.as_str() == left.state)
                .unwrap_or(usize::MAX)
                .cmp(
                    &PipelineState::ORDER
                        .iter()
                        .position(|state| state.as_str() == right.state)
                        .unwrap_or(usize::MAX),
                )
                .then_with(|| left.state.cmp(&right.state))
        });
    }
}

fn validate_readiness_evidence(
    manifest: &RunManifest,
    artifact_index: &BatchArtifactIndex,
    report: &BatchReport,
    summaries: &BTreeMap<String, RunSummary>,
) -> Result<(), DynError> {
    if manifest.runs.is_empty() {
        return Err("missing readiness evidence: normalized manifest has no runs".into());
    }

    if report.total_runs != manifest.runs.len() {
        return Err(format!(
            "missing readiness evidence: report total_runs={} does not match manifest runs={}",
            report.total_runs,
            manifest.runs.len()
        )
        .into());
    }

    let artifact_run_ids = artifact_index
        .runs
        .iter()
        .map(|run| run.run_id.clone())
        .collect::<BTreeSet<_>>();
    let manifest_run_ids = manifest
        .runs
        .iter()
        .map(|run| run.run_id.clone())
        .collect::<BTreeSet<_>>();

    if artifact_run_ids != manifest_run_ids {
        return Err(
            "missing readiness evidence: artifact index runs do not match manifest runs".into(),
        );
    }

    let summary_run_ids = summaries.keys().cloned().collect::<BTreeSet<_>>();
    if summary_run_ids != manifest_run_ids {
        return Err("missing readiness evidence: summary artifacts do not cover all runs".into());
    }

    validate_provenance_stamp(&report.provenance)?;
    validate_provenance_stamp(&artifact_index.batch.provenance)?;
    if report.provenance != artifact_index.batch.provenance {
        return Err(
            "missing readiness evidence: report and artifact-index provenance do not match".into(),
        );
    }

    for (run_id, summary) in summaries {
        let early_failure = summary.failure_state.is_some()
            && summary.failure_state != Some(PipelineState::Verify)
            && summary.failure_state != Some(PipelineState::Publish);
        if !early_failure {
            validate_run_summary_telemetry(summary)?;
        }
        if !provenance_matches(summary, &report.provenance) {
            return Err(format!(
                "missing readiness evidence: summary provenance mismatch for run {}",
                run_id
            )
            .into());
        }
    }

    Ok(())
}
