use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use atlassy_confluence::{
    ConfluenceClient, ConfluenceError, LiveConfluenceClient, StubConfluenceClient, StubPage,
};
use atlassy_contracts::{
    ERR_BOOTSTRAP_INVALID_STATE, ERR_BOOTSTRAP_REQUIRED, ERR_LOCKED_NODE_MUTATION,
    ERR_OUT_OF_SCOPE_MUTATION, ERR_RUNTIME_BACKEND, ERR_RUNTIME_UNMAPPED_HARD,
    ERR_TABLE_SHAPE_CHANGE, FLOW_BASELINE, FLOW_OPTIMIZED, PATTERN_A, PATTERN_B, PATTERN_C,
    PIPELINE_VERSION, PipelineState, ProvenanceStamp, RUNTIME_LIVE, RUNTIME_STUB, RunSummary,
    validate_provenance_stamp, validate_run_summary_telemetry,
};
use atlassy_pipeline::{Orchestrator, PipelineError, RunMode, RunRequest};
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
        #[arg(long)]
        bootstrap_empty_page: bool,
        #[arg(long, value_enum, default_value_t = RuntimeBackend::Stub)]
        runtime_backend: RuntimeBackend,
    },
    RunBatch {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long, default_value = ".")]
        artifacts_dir: PathBuf,
        #[arg(long, value_enum, default_value_t = RuntimeBackend::Stub)]
        runtime_backend: RuntimeBackend,
    },
    RunReadiness {
        #[arg(long, default_value = ".")]
        artifacts_dir: PathBuf,
        #[arg(long)]
        verify_replay: bool,
    },
    CreateSubpage {
        #[arg(long)]
        parent_page_id: String,
        #[arg(long)]
        space_key: String,
        #[arg(long)]
        title: String,
        #[arg(long, value_enum, default_value_t = RuntimeBackend::Stub)]
        runtime_backend: RuntimeBackend,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum CliMode {
    NoOp,
    SimpleScopedUpdate,
    SimpleScopedProseUpdate,
    SimpleScopedTableCellUpdate,
}

#[derive(Debug, Clone, ValueEnum)]
enum RuntimeBackend {
    Stub,
    Live,
}

impl RuntimeBackend {
    fn as_str(&self) -> &'static str {
        match self {
            RuntimeBackend::Stub => RUNTIME_STUB,
            RuntimeBackend::Live => RUNTIME_LIVE,
        }
    }
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
    #[serde(default = "default_runtime_mode")]
    runtime_mode: String,
    #[serde(default)]
    lifecycle_create_subpage_validated: bool,
}

impl Default for BatchManifestMetadata {
    fn default() -> Self {
        Self {
            required_scenario_ids: default_required_scenario_ids(),
            observed_scenario_ids: Vec::new(),
            live_smoke: DriftStatusInput::default(),
            runtime_mode: default_runtime_mode(),
            lifecycle_create_subpage_validated: false,
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
    bootstrap_empty_page: Option<bool>,
    #[serde(default)]
    simulate_empty_page: bool,
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
    provenance: ProvenanceStamp,
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
    provenance: ProvenanceStamp,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ReadinessGateResult {
    gate_id: String,
    gate_name: String,
    target: String,
    pass: bool,
    mandatory: bool,
    owner_role: String,
    blocking_reason: Option<String>,
    evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ReadinessChecklist {
    schema_version: String,
    generated_ts: String,
    owner_roles: ReadinessOwnerRoles,
    source_artifacts: Vec<String>,
    provenance: ProvenanceStamp,
    gates: Vec<ReadinessGateResult>,
    blocked: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ReadinessOwnerRoles {
    product_owner: String,
    engineering_owner: String,
    qa_owner: String,
    data_metrics_owner: String,
    release_reviewer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct RunbookSection {
    failure_class: String,
    severity: String,
    primary_owner_role: String,
    escalation_owner_role: String,
    escalation_trigger: String,
    triage_steps: Vec<String>,
    evidence_checks: Vec<String>,
    fallback: bool,
    blocks_signoff: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct RunbookBundle {
    schema_version: String,
    generated_ts: String,
    provenance: ProvenanceStamp,
    sections: Vec<RunbookSection>,
    blocked: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct RiskStatusDelta {
    risk_id: String,
    title: String,
    priority: String,
    previous_status: String,
    current_status: String,
    reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct FailureClassSummary {
    failure_class: String,
    severity: String,
    count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct DecisionPacket {
    schema_version: String,
    generated_ts: String,
    recommendation: String,
    provenance: ProvenanceStamp,
    blocking_condition: Option<String>,
    rationale: Vec<String>,
    gate_outcomes: Vec<ReadinessGateResult>,
    kpi_summary: Option<KpiReport>,
    risk_status_deltas: Vec<RiskStatusDelta>,
    top_failure_classes: Vec<FailureClassSummary>,
    checklist_path: String,
    runbook_path: String,
    report_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ReadinessOutputs {
    checklist: ReadinessChecklist,
    runbook_bundle: RunbookBundle,
    decision_packet: DecisionPacket,
}

#[derive(Debug, Clone)]
struct ReadinessEvidence {
    manifest: RunManifest,
    provenance: ProvenanceStamp,
    report: BatchReport,
    summaries: BTreeMap<String, RunSummary>,
    source_artifacts: Vec<String>,
}

#[derive(Debug, Clone)]
struct FlowGroup {
    page_id: String,
    pattern: String,
    edit_intent_hash: String,
    baseline: Vec<RunSummary>,
    optimized: Vec<RunSummary>,
}

fn main() -> Result<(), DynError> {
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
            bootstrap_empty_page,
            runtime_backend,
        } => {
            let provenance = collect_provenance(runtime_backend.as_str())?;
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

            let request = RunRequest {
                request_id,
                page_id,
                edit_intent_hash: hash_edit_intent(&edit_intent),
                flow: FLOW_OPTIMIZED.to_string(),
                pattern: PATTERN_A.to_string(),
                edit_intent,
                scope_selectors,
                timestamp: "2026-03-06T10:00:00Z".to_string(),
                provenance,
                run_mode,
                force_verify_fail,
                bootstrap_empty_page,
            };

            run_single_request(request, artifacts_dir, runtime_backend)?;
        }
        Commands::RunBatch {
            manifest,
            artifacts_dir,
            runtime_backend,
        } => {
            let report = execute_batch_from_manifest_file_with_backend(
                &manifest,
                &artifacts_dir,
                runtime_backend,
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::RunReadiness {
            artifacts_dir,
            verify_replay,
        } => {
            let readiness = generate_readiness_outputs_from_artifacts(&artifacts_dir)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&readiness.decision_packet)?
            );
            if verify_replay {
                verify_decision_packet_replay(&artifacts_dir)?;
                println!("readiness replay verification passed");
            }
            ensure_readiness_unblocked(&readiness.decision_packet)?;
        }
        Commands::CreateSubpage {
            parent_page_id,
            space_key,
            title,
            runtime_backend,
        } => {
            let result = match runtime_backend {
                RuntimeBackend::Stub => {
                    let mut pages = HashMap::new();
                    pages.insert(
                        parent_page_id.clone(),
                        StubPage {
                            version: 1,
                            adf: serde_json::json!({"type": "doc", "version": 1, "content": []}),
                        },
                    );
                    let mut client = StubConfluenceClient::new(pages);
                    client
                        .create_page(&title, &parent_page_id, &space_key)
                        .map_err(|error| format!("{error}"))
                }
                RuntimeBackend::Live => {
                    let mut client = LiveConfluenceClient::from_env()
                        .map_err(|error| format!("live runtime startup failure: {error}"))?;
                    client
                        .create_page(&title, &parent_page_id, &space_key)
                        .map_err(|error| format!("{error}"))
                }
            };

            match result {
                Ok(response) => {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                }
                Err(error) => {
                    eprintln!("create-subpage failed: {error}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn map_live_startup_error(error: ConfluenceError) -> PipelineError {
    PipelineError::Hard {
        state: PipelineState::Fetch,
        code: ERR_RUNTIME_BACKEND.to_string(),
        message: format!("live runtime startup failure: {error}"),
    }
}

fn run_single_request(
    request: RunRequest,
    artifacts_dir: PathBuf,
    runtime_backend: RuntimeBackend,
) -> Result<(), DynError> {
    match runtime_backend {
        RuntimeBackend::Stub => {
            let mut pages = HashMap::new();
            pages.insert(
                request.page_id.clone(),
                StubPage {
                    version: 1,
                    adf: demo_page(),
                },
            );

            let mut orchestrator =
                Orchestrator::new(StubConfluenceClient::new(pages), artifacts_dir);
            match orchestrator.run(request) {
                Ok(summary) => println!("{}", serde_json::to_string_pretty(&summary)?),
                Err(error) => {
                    eprintln!("pipeline failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        RuntimeBackend::Live => {
            let client = match LiveConfluenceClient::from_env() {
                Ok(client) => client,
                Err(error) => {
                    eprintln!("pipeline failed: {}", map_live_startup_error(error));
                    std::process::exit(1);
                }
            };
            let mut orchestrator = Orchestrator::new(client, artifacts_dir);
            match orchestrator.run(request) {
                Ok(summary) => println!("{}", serde_json::to_string_pretty(&summary)?),
                Err(error) => {
                    eprintln!("pipeline failed: {error}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
fn execute_batch_from_manifest_file(
    manifest_path: &Path,
    artifacts_dir: &Path,
) -> Result<BatchReport, DynError> {
    execute_batch_from_manifest_file_with_backend(
        manifest_path,
        artifacts_dir,
        RuntimeBackend::Stub,
    )
}

fn execute_batch_from_manifest_file_with_backend(
    manifest_path: &Path,
    artifacts_dir: &Path,
    runtime_backend: RuntimeBackend,
) -> Result<BatchReport, DynError> {
    let manifest_text = fs::read_to_string(manifest_path)?;
    let mut run_manifest: RunManifest = serde_json::from_str(&manifest_text)?;
    validate_manifest(&run_manifest)?;
    normalize_manifest(&mut run_manifest);

    let provenance = collect_provenance(runtime_backend.as_str())?;
    validate_provenance_stamp(&provenance)?;
    run_manifest.batch.runtime_mode = runtime_backend.as_str().to_string();

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

    execute_manifest_runs(&run_manifest, artifacts_dir, runtime_backend, &provenance)?;

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

fn execute_manifest_runs(
    manifest: &RunManifest,
    artifacts_dir: &Path,
    runtime_backend: RuntimeBackend,
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
            force_verify_fail: run.force_verify_fail,
            bootstrap_empty_page: run.bootstrap_empty_page.unwrap_or(false),
        };

        match runtime_backend {
            RuntimeBackend::Stub => {
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
            RuntimeBackend::Live => {
                let client = LiveConfluenceClient::from_env()?;
                let mut orchestrator = Orchestrator::new(client, artifacts_dir);
                let _ = orchestrator.run(request);
            }
        }
    }

    Ok(())
}

fn generate_readiness_outputs_from_artifacts(
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

    persist_readiness_outputs(artifacts_dir, &outputs)?;
    Ok(outputs)
}

fn load_readiness_evidence(artifacts_dir: &Path) -> Result<ReadinessEvidence, DynError> {
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

fn load_required_json<T>(path: &Path) -> Result<T, DynError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if !path.exists() {
        return Err(format!(
            "missing readiness evidence: {} (run `atlassy run-batch --manifest <file>` first)",
            path.display()
        )
        .into());
    }

    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
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
        // Early-failure runs (e.g., bootstrap failures at fetch) may not have
        // complete verify/publish telemetry — skip strict validation for those.
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

fn evaluate_readiness_gates(evidence: &ReadinessEvidence) -> ReadinessChecklist {
    let generated_ts = deterministic_generated_ts(&evidence.manifest);
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

    // Gate 7: lifecycle enablement validation
    // Checks batch summaries for lifecycle evidence: bootstrap-required failure,
    // bootstrap success, bootstrap-on-non-empty failure, and create-subpage marker.
    let has_bootstrap_required_failure = evidence.summaries.values().any(|s| {
        s.empty_page_detected
            && !s.success
            && s.error_codes.iter().any(|c| c == ERR_BOOTSTRAP_REQUIRED)
    });
    let has_bootstrap_success = evidence
        .summaries
        .values()
        .any(|s| s.bootstrap_applied && s.success);
    let has_bootstrap_invalid_state = evidence.summaries.values().any(|s| {
        !s.empty_page_detected
            && !s.success
            && s.error_codes
                .iter()
                .any(|c| c == ERR_BOOTSTRAP_INVALID_STATE)
    });
    let has_create_subpage_evidence = evidence.manifest.batch.lifecycle_create_subpage_validated;
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
            vec![
                "artifacts/batch/manifest.normalized.json".to_string(),
                "artifacts/batch/report.json".to_string(),
            ],
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

fn build_operator_runbooks(report: &BatchReport, checklist: &ReadinessChecklist) -> RunbookBundle {
    let mut sections = Vec::new();

    if report.diagnostics.iter().any(|diag| {
        diag.error_class.as_deref() == Some("pipeline_hard")
            && diag
                .message
                .as_deref()
                .is_some_and(|message| message.contains("`verify`"))
    }) {
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

    if report
        .diagnostics
        .iter()
        .any(|diag| diag.error_class.as_deref() == Some("retry_policy"))
    {
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

    if report
        .diagnostics
        .iter()
        .any(|diag| diag.error_class.as_deref() == Some("telemetry_incomplete"))
    {
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

    let known_classes = sections
        .iter()
        .map(|section| section.failure_class.clone())
        .collect::<BTreeSet<_>>();

    for diag in &report.diagnostics {
        if let Some(class) = &diag.error_class {
            let mapped = matches!(
                class.as_str(),
                "pipeline_hard" | "retry_policy" | "telemetry_incomplete"
            ) || (class == "pipeline_hard"
                && diag
                    .message
                    .as_deref()
                    .is_some_and(|message| message.contains("`verify`")));

            if !mapped {
                let fallback_name = format!("unknown:{class}");
                if !known_classes.contains(&fallback_name)
                    && !sections
                        .iter()
                        .any(|section| section.failure_class == fallback_name)
                {
                    sections.push(RunbookSection {
                        failure_class: fallback_name,
                        severity: "high".to_string(),
                        primary_owner_role: "engineering_owner".to_string(),
                        escalation_owner_role: "release_reviewer".to_string(),
                        escalation_trigger: "unmapped failure class observed in diagnostics"
                            .to_string(),
                        triage_steps: vec![
                            "route the failure to manual review".to_string(),
                            "define deterministic runbook mapping before next sign-off".to_string(),
                        ],
                        evidence_checks: vec!["artifacts/batch/report.json".to_string()],
                        fallback: true,
                        blocks_signoff: true,
                    });
                }
            }
        }
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

fn build_risk_status_deltas(report: &BatchReport) -> Vec<RiskStatusDelta> {
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

fn assemble_decision_packet(
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

fn summarize_failure_classes(
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

fn persist_readiness_outputs(
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

fn rebuild_decision_packet_from_artifacts(
    artifacts_dir: &Path,
) -> Result<DecisionPacket, DynError> {
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

fn verify_decision_packet_replay(artifacts_dir: &Path) -> Result<(), DynError> {
    let stored_path = artifacts_dir
        .join("artifacts")
        .join("batch")
        .join("decision.packet.json");
    let stored: DecisionPacket = load_required_json(&stored_path)?;
    let rebuilt = rebuild_decision_packet_from_artifacts(artifacts_dir)?;
    if stored != rebuilt {
        return Err(
            "readiness replay mismatch: rebuilt decision packet diverges from stored output"
                .to_string()
                .into(),
        );
    }
    Ok(())
}

fn ensure_readiness_unblocked(decision_packet: &DecisionPacket) -> Result<(), DynError> {
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

fn deterministic_generated_ts(manifest: &RunManifest) -> String {
    manifest
        .runs
        .iter()
        .map(|run| run.timestamp.as_str())
        .max()
        .unwrap_or("1970-01-01T00:00:00Z")
        .to_string()
}

fn rebuild_batch_report_from_artifacts(
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
            // Runs that failed before reaching verify/publish (e.g., bootstrap
            // failures at fetch) are exempt from telemetry completeness checks.
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

fn build_artifact_index(
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

fn classify_run_from_summary(
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
                .any(|code| code == ERR_RUNTIME_UNMAPPED_HARD)
            {
                return BatchRunDiagnostic {
                    run_id: run.run_id.clone(),
                    page_id: run.page_id.clone(),
                    pattern: run.pattern.clone(),
                    flow: run.flow.clone(),
                    status: "failed".to_string(),
                    error_class: Some("runtime_unmapped_hard".to_string()),
                    error_code: Some(ERR_RUNTIME_UNMAPPED_HARD.to_string()),
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

    grouped
        .into_values()
        .filter(|group| !group.baseline.is_empty() || !group.optimized.is_empty())
        .collect()
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
        "context_reduction_ratio",
        "scoped_section_tokens",
        "edit_success_rate",
        "structural_preservation",
        "conflict_rate",
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
            delta_absolute: normalize_metric_value(delta_absolute),
            delta_relative: normalize_metric_value(delta_relative),
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
        "context_reduction_ratio" => summaries
            .iter()
            .map(|summary| summary.context_reduction_ratio * 100.0)
            .collect(),
        "scoped_section_tokens" => summaries
            .iter()
            .map(|summary| summary.scoped_adf_bytes as f64 / 4.0)
            .collect(),
        "edit_success_rate" => {
            if summaries.is_empty() {
                Vec::new()
            } else {
                let successful = summaries
                    .iter()
                    .filter(|summary| summary.publish_result == "published")
                    .count() as f64;
                vec![(successful / summaries.len() as f64) * 100.0]
            }
        }
        "structural_preservation" => {
            if summaries.is_empty() {
                Vec::new()
            } else {
                let preserved = summaries
                    .iter()
                    .filter(|summary| {
                        summary.verify_result == "pass"
                            && !summary.locked_node_mutation
                            && !summary.out_of_scope_mutation
                    })
                    .count() as f64;
                vec![(preserved / summaries.len() as f64) * 100.0]
            }
        }
        "conflict_rate" => {
            if summaries.is_empty() {
                Vec::new()
            } else {
                let conflicts = summaries
                    .iter()
                    .filter(|summary| summary.retry_count > 0)
                    .count() as f64;
                vec![(conflicts / summaries.len() as f64) * 100.0]
            }
        }
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

fn normalize_metric_value(value: f64) -> f64 {
    const SCALE: f64 = 1_000_000_000_000.0;
    (value * SCALE).round() / SCALE
}

fn evaluate_kpi_checks(global: &KpiRollup) -> Vec<GateCheck> {
    let metric = |name: &str| global.metrics.iter().find(|metric| metric.kpi == name);

    let context_reduction_check = metric("context_reduction_ratio")
        .map(|metric| metric.optimized.median >= 70.0)
        .unwrap_or(false);

    let edit_success_check = metric("edit_success_rate")
        .map(|metric| metric.optimized.median > 95.0)
        .unwrap_or(false);

    let structural_preservation_check = metric("structural_preservation")
        .map(|metric| metric.optimized.median >= 100.0)
        .unwrap_or(false);

    let conflict_rate_check = metric("conflict_rate")
        .map(|metric| metric.optimized.median < 10.0)
        .unwrap_or(false);

    let latency_check = metric("publish_latency")
        .map(|metric| {
            metric.optimized.median < 3000.0 && metric.optimized.p90 <= metric.baseline.p90
        })
        .unwrap_or(false);

    vec![
        GateCheck {
            name: "context_reduction_ratio".to_string(),
            target: "optimized median >= 70%".to_string(),
            pass: context_reduction_check,
        },
        GateCheck {
            name: "edit_success_rate".to_string(),
            target: "optimized median > 95%".to_string(),
            pass: edit_success_check,
        },
        GateCheck {
            name: "structural_preservation".to_string(),
            target: "optimized median = 100%".to_string(),
            pass: structural_preservation_check,
        },
        GateCheck {
            name: "conflict_rate".to_string(),
            target: "optimized median < 10%".to_string(),
            pass: conflict_rate_check,
        },
        GateCheck {
            name: "publish_latency".to_string(),
            target: "optimized median < 3000 ms and p90 <= baseline".to_string(),
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

    let mut context_reduction = summaries
        .iter()
        .map(|summary| OutlierRun {
            run_id: summary.run_id.clone(),
            kpi: "context_reduction_ratio".to_string(),
            value: summary.context_reduction_ratio * 100.0,
        })
        .collect::<Vec<_>>();
    context_reduction.sort_by(|left, right| {
        left.value
            .total_cmp(&right.value)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    outliers.extend(context_reduction.into_iter().take(3));

    outliers
}

fn build_regressions(flow_groups: &[FlowGroup]) -> Vec<RegressionSummary> {
    let mut regressions = Vec::new();

    for group in flow_groups {
        if group.baseline.is_empty() || group.optimized.is_empty() {
            continue;
        }

        let baseline_context_reduction =
            compute_stats(&kpi_values(&group.baseline, "context_reduction_ratio")).median;
        let optimized_context_reduction =
            compute_stats(&kpi_values(&group.optimized, "context_reduction_ratio")).median;
        if optimized_context_reduction < baseline_context_reduction {
            regressions.push(RegressionSummary {
                page_id: group.page_id.clone(),
                pattern: group.pattern.clone(),
                edit_intent_hash: group.edit_intent_hash.clone(),
                kpi: "context_reduction_ratio".to_string(),
                baseline: baseline_context_reduction,
                optimized: optimized_context_reduction,
            });
        }

        let baseline_edit_success =
            compute_stats(&kpi_values(&group.baseline, "edit_success_rate")).median;
        let optimized_edit_success =
            compute_stats(&kpi_values(&group.optimized, "edit_success_rate")).median;
        if optimized_edit_success < baseline_edit_success {
            regressions.push(RegressionSummary {
                page_id: group.page_id.clone(),
                pattern: group.pattern.clone(),
                edit_intent_hash: group.edit_intent_hash.clone(),
                kpi: "edit_success_rate".to_string(),
                baseline: baseline_edit_success,
                optimized: optimized_edit_success,
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

fn default_runtime_mode() -> String {
    RUNTIME_STUB.to_string()
}

fn collect_provenance(runtime_mode: &str) -> Result<ProvenanceStamp, DynError> {
    if !matches!(runtime_mode, RUNTIME_STUB | RUNTIME_LIVE) {
        return Err(format!(
            "invalid runtime mode `{runtime_mode}`: expected `{}` or `{}`",
            RUNTIME_STUB, RUNTIME_LIVE
        )
        .into());
    }

    let git_commit_sha = resolve_git_commit_sha()?;
    let git_dirty = resolve_git_dirty()?;
    let provenance = ProvenanceStamp {
        git_commit_sha,
        git_dirty,
        pipeline_version: PIPELINE_VERSION.to_string(),
        runtime_mode: runtime_mode.to_string(),
    };
    validate_provenance_stamp(&provenance)?;
    Ok(provenance)
}

fn resolve_git_commit_sha() -> Result<String, DynError> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Err("failed to collect git commit SHA via `git rev-parse HEAD`".into());
    }

    let value = String::from_utf8(output.stdout)?.trim().to_string();
    if value.len() != 40 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(
            "git commit SHA is malformed; expected 40 lowercase/uppercase hex chars".into(),
        );
    }
    Ok(value)
}

fn resolve_git_dirty() -> Result<bool, DynError> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    if !output.status.success() {
        return Err("failed to inspect git dirty state via `git status --porcelain`".into());
    }
    Ok(!String::from_utf8(output.stdout)?.trim().is_empty())
}

fn provenance_matches(summary: &RunSummary, provenance: &ProvenanceStamp) -> bool {
    summary.git_commit_sha == provenance.git_commit_sha
        && summary.git_dirty == provenance.git_dirty
        && summary.pipeline_version == provenance.pipeline_version
        && summary.runtime_mode == provenance.runtime_mode
}

fn summary_telemetry_complete(summary: &RunSummary) -> bool {
    summary.telemetry_complete && validate_run_summary_telemetry(summary).is_ok()
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

fn empty_page() -> serde_json::Value {
    serde_json::json!({
      "type": "doc",
      "version": 1,
      "content": [
        {
          "type": "paragraph",
          "content": []
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
    fn live_startup_errors_map_to_runtime_backend_hard_error() {
        let mapped = map_live_startup_error(ConfluenceError::Transport(
            "missing ATLASSY_CONFLUENCE_API_TOKEN".to_string(),
        ));

        match mapped {
            PipelineError::Hard {
                state,
                code,
                message,
            } => {
                assert_eq!(state, PipelineState::Fetch);
                assert_eq!(code, ERR_RUNTIME_BACKEND);
                assert!(message.contains("live runtime startup failure"));
                assert!(message.contains("missing ATLASSY_CONFLUENCE_API_TOKEN"));
            }
            other => panic!("unexpected mapped error: {other:?}"),
        }
    }

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
            serde_json::to_string_pretty(&summary_json)
                .expect("summary serialization should succeed"),
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
            diag.error_class.as_deref() == Some("provenance_incomplete")
                && diag.error_code.as_deref() == Some("ERR_PROVENANCE_MISMATCH")
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
        summary_json["error_codes"] = serde_json::json!([ERR_RUNTIME_UNMAPPED_HARD]);
        summary_json["failure_state"] = serde_json::json!("publish");
        fs::write(
            &summary_path,
            serde_json::to_string_pretty(&summary_json)
                .expect("summary serialization should succeed"),
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
            diag.error_class.as_deref() == Some("runtime_unmapped_hard")
                && diag.error_code.as_deref() == Some(ERR_RUNTIME_UNMAPPED_HARD)
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

    #[test]
    fn readiness_checklist_is_deterministic_and_blocks_on_mandatory_failures() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        execute_batch_from_manifest_file(
            &fixture_path("batch_complete_manifest.json"),
            temp.path(),
        )
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
    fn runbook_generation_includes_metadata_and_unknown_fallback() {
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
            serde_json::to_string_pretty(&report_json)
                .expect("report serialization should succeed"),
        )
        .expect("report should be written");

        let unknown_readiness = generate_readiness_outputs_from_artifacts(unknown_temp.path())
            .expect("readiness outputs should be generated");
        assert!(
            unknown_readiness
                .runbook_bundle
                .sections
                .iter()
                .any(|section| {
                    section.fallback && section.failure_class == "unknown:mystery_class"
                })
        );
        assert!(unknown_readiness.runbook_bundle.blocked);
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
        execute_batch_from_manifest_file(
            &fixture_path("batch_complete_manifest.json"),
            temp.path(),
        )
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
            serde_json::to_string_pretty(&packet_json)
                .expect("packet serialization should succeed"),
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
        // Batch WITHOUT lifecycle evidence should fail Gate 7
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
        // The recommendation should be "iterate" (coverage gate fails first)
        assert_eq!(no_lifecycle.decision_packet.recommendation, "iterate");

        // Batch WITH lifecycle evidence should pass Gate 7
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
}
