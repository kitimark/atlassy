use std::collections::BTreeMap;
use std::error::Error;

use atlassy_contracts::{ErrorCode, ProvenanceStamp, RUNTIME_STUB, RunSummary};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const REQUIRED_SCENARIO_IDS: [&str; 10] = [
    "S-001", "S-002", "S-003", "S-004", "S-005", "S-006", "S-007", "S-008", "S-009", "S-010",
];

pub type DynError = Box<dyn Error>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunManifest {
    #[serde(default)]
    pub batch: BatchManifestMetadata,
    pub runs: Vec<ManifestRunEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchManifestMetadata {
    #[serde(default = "default_required_scenario_ids")]
    pub required_scenario_ids: Vec<String>,
    #[serde(default)]
    pub observed_scenario_ids: Vec<String>,
    #[serde(default)]
    pub live_smoke: DriftStatusInput,
    #[serde(default = "default_runtime_mode")]
    pub runtime_mode: String,
    #[serde(default)]
    pub lifecycle_create_subpage_validated: bool,
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
pub struct DriftStatusInput {
    #[serde(default = "default_true")]
    pub scoped_fetch_parity: bool,
    #[serde(default = "default_true")]
    pub publish_conflict_parity: bool,
    #[serde(default = "default_true")]
    pub error_payload_parity: bool,
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
pub struct ManifestRunEntry {
    pub run_id: String,
    pub page_id: String,
    pub pattern: String,
    pub flow: String,
    pub edit_intent: String,
    pub edit_intent_hash: String,
    #[serde(default)]
    pub scenario_ids: Vec<String>,
    #[serde(default)]
    pub scope_selectors: Vec<String>,
    #[serde(default = "default_manifest_timestamp")]
    pub timestamp: String,
    #[serde(default)]
    pub mode: ManifestMode,
    pub target_path: Option<String>,
    #[serde(default)]
    pub target_index: Option<u32>,
    pub new_value: Option<String>,
    #[serde(default)]
    pub force_verify_fail: bool,
    #[serde(default)]
    pub simulate_conflict_once: bool,
    #[serde(default)]
    pub simulate_conflict_exhausted: bool,
    pub bootstrap_empty_page: Option<bool>,
    #[serde(default)]
    pub simulate_empty_page: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestMode {
    #[default]
    NoOp,
    SimpleScopedUpdate,
    SimpleScopedProseUpdate,
    SimpleScopedTableCellUpdate,
}

const ERROR_CLASS_STRINGS: [&str; 6] = [
    "io",
    "telemetry_incomplete",
    "provenance_incomplete",
    "retry_policy",
    "runtime_unmapped_hard",
    "pipeline_hard",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Io,
    TelemetryIncomplete,
    ProvenanceIncomplete,
    RetryPolicy,
    RuntimeUnmappedHard,
    PipelineHard,
}

impl ErrorClass {
    pub const ALL: [Self; 6] = [
        Self::Io,
        Self::TelemetryIncomplete,
        Self::ProvenanceIncomplete,
        Self::RetryPolicy,
        Self::RuntimeUnmappedHard,
        Self::PipelineHard,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::TelemetryIncomplete => "telemetry_incomplete",
            Self::ProvenanceIncomplete => "provenance_incomplete",
            Self::RetryPolicy => "retry_policy",
            Self::RuntimeUnmappedHard => "runtime_unmapped_hard",
            Self::PipelineHard => "pipeline_hard",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "io" => Some(Self::Io),
            "telemetry_incomplete" => Some(Self::TelemetryIncomplete),
            "provenance_incomplete" => Some(Self::ProvenanceIncomplete),
            "retry_policy" => Some(Self::RetryPolicy),
            "runtime_unmapped_hard" => Some(Self::RuntimeUnmappedHard),
            "pipeline_hard" => Some(Self::PipelineHard),
            _ => None,
        }
    }
}

impl Serialize for ErrorClass {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ErrorClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).ok_or_else(|| DeError::unknown_variant(&value, &ERROR_CLASS_STRINGS))
    }
}

const DIAGNOSTIC_CODE_CLI_STRINGS: [&str; 3] = [
    "ERR_SUMMARY_MISSING",
    "ERR_TELEMETRY_INCOMPLETE",
    "ERR_PROVENANCE_MISMATCH",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCode {
    Pipeline(ErrorCode),
    SummaryMissing,
    TelemetryIncomplete,
    ProvenanceMismatch,
}

impl DiagnosticCode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pipeline(code) => code.as_str(),
            Self::SummaryMissing => "ERR_SUMMARY_MISSING",
            Self::TelemetryIncomplete => "ERR_TELEMETRY_INCOMPLETE",
            Self::ProvenanceMismatch => "ERR_PROVENANCE_MISMATCH",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "ERR_SUMMARY_MISSING" => Some(Self::SummaryMissing),
            "ERR_TELEMETRY_INCOMPLETE" => Some(Self::TelemetryIncomplete),
            "ERR_PROVENANCE_MISMATCH" => Some(Self::ProvenanceMismatch),
            _ => ErrorCode::ALL
                .iter()
                .copied()
                .find(|code| code.as_str() == value)
                .map(Self::Pipeline),
        }
    }
}

impl Serialize for DiagnosticCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DiagnosticCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).ok_or_else(|| {
            DeError::unknown_variant(
                &value,
                &[
                    DIAGNOSTIC_CODE_CLI_STRINGS[0],
                    DIAGNOSTIC_CODE_CLI_STRINGS[1],
                    DIAGNOSTIC_CODE_CLI_STRINGS[2],
                    "ERR_* from ErrorCode::ALL",
                ],
            )
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BatchRunDiagnostic {
    pub run_id: String,
    pub page_id: String,
    pub pattern: String,
    pub flow: String,
    pub status: String,
    pub error_class: Option<ErrorClass>,
    pub error_code: Option<DiagnosticCode>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BatchReport {
    pub total_runs: usize,
    pub succeeded_runs: usize,
    pub failed_runs: usize,
    pub status: String,
    pub telemetry_complete: bool,
    pub retry_policy_ok: bool,
    pub provenance: ProvenanceStamp,
    pub diagnostics: Vec<BatchRunDiagnostic>,
    pub artifact_index_path: String,
    pub drift: DriftAssessment,
    pub scenario_coverage: ScenarioCoverageAssessment,
    pub safety: SafetyAssessment,
    pub gate_checks: Vec<GateCheck>,
    pub kpi: Option<KpiReport>,
    pub recommendation: RecommendationSection,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BatchArtifactIndex {
    pub batch: BatchArtifactMetadata,
    pub runs: Vec<RunArtifactIndexEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BatchArtifactMetadata {
    pub manifest_path: String,
    pub normalized_manifest_path: String,
    pub report_path: String,
    pub required_scenario_ids: Vec<String>,
    pub observed_scenario_ids: Vec<String>,
    pub live_smoke: DriftStatusInput,
    pub provenance: ProvenanceStamp,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RunArtifactIndexEntry {
    pub run_id: String,
    pub page_id: String,
    pub pattern: String,
    pub flow: String,
    pub edit_intent_hash: String,
    pub summary_path: String,
    pub state_artifacts: Vec<StateArtifactIndexEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StateArtifactIndexEntry {
    pub state: String,
    pub input_path: String,
    pub output_path: String,
    pub diagnostics_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DriftAssessment {
    pub scoped_fetch_parity: bool,
    pub publish_conflict_parity: bool,
    pub error_payload_parity: bool,
    pub unresolved_material_drift: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioCoverageAssessment {
    pub required_scenario_ids: Vec<String>,
    pub observed_scenario_ids: Vec<String>,
    pub missing_scenario_ids: Vec<String>,
    pub complete: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SafetyAssessment {
    pub locked_node_violation_runs: Vec<String>,
    pub out_of_scope_violation_runs: Vec<String>,
    pub table_shape_violation_runs: Vec<String>,
    pub safety_failed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GateCheck {
    pub name: String,
    pub target: String,
    pub pass: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct KpiReport {
    pub global_rollup: KpiRollup,
    pub pattern_rollups: Vec<KpiRollup>,
    pub checks: Vec<GateCheck>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct KpiRollup {
    pub scope: String,
    pub pair_count: usize,
    pub metrics: Vec<KpiMetricComparison>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct KpiMetricComparison {
    pub kpi: String,
    pub baseline: KpiStats,
    pub optimized: KpiStats,
    pub delta_absolute: f64,
    pub delta_relative: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct KpiStats {
    pub count: usize,
    pub median: f64,
    pub p90: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RecommendationSection {
    pub decision: String,
    pub rationale: Vec<String>,
    pub outliers: Vec<OutlierRun>,
    pub regressions: Vec<RegressionSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OutlierRun {
    pub run_id: String,
    pub kpi: String,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RegressionSummary {
    pub page_id: String,
    pub pattern: String,
    pub edit_intent_hash: String,
    pub kpi: String,
    pub baseline: f64,
    pub optimized: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReadinessGateResult {
    pub gate_id: String,
    pub gate_name: String,
    pub target: String,
    pub pass: bool,
    pub mandatory: bool,
    pub owner_role: String,
    pub blocking_reason: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReadinessChecklist {
    pub schema_version: String,
    pub generated_ts: String,
    pub owner_roles: ReadinessOwnerRoles,
    pub source_artifacts: Vec<String>,
    pub provenance: ProvenanceStamp,
    pub gates: Vec<ReadinessGateResult>,
    pub blocked: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReadinessOwnerRoles {
    pub product_owner: String,
    pub engineering_owner: String,
    pub qa_owner: String,
    pub data_metrics_owner: String,
    pub release_reviewer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RunbookSection {
    pub failure_class: String,
    pub severity: String,
    pub primary_owner_role: String,
    pub escalation_owner_role: String,
    pub escalation_trigger: String,
    pub triage_steps: Vec<String>,
    pub evidence_checks: Vec<String>,
    pub fallback: bool,
    pub blocks_signoff: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RunbookBundle {
    pub schema_version: String,
    pub generated_ts: String,
    pub provenance: ProvenanceStamp,
    pub sections: Vec<RunbookSection>,
    pub blocked: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RiskStatusDelta {
    pub risk_id: String,
    pub title: String,
    pub priority: String,
    pub previous_status: String,
    pub current_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FailureClassSummary {
    pub failure_class: String,
    pub severity: String,
    pub count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DecisionPacket {
    pub schema_version: String,
    pub generated_ts: String,
    pub recommendation: String,
    pub provenance: ProvenanceStamp,
    pub blocking_condition: Option<String>,
    pub rationale: Vec<String>,
    pub gate_outcomes: Vec<ReadinessGateResult>,
    pub kpi_summary: Option<KpiReport>,
    pub risk_status_deltas: Vec<RiskStatusDelta>,
    pub top_failure_classes: Vec<FailureClassSummary>,
    pub checklist_path: String,
    pub runbook_path: String,
    pub report_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReadinessOutputs {
    pub checklist: ReadinessChecklist,
    pub runbook_bundle: RunbookBundle,
    pub decision_packet: DecisionPacket,
}

#[derive(Debug, Clone)]
pub struct ReadinessEvidence {
    pub manifest: RunManifest,
    pub provenance: ProvenanceStamp,
    pub report: BatchReport,
    pub summaries: BTreeMap<String, RunSummary>,
    pub source_artifacts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FlowGroup {
    pub page_id: String,
    pub pattern: String,
    pub edit_intent_hash: String,
    pub baseline: Vec<RunSummary>,
    pub optimized: Vec<RunSummary>,
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

fn default_manifest_timestamp() -> String {
    "1970-01-01T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticCode, ErrorClass};
    use atlassy_contracts::ErrorCode;

    #[test]
    fn error_class_round_trips_with_stable_strings() {
        for (class, expected_json) in [
            (ErrorClass::Io, "\"io\""),
            (ErrorClass::TelemetryIncomplete, "\"telemetry_incomplete\""),
            (
                ErrorClass::ProvenanceIncomplete,
                "\"provenance_incomplete\"",
            ),
            (ErrorClass::RetryPolicy, "\"retry_policy\""),
            (ErrorClass::RuntimeUnmappedHard, "\"runtime_unmapped_hard\""),
            (ErrorClass::PipelineHard, "\"pipeline_hard\""),
        ] {
            let encoded = serde_json::to_string(&class).expect("error class should serialize");
            assert_eq!(encoded, expected_json);

            let decoded: ErrorClass =
                serde_json::from_str(&encoded).expect("error class should deserialize");
            assert_eq!(decoded, class);
        }
    }

    #[test]
    fn diagnostic_code_round_trips_with_flat_strings() {
        for (code, expected_json) in [
            (DiagnosticCode::SummaryMissing, "\"ERR_SUMMARY_MISSING\""),
            (
                DiagnosticCode::TelemetryIncomplete,
                "\"ERR_TELEMETRY_INCOMPLETE\"",
            ),
            (
                DiagnosticCode::ProvenanceMismatch,
                "\"ERR_PROVENANCE_MISMATCH\"",
            ),
        ] {
            let encoded = serde_json::to_string(&code).expect("diagnostic code should serialize");
            assert_eq!(encoded, expected_json);

            let decoded: DiagnosticCode =
                serde_json::from_str(&encoded).expect("diagnostic code should deserialize");
            assert_eq!(decoded, code);
        }

        for error_code in ErrorCode::ALL {
            let code = DiagnosticCode::Pipeline(error_code);
            let encoded = serde_json::to_string(&code).expect("pipeline code should serialize");
            assert_eq!(encoded, format!("\"{}\"", error_code.as_str()));

            let decoded: DiagnosticCode =
                serde_json::from_str(&encoded).expect("pipeline code should deserialize");
            assert_eq!(decoded, code);
        }

        let scope_miss: DiagnosticCode =
            serde_json::from_str("\"ERR_SCOPE_MISS\"").expect("scope miss should deserialize");
        assert_eq!(scope_miss, DiagnosticCode::Pipeline(ErrorCode::ScopeMiss));

        assert!(
            serde_json::from_str::<DiagnosticCode>(r#"{"Pipeline":"ERR_SCOPE_MISS"}"#).is_err(),
            "tagged structure should not deserialize"
        );
        assert!(
            serde_json::from_str::<DiagnosticCode>("\"ERR_UNKNOWN\"").is_err(),
            "unknown diagnostic code should fail"
        );
    }
}
