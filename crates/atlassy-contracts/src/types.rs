use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceStamp {
    pub git_commit_sha: String,
    pub git_dirty: bool,
    pub pipeline_version: String,
    pub runtime_mode: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PipelineState {
    Fetch,
    Classify,
    ExtractProse,
    MdAssistEdit,
    AdfTableEdit,
    AdfBlockOps,
    MergeCandidates,
    Patch,
    Verify,
    Publish,
}

impl PipelineState {
    pub const ORDER: [PipelineState; 10] = [
        PipelineState::Fetch,
        PipelineState::Classify,
        PipelineState::ExtractProse,
        PipelineState::MdAssistEdit,
        PipelineState::AdfTableEdit,
        PipelineState::AdfBlockOps,
        PipelineState::MergeCandidates,
        PipelineState::Patch,
        PipelineState::Verify,
        PipelineState::Publish,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            PipelineState::Fetch => "fetch",
            PipelineState::Classify => "classify",
            PipelineState::ExtractProse => "extract_prose",
            PipelineState::MdAssistEdit => "md_assist_edit",
            PipelineState::AdfTableEdit => "adf_table_edit",
            PipelineState::AdfBlockOps => "adf_block_ops",
            PipelineState::MergeCandidates => "merge_candidates",
            PipelineState::Patch => "patch",
            PipelineState::Verify => "verify",
            PipelineState::Publish => "publish",
        }
    }

    pub fn expected_next(current: Option<PipelineState>) -> Option<PipelineState> {
        let start_idx = current
            .and_then(|state| Self::ORDER.iter().position(|candidate| *candidate == state))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        Self::ORDER.get(start_idx).copied()
    }
}

impl std::fmt::Display for PipelineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeMeta {
    pub request_id: String,
    pub page_id: String,
    pub state: PipelineState,
    pub timestamp: String,
}

impl EnvelopeMeta {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.request_id.trim().is_empty() {
            return Err(ContractError::MissingField("request_id"));
        }
        if self.page_id.trim().is_empty() {
            return Err(ContractError::MissingField("page_id"));
        }
        if self.timestamp.trim().is_empty() {
            return Err(ContractError::MissingField("timestamp"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEnvelope<T> {
    #[serde(flatten)]
    pub meta: EnvelopeMeta,
    pub payload: T,
}

impl<T> StateEnvelope<T> {
    pub fn validate_meta(&self) -> Result<(), ContractError> {
        self.meta.validate()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRef {
    pub path: String,
    pub node_type: String,
    pub route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub recovery: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Diagnostics {
    pub warnings: Vec<String>,
    pub errors: Vec<ErrorInfo>,
    pub metrics: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchInput {
    pub page_id: String,
    pub edit_intent: String,
    pub scope_selectors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOutput {
    pub scoped_adf: serde_json::Value,
    pub page_version: u64,
    pub allowed_scope_paths: Vec<String>,
    pub node_path_index: BTreeMap<String, String>,
    pub scope_resolution_failed: bool,
    pub full_page_fetch: bool,
    pub fallback_reason: Option<String>,
    pub full_page_adf_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyInput {
    pub scoped_adf: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyOutput {
    pub node_manifest: Vec<NodeRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractProseInput {
    pub node_manifest: Vec<NodeRef>,
    pub scoped_adf: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownBlock {
    pub md_block_id: String,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownMapEntry {
    pub md_block_id: String,
    pub adf_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractProseOutput {
    pub markdown_blocks: Vec<MarkdownBlock>,
    pub md_to_adf_map: Vec<MarkdownMapEntry>,
    pub editable_prose_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdAssistEditInput {
    pub markdown_blocks: Vec<MarkdownBlock>,
    pub md_to_adf_map: Vec<MarkdownMapEntry>,
    pub editable_prose_paths: Vec<String>,
    pub allowed_scope_paths: Vec<String>,
    pub edit_intent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProseChangeCandidate {
    pub path: String,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdAssistEditOutput {
    pub edited_markdown_blocks: Vec<MarkdownBlock>,
    pub prose_changed_paths: Vec<String>,
    pub prose_change_candidates: Vec<ProseChangeCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfTableEditInput {
    pub table_nodes: Vec<NodeRef>,
    pub allowed_scope_paths: Vec<String>,
    pub edit_intent: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableOperation {
    CellTextUpdate,
    RowAdd,
    RowRemove,
    ColumnAdd,
    ColumnRemove,
    MergeCells,
    SplitCells,
    TableAttrUpdate,
}

impl TableOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            TableOperation::CellTextUpdate => "cell_text_update",
            TableOperation::RowAdd => "row_add",
            TableOperation::RowRemove => "row_remove",
            TableOperation::ColumnAdd => "column_add",
            TableOperation::ColumnRemove => "column_remove",
            TableOperation::MergeCells => "merge_cells",
            TableOperation::SplitCells => "split_cells",
            TableOperation::TableAttrUpdate => "table_attr_update",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableChangeCandidate {
    pub op: TableOperation,
    pub path: String,
    pub value: serde_json::Value,
    pub source_route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfTableEditOutput {
    pub table_candidates: Vec<TableChangeCandidate>,
    pub table_changed_paths: Vec<String>,
    pub allowed_ops: Vec<TableOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCandidatesInput {
    pub prose_changed_paths: Vec<String>,
    pub table_changed_paths: Vec<String>,
    pub block_operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCandidatesOutput {
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    Replace {
        path: String,
        value: serde_json::Value,
    },
    Insert {
        parent_path: String,
        index: usize,
        block: serde_json::Value,
    },
    Remove {
        target_path: String,
    },
    UpdateAttrs {
        target_path: String,
        attrs: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum BlockOp {
    Insert {
        parent_path: String,
        index: usize,
        block: serde_json::Value,
    },
    Remove {
        target_path: String,
    },
    InsertSection {
        parent_path: String,
        index: usize,
        heading_level: u8,
        heading_text: String,
        body_blocks: Vec<serde_json::Value>,
    },
    RemoveSection {
        heading_path: String,
    },
    InsertTable {
        parent_path: String,
        index: usize,
        rows: usize,
        cols: usize,
        header_row: bool,
    },
    InsertList {
        parent_path: String,
        index: usize,
        ordered: bool,
        items: Vec<String>,
    },
    InsertRow {
        table_path: String,
        index: usize,
        cells: Vec<String>,
    },
    RemoveRow {
        table_path: String,
        index: usize,
    },
    InsertColumn {
        table_path: String,
        index: usize,
    },
    RemoveColumn {
        table_path: String,
        index: usize,
    },
    UpdateAttrs {
        target_path: String,
        attrs: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInput {
    pub scoped_adf: serde_json::Value,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOutput {
    pub patch_ops: Vec<Operation>,
    pub candidate_page_adf: serde_json::Value,
    pub patch_ops_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifyResult {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
    pub original_scoped_adf: serde_json::Value,
    pub candidate_page_adf: serde_json::Value,
    pub allowed_scope_paths: Vec<String>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutput {
    pub verify_result: VerifyResult,
    pub diagnostics: Diagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublishResult {
    Published,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishInput {
    pub candidate_page_adf: serde_json::Value,
    pub page_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishOutput {
    pub publish_result: PublishResult,
    pub new_version: Option<u64>,
    pub retry_count: u32,
    pub diagnostics: Diagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub success: bool,
    pub run_id: String,
    pub request_id: String,
    pub page_id: String,
    pub flow: String,
    pub pattern: String,
    pub edit_intent_hash: String,
    pub scope_selectors: Vec<String>,
    pub scope_resolution_failed: bool,
    pub full_page_fetch: bool,
    pub full_page_adf_bytes: u64,
    pub scoped_adf_bytes: u64,
    pub context_reduction_ratio: f64,
    pub pipeline_version: String,
    pub git_commit_sha: String,
    pub git_dirty: bool,
    pub runtime_mode: String,
    pub state_token_usage: BTreeMap<String, u64>,
    pub total_tokens: u64,
    pub retry_count: u32,
    pub retry_tokens: u64,
    pub patch_ops_bytes: u64,
    pub verify_result: String,
    pub verify_error_codes: Vec<String>,
    pub publish_result: String,
    pub publish_error_code: Option<String>,
    pub new_version: Option<u64>,
    pub start_ts: String,
    pub verify_end_ts: String,
    pub publish_end_ts: String,
    pub latency_ms: u64,
    pub locked_node_mutation: bool,
    pub out_of_scope_mutation: bool,
    pub telemetry_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_target_path: Option<String>,
    pub applied_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
    pub error_codes: Vec<String>,
    pub token_metrics: BTreeMap<String, u64>,
    pub failure_state: Option<PipelineState>,
    pub empty_page_detected: bool,
    pub bootstrap_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPageRequest {
    pub plan_id: String,
    pub pages: Vec<PageTarget>,
    pub rollback_on_failure: bool,
    pub provenance: ProvenanceStamp,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTarget {
    pub page_id: Option<String>,
    pub create: Option<CreatePageTarget>,
    pub edit_intent: String,
    pub scope_selectors: Vec<String>,
    pub run_mode: PageRunMode,
    pub block_ops: Vec<BlockOp>,
    pub bootstrap_empty_page: bool,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageTarget {
    pub title: String,
    pub parent_page_id: String,
    pub space_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSnapshot {
    pub page_id: String,
    pub version_before: u64,
    pub adf_before: serde_json::Value,
    pub version_after: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPageSummary {
    pub plan_id: String,
    pub success: bool,
    pub page_results: Vec<PageResult>,
    pub rollback_results: Vec<RollbackResult>,
    pub total_pages: usize,
    pub succeeded_pages: usize,
    pub failed_pages: usize,
    pub rolled_back_pages: usize,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResult {
    pub page_id: String,
    pub created: bool,
    pub summary: RunSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub page_id: String,
    pub success: bool,
    pub conflict: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum PageRunMode {
    NoOp,
    SimpleScopedProseUpdate {
        target_path: Option<String>,
        markdown: String,
    },
    SimpleScopedTableCellUpdate {
        target_path: Option<String>,
        text: String,
    },
    ForbiddenTableOperation {
        target_path: String,
        operation: TableOperation,
    },
    SyntheticRouteConflict {
        prose_path: String,
        table_path: String,
    },
    SyntheticTableShapeDrift {
        path: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("expected next state `{expected}`, got `{actual}`")]
    InvalidTransition { expected: String, actual: String },
    #[error("changed paths must be unique and lexicographically sorted")]
    InvalidChangedPaths,
    #[error("invalid JSON pointer path: {0}")]
    InvalidPath(String),
    #[error("invalid markdown mapping: {0}")]
    InvalidMarkdownMapping(String),
    #[error("prose path is not mapped: {0}")]
    UnmappedProsePath(String),
    #[error("prose boundary violation: {0}")]
    ProseBoundaryViolation(String),
    #[error("table operation is not allowed: {0}")]
    TableOperationNotAllowed(String),
    #[error("table candidate paths must be deterministic and sorted")]
    TableCandidateOrder,
    #[error("telemetry incomplete: {0}")]
    TelemetryIncomplete(String),
}
