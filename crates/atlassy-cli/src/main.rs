use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use atlassy_confluence::{StubConfluenceClient, StubPage};
use atlassy_contracts::{
    ERR_LOCKED_NODE_MUTATION, ERR_OUT_OF_SCOPE_MUTATION, ERR_TABLE_SHAPE_CHANGE, FLOW_BASELINE,
    FLOW_OPTIMIZED, PATTERN_A, PATTERN_B, PATTERN_C, PipelineState, RunSummary,
};
use atlassy_pipeline::{Orchestrator, RunMode, RunRequest};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

const REQUIRED_SCENARIO_IDS: [&str; 10] = [
    "S-001", "S-002", "S-003", "S-004", "S-005", "S-006", "S-007", "S-008", "S-009", "S-010",
];

type DynError = Box<dyn Error>;

#[derive(Debug, Parser)]
#[command(name = "atlassy")]
#[command(about = "Atlassy CLI for v1 pipeline execution")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Run {
        #[arg(long)]
        request_id: String,
        #[arg(long)]
        page_id: String,
        #[arg(long)]
        edit_intent: String,
        #[arg(long = "scope")]
        scope_selectors: Vec<String>,
        #[arg(long, default_value = ".")]
        artifacts_dir: PathBuf,
        #[arg(long, value_enum, default_value_t = CliMode::NoOp)]
        mode: CliMode,
        #[arg(long)]
        target_path: Option<String>,
        #[arg(long)]
        new_value: Option<String>,
        #[arg(long)]
        force_verify_fail: bool,
    },
    RunBatch {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long, default_value = ".")]
        artifacts_dir: PathBuf,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum CliMode {
    NoOp,
    SimpleScopedUpdate,
    SimpleScopedProseUpdate,
    SimpleScopedTableCellUpdate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RunManifest {
    #[serde(default)]
    batch: BatchManifestMetadata,
    runs: Vec<ManifestRunEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BatchManifestMetadata {
    #[serde(default = "default_required_scenario_ids")]
    required_scenario_ids: Vec<String>,
    #[serde(default)]
    observed_scenario_ids: Vec<String>,
    #[serde(default)]
    live_smoke: DriftStatusInput,
}

impl Default for BatchManifestMetadata {
    fn default() -> Self {
        Self {
            required_scenario_ids: default_required_scenario_ids(),
            observed_scenario_ids: Vec::new(),
            live_smoke: DriftStatusInput::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct DriftStatusInput {
    #[serde(default = "default_true")]
    scoped_fetch_parity: bool,
    #[serde(default = "default_true")]
    publish_conflict_parity: bool,
    #[serde(default = "default_true")]
    error_payload_parity: bool,
}

impl Default for DriftStatusInput {
    fn default() -> Self {
        Self {
            scoped_fetch_parity: true,
            publish_conflict_parity: true,
            error_payload_parity: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ManifestRunEntry {
    run_id: String,
    page_id: String,
    pattern: String,
    flow: String,
    edit_intent: String,
    edit_intent_hash: String,
    #[serde(default)]
    scenario_ids: Vec<String>,
    #[serde(default)]
    scope_selectors: Vec<String>,
    timestamp: String,
    #[serde(default)]
    mode: ManifestMode,
    target_path: Option<String>,
    new_value: Option<String>,
    #[serde(default)]
    force_verify_fail: bool,
    #[serde(default)]
    simulate_conflict_once: bool,
    #[serde(default)]
    simulate_conflict_exhausted: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ManifestMode {
    #[default]
    NoOp,
    SimpleScopedUpdate,
    SimpleScopedProseUpdate,
    SimpleScopedTableCellUpdate,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct BatchRunDiagnostic {
    run_id: String,
    page_id: String,
    pattern: String,
    flow: String,
    status: String,
    error_class: Option<String>,
    error_code: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct BatchReport {
    total_runs: usize,
    succeeded_runs: usize,
    failed_runs: usize,
    status: String,
    telemetry_complete: bool,
    retry_policy_ok: bool,
    diagnostics: Vec<BatchRunDiagnostic>,
    artifact_index_path: String,
    drift: DriftAssessment,
    scenario_coverage: ScenarioCoverageAssessment,
    safety: SafetyAssessment,
    gate_checks: Vec<GateCheck>,
    kpi: Option<KpiReport>,
    recommendation: RecommendationSection,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct BatchArtifactIndex {
    batch: BatchArtifactMetadata,
    runs: Vec<RunArtifactIndexEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct BatchArtifactMetadata {
    manifest_path: String,
    normalized_manifest_path: String,
    report_path: String,
    required_scenario_ids: Vec<String>,
    observed_scenario_ids: Vec<String>,
    live_smoke: DriftStatusInput,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct RunArtifactIndexEntry {
    run_id: String,
    page_id: String,
    pattern: String,
    flow: String,
    edit_intent_hash: String,
    summary_path: String,
    state_artifacts: Vec<StateArtifactIndexEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct StateArtifactIndexEntry {
    state: String,
    input_path: String,
    output_path: String,
    diagnostics_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct DriftAssessment {
    scoped_fetch_parity: bool,
    publish_conflict_parity: bool,
    error_payload_parity: bool,
    unresolved_material_drift: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ScenarioCoverageAssessment {
    required_scenario_ids: Vec<String>,
    observed_scenario_ids: Vec<String>,
    missing_scenario_ids: Vec<String>,
    complete: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct SafetyAssessment {
    locked_node_violation_runs: Vec<String>,
    out_of_scope_violation_runs: Vec<String>,
    table_shape_violation_runs: Vec<String>,
    safety_failed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct GateCheck {
    name: String,
    target: String,
    pass: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct KpiReport {
    global_rollup: KpiRollup,
    pattern_rollups: Vec<KpiRollup>,
    checks: Vec<GateCheck>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct KpiRollup {
    scope: String,
    pair_count: usize,
    metrics: Vec<KpiMetricComparison>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct KpiMetricComparison {
    kpi: String,
    baseline: KpiStats,
    optimized: KpiStats,
    delta_absolute: f64,
    delta_relative: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct KpiStats {
    count: usize,
    median: f64,
    p90: f64,
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct RecommendationSection {
    decision: String,
    rationale: Vec<String>,
    outliers: Vec<OutlierRun>,
    regressions: Vec<RegressionSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct OutlierRun {
    run_id: String,
    kpi: String,
    value: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct RegressionSummary {
    page_id: String,
    pattern: String,
    edit_intent_hash: String,
    kpi: String,
    baseline: f64,
    optimized: f64,
}

#[derive(Debug, Clone)]
struct FlowGroup {
    page_id: String,
    pattern: String,
    edit_intent_hash: String,
    baseline: Vec<RunSummary>,
    optimized: Vec<RunSummary>,
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            request_id,
            page_id,
            edit_intent,
            scope_selectors,
            artifacts_dir,
            mode,
            target_path,
            new_value,
            force_verify_fail,
        } => {
            let run_mode = match mode {
                CliMode::NoOp => RunMode::NoOp,
                CliMode::SimpleScopedUpdate => {
                    let path =
                        target_path.unwrap_or_else(|| "/content/1/content/0/text".to_string());
                    let value =
                        serde_json::json!(new_value.unwrap_or_else(|| "Updated text".to_string()));
                    RunMode::SimpleScopedUpdate {
                        target_path: path,
                        new_value: value,
                    }
                }
                CliMode::SimpleScopedProseUpdate => {
                    let path =
                        target_path.unwrap_or_else(|| "/content/1/content/0/text".to_string());
                    let markdown = new_value.unwrap_or_else(|| "Updated prose body".to_string());
                    RunMode::SimpleScopedProseUpdate {
                        target_path: path,
                        markdown,
                    }
                }
                CliMode::SimpleScopedTableCellUpdate => {
                    let path = target_path.unwrap_or_else(|| {
                        "/content/2/content/0/content/0/content/0/content/0/text".to_string()
                    });
                    let text = new_value.unwrap_or_else(|| "Updated table cell".to_string());
                    RunMode::SimpleScopedTableCellUpdate {
                        target_path: path,
                        text,
                    }
                }
            };

            let mut pages = HashMap::new();
            pages.insert(
                page_id.clone(),
                StubPage {
                    version: 1,
                    adf: demo_page(),
                },
            );

            let mut orchestrator =
                Orchestrator::new(StubConfluenceClient::new(pages), artifacts_dir);
            let request = RunRequest {
                request_id,
                page_id,
                edit_intent_hash: hash_edit_intent(&edit_intent),
                flow: FLOW_OPTIMIZED.to_string(),
                pattern: PATTERN_A.to_string(),
                edit_intent,
                scope_selectors,
                timestamp: "2026-03-06T10:00:00Z".to_string(),
                run_mode,
                force_verify_fail,
            };

            match orchestrator.run(request) {
                Ok(summary) => println!("{}", serde_json::to_string_pretty(&summary)?),
                Err(error) => {
                    eprintln!("pipeline failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        Commands::RunBatch {
            manifest,
            artifacts_dir,
        } => {
            let report = execute_batch_from_manifest_file(&manifest, &artifacts_dir)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}

fn execute_batch_from_manifest_file(
    manifest_path: &Path,
    artifacts_dir: &Path,
) -> Result<BatchReport, DynError> {
    let manifest_text = fs::read_to_string(manifest_path)?;
    let mut run_manifest: RunManifest = serde_json::from_str(&manifest_text)?;
    validate_manifest(&run_manifest)?;
    normalize_manifest(&mut run_manifest);

    let batch_dir = artifacts_dir.join("artifacts").join("batch");
    fs::create_dir_all(&batch_dir)?;
    fs::write(
        batch_dir.join("manifest.normalized.json"),
        serde_json::to_string_pretty(&run_manifest)?,
    )?;

    execute_manifest_runs(&run_manifest, artifacts_dir);

    let artifact_index = build_artifact_index(&run_manifest);
    fs::write(
        batch_dir.join("artifact-index.json"),
        serde_json::to_string_pretty(&artifact_index)?,
    )?;

    let report = rebuild_batch_report_from_artifacts(
        &run_manifest,
        artifacts_dir,
        "artifacts/batch/artifact-index.json",
    )?;

    fs::write(
        batch_dir.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(report)
}

fn execute_manifest_runs(manifest: &RunManifest, artifacts_dir: &Path) {
    for run in &manifest.runs {
        let mut pages = HashMap::new();
        pages.insert(
            run.page_id.clone(),
            StubPage {
                version: 1,
                adf: demo_page(),
            },
        );

        let client = if run.simulate_conflict_exhausted {
            StubConfluenceClient::new(pages).with_always_conflict()
        } else if run.simulate_conflict_once {
            StubConfluenceClient::new(pages).with_conflict_once()
        } else {
            StubConfluenceClient::new(pages)
        };

        let request = RunRequest {
            request_id: run.run_id.clone(),
            page_id: run.page_id.clone(),
            edit_intent: run.edit_intent.clone(),
            edit_intent_hash: run.edit_intent_hash.clone(),
            flow: run.flow.clone(),
            pattern: run.pattern.clone(),
            scope_selectors: run.scope_selectors.clone(),
            timestamp: run.timestamp.clone(),
            run_mode: run_mode_from_manifest(run),
            force_verify_fail: run.force_verify_fail,
        };

        let mut orchestrator = Orchestrator::new(client, artifacts_dir);
        let _ = orchestrator.run(request);
    }
}

fn rebuild_batch_report_from_artifacts(
    manifest: &RunManifest,
    artifacts_dir: &Path,
    artifact_index_path: &str,
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
        .map(|run| classify_run_from_summary(run, summaries.get(&run.run_id)))
        .collect::<Vec<_>>();

    let failed_runs = diagnostics
        .iter()
        .filter(|diag| diag.status == "failed")
        .count();
    let telemetry_complete = manifest.runs.iter().all(|run| {
        summaries
            .get(&run.run_id)
            .is_some_and(|summary| summary.telemetry_complete)
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

    let kpi = if telemetry_complete && paired_matrix_complete {
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
        .filter(|summary| summary.telemetry_complete)
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

fn build_artifact_index(manifest: &RunManifest) -> BatchArtifactIndex {
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
        },
        runs,
    }
}

fn classify_run_from_summary(
    run: &ManifestRunEntry,
    summary: Option<&RunSummary>,
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

fn assess_drift(input: &DriftStatusInput) -> DriftAssessment {
    let unresolved_material_drift =
        !input.scoped_fetch_parity || !input.publish_conflict_parity || !input.error_payload_parity;
    DriftAssessment {
        scoped_fetch_parity: input.scoped_fetch_parity,
        publish_conflict_parity: input.publish_conflict_parity,
        error_payload_parity: input.error_payload_parity,
        unresolved_material_drift,
    }
}

fn assess_scenario_coverage(manifest: &RunManifest) -> ScenarioCoverageAssessment {
    let mut required = manifest.batch.required_scenario_ids.clone();
    required.sort();
    required.dedup();

    let observed = observed_scenario_ids(manifest);
    let observed_set = observed.iter().cloned().collect::<BTreeSet<_>>();
    let missing = required
        .iter()
        .filter(|scenario| !observed_set.contains(*scenario))
        .cloned()
        .collect::<Vec<_>>();

    ScenarioCoverageAssessment {
        required_scenario_ids: required,
        observed_scenario_ids: observed,
        missing_scenario_ids: missing.clone(),
        complete: missing.is_empty(),
    }
}

fn observed_scenario_ids(manifest: &RunManifest) -> Vec<String> {
    let mut observed = manifest.batch.observed_scenario_ids.clone();
    for run in &manifest.runs {
        observed.extend(run.scenario_ids.clone());
    }
    observed.sort();
    observed.dedup();
    observed
}

fn assess_safety(summaries: &BTreeMap<String, RunSummary>) -> SafetyAssessment {
    let mut locked = Vec::new();
    let mut out_of_scope = Vec::new();
    let mut table_shape = Vec::new();

    for (run_id, summary) in summaries {
        if summary
            .error_codes
            .iter()
            .any(|code| code == ERR_LOCKED_NODE_MUTATION)
            || summary.locked_node_mutation
        {
            locked.push(run_id.clone());
        }
        if summary
            .error_codes
            .iter()
            .any(|code| code == ERR_OUT_OF_SCOPE_MUTATION)
        {
            out_of_scope.push(run_id.clone());
        }
        if summary
            .error_codes
            .iter()
            .any(|code| code == ERR_TABLE_SHAPE_CHANGE)
        {
            table_shape.push(run_id.clone());
        }
    }

    locked.sort();
    out_of_scope.sort();
    table_shape.sort();

    SafetyAssessment {
        locked_node_violation_runs: locked.clone(),
        out_of_scope_violation_runs: out_of_scope.clone(),
        table_shape_violation_runs: table_shape.clone(),
        safety_failed: !(locked.is_empty() && out_of_scope.is_empty() && table_shape.is_empty()),
    }
}

fn collect_flow_groups(
    manifest: &RunManifest,
    summaries: &BTreeMap<String, RunSummary>,
) -> Vec<FlowGroup> {
    let mut grouped: BTreeMap<(String, String, String), FlowGroup> = BTreeMap::new();

    for run in &manifest.runs {
        let key = (
            run.page_id.clone(),
            run.pattern.clone(),
            run.edit_intent_hash.clone(),
        );
        let group = grouped.entry(key.clone()).or_insert_with(|| FlowGroup {
            page_id: key.0.clone(),
            pattern: key.1.clone(),
            edit_intent_hash: key.2.clone(),
            baseline: Vec::new(),
            optimized: Vec::new(),
        });

        if let Some(summary) = summaries.get(&run.run_id)
            && summary.telemetry_complete
        {
            if run.flow == FLOW_BASELINE {
                group.baseline.push(summary.clone());
            } else if run.flow == FLOW_OPTIMIZED {
                group.optimized.push(summary.clone());
            }
        }
    }

    grouped.into_values().collect()
}

fn build_kpi_report(flow_groups: &[FlowGroup]) -> KpiReport {
    let global_rollup = build_kpi_rollup("global", flow_groups);
    let pattern_rollups = [PATTERN_A, PATTERN_B, PATTERN_C]
        .into_iter()
        .map(|pattern| {
            let groups = flow_groups
                .iter()
                .filter(|group| group.pattern == pattern)
                .cloned()
                .collect::<Vec<_>>();
            build_kpi_rollup(pattern, &groups)
        })
        .collect::<Vec<_>>();

    let checks = evaluate_kpi_checks(&global_rollup);

    KpiReport {
        global_rollup,
        pattern_rollups,
        checks,
    }
}

fn build_kpi_rollup(scope: &str, flow_groups: &[FlowGroup]) -> KpiRollup {
    let pair_count = flow_groups
        .iter()
        .filter(|group| !group.baseline.is_empty() && !group.optimized.is_empty())
        .count();

    let baseline = flow_groups
        .iter()
        .flat_map(|group| group.baseline.clone())
        .collect::<Vec<_>>();
    let optimized = flow_groups
        .iter()
        .flat_map(|group| group.optimized.clone())
        .collect::<Vec<_>>();

    let metrics = [
        "tokens_per_successful_update",
        "full_page_retrieval_rate",
        "retry_conflict_token_waste",
        "formatting_fidelity_pass_rate",
        "publish_latency",
    ]
    .into_iter()
    .map(|kpi| {
        let baseline_values = kpi_values(&baseline, kpi);
        let optimized_values = kpi_values(&optimized, kpi);
        let baseline_stats = compute_stats(&baseline_values);
        let optimized_stats = compute_stats(&optimized_values);

        let delta_absolute = optimized_stats.median - baseline_stats.median;
        let delta_relative = if baseline_stats.median.abs() < f64::EPSILON {
            0.0
        } else {
            delta_absolute / baseline_stats.median
        };

        KpiMetricComparison {
            kpi: kpi.to_string(),
            baseline: baseline_stats,
            optimized: optimized_stats,
            delta_absolute,
            delta_relative,
        }
    })
    .collect::<Vec<_>>();

    KpiRollup {
        scope: scope.to_string(),
        pair_count,
        metrics,
    }
}

fn kpi_values(summaries: &[RunSummary], kpi: &str) -> Vec<f64> {
    match kpi {
        "tokens_per_successful_update" => summaries
            .iter()
            .filter(|summary| summary.publish_result == "published")
            .map(|summary| summary.total_tokens as f64)
            .collect(),
        "full_page_retrieval_rate" => summaries
            .iter()
            .map(|summary| {
                if summary.scope_resolution_failed || summary.full_page_fetch {
                    100.0
                } else {
                    0.0
                }
            })
            .collect(),
        "retry_conflict_token_waste" => summaries
            .iter()
            .map(|summary| summary.retry_tokens as f64)
            .collect(),
        "formatting_fidelity_pass_rate" => summaries
            .iter()
            .map(|summary| {
                if summary.verify_result == "pass" && !summary.locked_node_mutation {
                    100.0
                } else {
                    0.0
                }
            })
            .collect(),
        "publish_latency" => summaries
            .iter()
            .map(|summary| summary.latency_ms as f64)
            .collect(),
        _ => Vec::new(),
    }
}

fn compute_stats(values: &[f64]) -> KpiStats {
    if values.is_empty() {
        return KpiStats {
            count: 0,
            median: 0.0,
            p90: 0.0,
            min: 0.0,
            max: 0.0,
        };
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));

    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        let right = sorted.len() / 2;
        let left = right - 1;
        (sorted[left] + sorted[right]) / 2.0
    };

    let p90_idx = ((sorted.len() as f64) * 0.9).ceil() as usize;
    let p90 = sorted[p90_idx.saturating_sub(1).min(sorted.len() - 1)];

    KpiStats {
        count: sorted.len(),
        median,
        p90,
        min: *sorted.first().unwrap_or(&0.0),
        max: *sorted.last().unwrap_or(&0.0),
    }
}

fn evaluate_kpi_checks(global: &KpiRollup) -> Vec<GateCheck> {
    let metric = |name: &str| global.metrics.iter().find(|metric| metric.kpi == name);

    let tokens_check = metric("tokens_per_successful_update")
        .map(|metric| metric.delta_relative <= -0.4 && metric.delta_relative >= -0.6)
        .unwrap_or(false);

    let full_page_check = metric("full_page_retrieval_rate")
        .map(|metric| metric.delta_relative <= -0.6 && metric.delta_relative >= -0.8)
        .unwrap_or(false);

    let fidelity_check = metric("formatting_fidelity_pass_rate")
        .map(|metric| metric.optimized.median >= metric.baseline.median)
        .unwrap_or(false);

    let latency_check = metric("publish_latency")
        .map(|metric| {
            metric.optimized.median <= metric.baseline.median
                && metric.optimized.p90 <= metric.baseline.p90
        })
        .unwrap_or(false);

    vec![
        GateCheck {
            name: "tokens_per_successful_update".to_string(),
            target: "40-60% reduction vs baseline median".to_string(),
            pass: tokens_check,
        },
        GateCheck {
            name: "full_page_retrieval_rate".to_string(),
            target: "60-80% reduction vs baseline".to_string(),
            pass: full_page_check,
        },
        GateCheck {
            name: "formatting_fidelity_pass_rate".to_string(),
            target: "non-regressive vs baseline".to_string(),
            pass: fidelity_check,
        },
        GateCheck {
            name: "publish_latency".to_string(),
            target: "non-regressive at median and p90".to_string(),
            pass: latency_check,
        },
    ]
}

fn build_recommendation(
    gate_checks: &[GateCheck],
    kpi: Option<&KpiReport>,
    valid_summaries: &[RunSummary],
    flow_groups: &[FlowGroup],
    safety_failed: bool,
    drift_unresolved: bool,
) -> RecommendationSection {
    let kpi_failed = kpi
        .map(|report| report.checks.iter().any(|check| !check.pass))
        .unwrap_or(true);
    let gate_failed = gate_checks.iter().any(|check| !check.pass);

    let decision = if safety_failed || drift_unresolved {
        "stop"
    } else if gate_failed || kpi_failed {
        "iterate"
    } else {
        "go"
    }
    .to_string();

    let mut rationale = gate_checks
        .iter()
        .filter(|check| !check.pass)
        .map(|check| format!("gate failed: {} ({})", check.name, check.target))
        .collect::<Vec<_>>();
    if let Some(report) = kpi {
        rationale.extend(
            report
                .checks
                .iter()
                .filter(|check| !check.pass)
                .map(|check| format!("kpi target missed: {} ({})", check.name, check.target)),
        );
    }
    rationale.sort();
    rationale.dedup();

    RecommendationSection {
        decision,
        rationale,
        outliers: build_outliers(valid_summaries),
        regressions: build_regressions(flow_groups),
    }
}

fn build_outliers(summaries: &[RunSummary]) -> Vec<OutlierRun> {
    let mut outliers = Vec::new();

    let mut latency = summaries
        .iter()
        .map(|summary| OutlierRun {
            run_id: summary.run_id.clone(),
            kpi: "publish_latency".to_string(),
            value: summary.latency_ms as f64,
        })
        .collect::<Vec<_>>();
    latency.sort_by(|left, right| {
        right
            .value
            .total_cmp(&left.value)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    outliers.extend(latency.into_iter().take(3));

    let mut tokens = summaries
        .iter()
        .map(|summary| OutlierRun {
            run_id: summary.run_id.clone(),
            kpi: "tokens_per_successful_update".to_string(),
            value: summary.total_tokens as f64,
        })
        .collect::<Vec<_>>();
    tokens.sort_by(|left, right| {
        right
            .value
            .total_cmp(&left.value)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    outliers.extend(tokens.into_iter().take(3));

    outliers
}

fn build_regressions(flow_groups: &[FlowGroup]) -> Vec<RegressionSummary> {
    let mut regressions = Vec::new();

    for group in flow_groups {
        if group.baseline.is_empty() || group.optimized.is_empty() {
            continue;
        }

        let baseline_tokens =
            compute_stats(&kpi_values(&group.baseline, "tokens_per_successful_update")).median;
        let optimized_tokens = compute_stats(&kpi_values(
            &group.optimized,
            "tokens_per_successful_update",
        ))
        .median;
        if optimized_tokens > baseline_tokens {
            regressions.push(RegressionSummary {
                page_id: group.page_id.clone(),
                pattern: group.pattern.clone(),
                edit_intent_hash: group.edit_intent_hash.clone(),
                kpi: "tokens_per_successful_update".to_string(),
                baseline: baseline_tokens,
                optimized: optimized_tokens,
            });
        }

        let baseline_latency =
            compute_stats(&kpi_values(&group.baseline, "publish_latency")).median;
        let optimized_latency =
            compute_stats(&kpi_values(&group.optimized, "publish_latency")).median;
        if optimized_latency > baseline_latency {
            regressions.push(RegressionSummary {
                page_id: group.page_id.clone(),
                pattern: group.pattern.clone(),
                edit_intent_hash: group.edit_intent_hash.clone(),
                kpi: "publish_latency".to_string(),
                baseline: baseline_latency,
                optimized: optimized_latency,
            });
        }

        let baseline_full_page =
            compute_stats(&kpi_values(&group.baseline, "full_page_retrieval_rate")).median;
        let optimized_full_page =
            compute_stats(&kpi_values(&group.optimized, "full_page_retrieval_rate")).median;
        if optimized_full_page > baseline_full_page {
            regressions.push(RegressionSummary {
                page_id: group.page_id.clone(),
                pattern: group.pattern.clone(),
                edit_intent_hash: group.edit_intent_hash.clone(),
                kpi: "full_page_retrieval_rate".to_string(),
                baseline: baseline_full_page,
                optimized: optimized_full_page,
            });
        }
    }

    regressions.sort_by(|left, right| {
        (
            left.page_id.as_str(),
            left.pattern.as_str(),
            left.edit_intent_hash.as_str(),
            left.kpi.as_str(),
        )
            .cmp(&(
                right.page_id.as_str(),
                right.pattern.as_str(),
                right.edit_intent_hash.as_str(),
                right.kpi.as_str(),
            ))
    });
    regressions
}

fn normalize_manifest(manifest: &mut RunManifest) {
    manifest.batch.required_scenario_ids.sort();
    manifest.batch.required_scenario_ids.dedup();
    manifest.batch.observed_scenario_ids.sort();
    manifest.batch.observed_scenario_ids.dedup();

    manifest.runs.sort_by(|left, right| {
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

    for run in &mut manifest.runs {
        run.scenario_ids.sort();
        run.scenario_ids.dedup();
    }
}

fn run_mode_from_manifest(entry: &ManifestRunEntry) -> RunMode {
    match entry.mode {
        ManifestMode::NoOp => RunMode::NoOp,
        ManifestMode::SimpleScopedUpdate => RunMode::SimpleScopedUpdate {
            target_path: entry
                .target_path
                .clone()
                .unwrap_or_else(|| "/content/1/content/0/text".to_string()),
            new_value: serde_json::json!(
                entry
                    .new_value
                    .clone()
                    .unwrap_or_else(|| "Updated text".to_string())
            ),
        },
        ManifestMode::SimpleScopedProseUpdate => RunMode::SimpleScopedProseUpdate {
            target_path: entry
                .target_path
                .clone()
                .unwrap_or_else(|| "/content/1/content/0/text".to_string()),
            markdown: entry
                .new_value
                .clone()
                .unwrap_or_else(|| "Updated prose body".to_string()),
        },
        ManifestMode::SimpleScopedTableCellUpdate => RunMode::SimpleScopedTableCellUpdate {
            target_path: entry.target_path.clone().unwrap_or_else(|| {
                "/content/2/content/0/content/0/content/0/content/0/text".to_string()
            }),
            text: entry
                .new_value
                .clone()
                .unwrap_or_else(|| "Updated table cell".to_string()),
        },
    }
}

fn load_run_summary(artifacts_dir: &Path, run_id: &str) -> Result<Option<RunSummary>, DynError> {
    let summary_path = artifacts_dir
        .join("artifacts")
        .join(run_id)
        .join("summary.json");
    if !summary_path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(summary_path)?;
    let summary = serde_json::from_str::<RunSummary>(&text)?;
    Ok(Some(summary))
}

fn validate_manifest(manifest: &RunManifest) -> Result<(), DynError> {
    if manifest.runs.is_empty() {
        return Err("manifest must contain at least one run".into());
    }

    let mut run_ids = BTreeSet::new();
    let mut pair_map: BTreeMap<(String, String, String), BTreeSet<String>> = BTreeMap::new();

    for run in &manifest.runs {
        for (field, value) in [
            ("run_id", run.run_id.trim()),
            ("page_id", run.page_id.trim()),
            ("pattern", run.pattern.trim()),
            ("flow", run.flow.trim()),
            ("edit_intent", run.edit_intent.trim()),
            ("edit_intent_hash", run.edit_intent_hash.trim()),
            ("timestamp", run.timestamp.trim()),
        ] {
            if value.is_empty() {
                return Err(format!("manifest run missing required field `{field}`").into());
            }
        }

        if !run_ids.insert(run.run_id.clone()) {
            return Err(format!("duplicate run_id found: {}", run.run_id).into());
        }

        if !matches!(run.flow.as_str(), FLOW_BASELINE | FLOW_OPTIMIZED) {
            return Err(format!("invalid flow `{}`: expected baseline|optimized", run.flow).into());
        }
        if !matches!(run.pattern.as_str(), PATTERN_A | PATTERN_B | PATTERN_C) {
            return Err(format!("invalid pattern `{}`: expected A|B|C", run.pattern).into());
        }

        pair_map
            .entry((
                run.page_id.clone(),
                run.pattern.clone(),
                run.edit_intent_hash.clone(),
            ))
            .or_default()
            .insert(run.flow.clone());
    }

    for (key, flows) in pair_map {
        if !flows.contains(FLOW_BASELINE) || !flows.contains(FLOW_OPTIMIZED) {
            return Err(format!(
                "unmatched pair for key (page_id={}, pattern={}, edit_intent_hash={})",
                key.0, key.1, key.2
            )
            .into());
        }
    }

    Ok(())
}

fn default_required_scenario_ids() -> Vec<String> {
    REQUIRED_SCENARIO_IDS
        .iter()
        .map(|scenario| scenario.to_string())
        .collect()
}

fn default_true() -> bool {
    true
}

fn hash_edit_intent(edit_intent: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    edit_intent.hash(&mut hasher);
    format!("h{:016x}", hasher.finish())
}

fn demo_page() -> serde_json::Value {
    serde_json::json!({
      "type": "doc",
      "version": 1,
      "content": [
        {
          "type": "heading",
          "attrs": {"level": 2, "id": "intro-heading"},
          "content": [{"type": "text", "text": "Overview"}]
        },
        {
          "type": "paragraph",
          "attrs": {"id": "intro-paragraph"},
          "content": [{"type": "text", "text": "Initial paragraph"}]
        },
        {
          "type": "table",
          "content": [
            {
              "type": "tableRow",
              "content": [
                {
                  "type": "tableCell",
                  "content": [
                    {
                      "type": "paragraph",
                      "content": [{"type": "text", "text": "Initial table cell"}]
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn complete_matrix_generates_aggregate_report() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let report = execute_batch_from_manifest_file(
            &fixture_path("batch_complete_manifest.json"),
            temp.path(),
        )
        .expect("batch run should succeed");

        assert_eq!(report.total_runs, 6);
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
                .any(|diag| diag.error_class.as_deref() == Some("telemetry_incomplete"))
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
            diag.error_class.as_deref() == Some("retry_policy")
                && diag.error_code.as_deref() == Some("ERR_CONFLICT_RETRY_EXHAUSTED")
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

        assert_eq!(report, rebuilt);
        assert_eq!(stored, rebuilt);
    }
}
