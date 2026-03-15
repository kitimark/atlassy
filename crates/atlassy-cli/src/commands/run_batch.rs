use std::collections::HashMap;
use std::fs;
use std::path::Path;

use atlassy_confluence::{LiveConfluenceClient, StubConfluenceClient, StubPage};
use atlassy_contracts::{ProvenanceStamp, RUNTIME_LIVE, RUNTIME_STUB};
use atlassy_pipeline::{Orchestrator, RunRequest};

use crate::batch::{build_artifact_index, rebuild_batch_report_from_artifacts};
use crate::manifest::{normalize_manifest, run_mode_from_manifest, validate_manifest};
use crate::provenance::collect_provenance;
use crate::{demo_page, empty_page, BatchReport, DynError, RunManifest};

pub fn execute_batch_from_manifest_file(
    manifest_path: &Path,
    artifacts_dir: &Path,
) -> Result<BatchReport, DynError> {
    execute_batch_from_manifest_file_with_backend(manifest_path, artifacts_dir, RUNTIME_STUB)
}

pub fn execute_batch_from_manifest_file_with_backend(
    manifest_path: &Path,
    artifacts_dir: &Path,
    runtime_mode: &str,
) -> Result<BatchReport, DynError> {
    let manifest_text = fs::read_to_string(manifest_path)?;
    let mut run_manifest: RunManifest = serde_json::from_str(&manifest_text)?;
    validate_manifest(&run_manifest)?;
    normalize_manifest(&mut run_manifest);

    let provenance = collect_provenance(runtime_mode)?;
    run_manifest.batch.runtime_mode = runtime_mode.to_string();

    let batch_dir = artifacts_dir.join("artifacts").join("batch");
    fs::create_dir_all(&batch_dir)?;
    fs::write(
        batch_dir.join("manifest.input.json"),
        serde_json::to_string_pretty(&run_manifest)?,
    )?;
    fs::write(
        batch_dir.join("manifest.normalized.json"),
        serde_json::to_string_pretty(&run_manifest)?,
    )?;

    execute_manifest_runs(&run_manifest, artifacts_dir, runtime_mode, &provenance)?;

    let artifact_index = build_artifact_index(&run_manifest, &provenance);
    fs::write(
        batch_dir.join("artifact-index.json"),
        serde_json::to_string_pretty(&artifact_index)?,
    )?;

    let report = rebuild_batch_report_from_artifacts(
        &run_manifest,
        artifacts_dir,
        "artifacts/batch/artifact-index.json",
        &provenance,
    )?;

    fs::write(
        batch_dir.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(report)
}

pub(crate) fn execute_manifest_runs(
    manifest: &RunManifest,
    artifacts_dir: &Path,
    runtime_mode: &str,
    provenance: &ProvenanceStamp,
) -> Result<(), DynError> {
    for run in &manifest.runs {
        let request = RunRequest {
            request_id: run.run_id.clone(),
            page_id: run.page_id.clone(),
            edit_intent: run.edit_intent.clone(),
            edit_intent_hash: run.edit_intent_hash.clone(),
            flow: run.flow.clone(),
            pattern: run.pattern.clone(),
            scope_selectors: run.scope_selectors.clone(),
            timestamp: run.timestamp.clone(),
            provenance: provenance.clone(),
            run_mode: run_mode_from_manifest(run),
            target_index: run
                .target_index
                .map(|index| index as usize)
                .unwrap_or_default(),
            block_ops: run.block_ops.clone(),
            force_verify_fail: run.force_verify_fail,
            bootstrap_empty_page: run.bootstrap_empty_page.unwrap_or(false),
        };

        match runtime_mode {
            RUNTIME_STUB => {
                let page_adf = if run.simulate_empty_page {
                    empty_page()
                } else {
                    demo_page()
                };
                let mut pages = HashMap::new();
                pages.insert(
                    run.page_id.clone(),
                    StubPage {
                        version: 1,
                        adf: page_adf,
                    },
                );
                let client = if run.simulate_conflict_exhausted {
                    StubConfluenceClient::new(pages).with_always_conflict()
                } else if run.simulate_conflict_once {
                    StubConfluenceClient::new(pages).with_conflict_once()
                } else {
                    StubConfluenceClient::new(pages)
                };
                let mut orchestrator = Orchestrator::new(client, artifacts_dir);
                let _ = orchestrator.run(request);
            }
            RUNTIME_LIVE => {
                let client = LiveConfluenceClient::from_env()?;
                let mut orchestrator = Orchestrator::new(client, artifacts_dir);
                let _ = orchestrator.run(request);
            }
            _ => {
                return Err(format!("invalid runtime mode `{runtime_mode}`").into());
            }
        }
    }

    Ok(())
}
