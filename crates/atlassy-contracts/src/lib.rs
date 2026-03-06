use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONTRACT_VERSION: &str = "1.0.0";
pub const PIPELINE_VERSION: &str = "v1";

pub const ERR_SCOPE_MISS: &str = "ERR_SCOPE_MISS";
pub const ERR_ROUTE_VIOLATION: &str = "ERR_ROUTE_VIOLATION";
pub const ERR_SCHEMA_INVALID: &str = "ERR_SCHEMA_INVALID";
pub const ERR_OUT_OF_SCOPE_MUTATION: &str = "ERR_OUT_OF_SCOPE_MUTATION";
pub const ERR_LOCKED_NODE_MUTATION: &str = "ERR_LOCKED_NODE_MUTATION";
pub const ERR_TABLE_SHAPE_CHANGE: &str = "ERR_TABLE_SHAPE_CHANGE";
pub const ERR_CONFLICT_RETRY_EXHAUSTED: &str = "ERR_CONFLICT_RETRY_EXHAUSTED";

pub const FLOW_BASELINE: &str = "baseline";
pub const FLOW_OPTIMIZED: &str = "optimized";

pub const PATTERN_A: &str = "A";
pub const PATTERN_B: &str = "B";
pub const PATTERN_C: &str = "C";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PipelineState {
    Fetch,
    Classify,
    ExtractProse,
    MdAssistEdit,
    AdfTableEdit,
    MergeCandidates,
    Patch,
    Verify,
    Publish,
}

impl PipelineState {
    pub const ORDER: [PipelineState; 9] = [
        PipelineState::Fetch,
        PipelineState::Classify,
        PipelineState::ExtractProse,
        PipelineState::MdAssistEdit,
        PipelineState::AdfTableEdit,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCandidatesOutput {
    pub changed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOp {
    pub op: String,
    pub path: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInput {
    pub scoped_adf: serde_json::Value,
    pub changed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOutput {
    pub patch_ops: Vec<PatchOp>,
    pub candidate_page_adf: serde_json::Value,
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
    pub changed_paths: Vec<String>,
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
    pub pipeline_version: String,
    pub state_token_usage: BTreeMap<String, u64>,
    pub total_tokens: u64,
    pub retry_count: u32,
    pub retry_tokens: u64,
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
    pub telemetry_complete: bool,
    pub applied_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
    pub error_codes: Vec<String>,
    pub token_metrics: BTreeMap<String, u64>,
    pub failure_state: Option<PipelineState>,
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

pub fn validate_changed_paths(paths: &[String]) -> Result<(), ContractError> {
    let mut prev: Option<&str> = None;
    for path in paths {
        if !is_json_pointer(path) {
            return Err(ContractError::InvalidPath(path.clone()));
        }
        if let Some(previous) = prev
            && previous >= path
        {
            return Err(ContractError::InvalidChangedPaths);
        }
        prev = Some(path);
    }
    Ok(())
}

pub fn normalize_changed_paths(paths: &[String]) -> Result<Vec<String>, ContractError> {
    let mut normalized = paths.to_vec();
    normalized.sort();
    normalized.dedup();
    validate_changed_paths(&normalized)?;
    Ok(normalized)
}

pub fn is_json_pointer(path: &str) -> bool {
    path.starts_with('/')
}

pub fn validate_markdown_mapping(
    markdown_blocks: &[MarkdownBlock],
    md_to_adf_map: &[MarkdownMapEntry],
    editable_prose_paths: &[String],
    allowed_scope_paths: &[String],
) -> Result<(), ContractError> {
    let mut block_seen: BTreeMap<&str, usize> = BTreeMap::new();
    for block in markdown_blocks {
        *block_seen.entry(block.md_block_id.as_str()).or_insert(0) += 1;
    }
    for (block_id, count) in block_seen {
        if count != 1 {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "markdown block `{block_id}` appears {count} times"
            )));
        }
    }

    let mut map_seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut path_seen: BTreeMap<&str, usize> = BTreeMap::new();
    for entry in md_to_adf_map {
        if !is_json_pointer(&entry.adf_path) {
            return Err(ContractError::InvalidPath(entry.adf_path.clone()));
        }
        if !is_within_scope(&entry.adf_path, allowed_scope_paths) {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "mapped path `{}` is outside allowed scope",
                entry.adf_path
            )));
        }

        *map_seen.entry(entry.md_block_id.as_str()).or_insert(0) += 1;
        *path_seen.entry(entry.adf_path.as_str()).or_insert(0) += 1;
    }

    for block in markdown_blocks {
        if !map_seen.contains_key(block.md_block_id.as_str()) {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "missing map entry for markdown block `{}`",
                block.md_block_id
            )));
        }
    }

    for (block_id, count) in map_seen {
        if count != 1 {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "mapping for markdown block `{block_id}` appears {count} times"
            )));
        }
    }

    for (path, count) in path_seen {
        if count != 1 {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "mapped path `{path}` appears {count} times"
            )));
        }
    }

    for path in editable_prose_paths {
        if !md_to_adf_map.iter().any(|entry| entry.adf_path == *path) {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "editable prose path `{path}` is missing from map"
            )));
        }
    }

    Ok(())
}

pub fn validate_prose_changed_paths(
    prose_changed_paths: &[String],
    mapped_paths: &[String],
) -> Result<(), ContractError> {
    for path in prose_changed_paths {
        if !mapped_paths.iter().any(|mapped| {
            path == mapped
                || path
                    .strip_prefix(mapped)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        }) {
            return Err(ContractError::UnmappedProsePath(path.clone()));
        }
    }
    Ok(())
}

pub fn validate_table_candidates(
    candidates: &[TableChangeCandidate],
    allowed_ops: &[TableOperation],
) -> Result<(), ContractError> {
    let allowed = allowed_ops.iter().map(|op| op.as_str()).collect::<Vec<_>>();

    let mut previous: Option<&str> = None;
    for candidate in candidates {
        if !allowed.iter().any(|op| *op == candidate.op.as_str()) {
            return Err(ContractError::TableOperationNotAllowed(
                candidate.op.as_str().to_string(),
            ));
        }
        if !is_json_pointer(&candidate.path) {
            return Err(ContractError::InvalidPath(candidate.path.clone()));
        }
        if let Some(prev) = previous
            && prev >= candidate.path.as_str()
        {
            return Err(ContractError::TableCandidateOrder);
        }
        previous = Some(candidate.path.as_str());
    }
    Ok(())
}

pub fn validate_run_summary_telemetry(summary: &RunSummary) -> Result<(), ContractError> {
    if summary.run_id.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete("run_id".to_string()));
    }
    if summary.page_id.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete("page_id".to_string()));
    }
    if !matches!(summary.flow.as_str(), FLOW_BASELINE | FLOW_OPTIMIZED) {
        return Err(ContractError::TelemetryIncomplete("flow".to_string()));
    }
    if !matches!(summary.pattern.as_str(), PATTERN_A | PATTERN_B | PATTERN_C) {
        return Err(ContractError::TelemetryIncomplete("pattern".to_string()));
    }
    if summary.edit_intent_hash.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "edit_intent_hash".to_string(),
        ));
    }
    if summary.start_ts.trim().is_empty()
        || summary.verify_end_ts.trim().is_empty()
        || summary.publish_end_ts.trim().is_empty()
    {
        return Err(ContractError::TelemetryIncomplete("timestamps".to_string()));
    }
    if summary.verify_result.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "verify_result".to_string(),
        ));
    }
    if summary.publish_result.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "publish_result".to_string(),
        ));
    }
    if summary.state_token_usage.is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "state_token_usage".to_string(),
        ));
    }
    Ok(())
}

fn is_within_scope(path: &str, allowed_scope_paths: &[String]) -> bool {
    allowed_scope_paths.iter().any(|allowed| {
        if allowed == "/" {
            return true;
        }
        path == allowed
            || path
                .strip_prefix(allowed)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_state_enum_values_are_stable() {
        let serialized = serde_json::to_string(&PipelineState::MdAssistEdit).unwrap();
        assert_eq!(serialized, "\"md_assist_edit\"");

        let order: Vec<&str> = PipelineState::ORDER
            .iter()
            .map(|state| state.as_str())
            .collect();
        assert_eq!(
            order,
            vec![
                "fetch",
                "classify",
                "extract_prose",
                "md_assist_edit",
                "adf_table_edit",
                "merge_candidates",
                "patch",
                "verify",
                "publish",
            ]
        );
    }

    #[test]
    fn changed_paths_must_be_unique_and_sorted() {
        let valid = vec!["/a".to_string(), "/b".to_string()];
        assert!(validate_changed_paths(&valid).is_ok());

        let duplicate = vec!["/a".to_string(), "/a".to_string()];
        assert_eq!(
            validate_changed_paths(&duplicate),
            Err(ContractError::InvalidChangedPaths)
        );

        let unsorted = vec!["/b".to_string(), "/a".to_string()];
        assert_eq!(
            validate_changed_paths(&unsorted),
            Err(ContractError::InvalidChangedPaths)
        );
    }

    #[test]
    fn envelope_serialization_and_required_fields() {
        let envelope = StateEnvelope {
            meta: EnvelopeMeta {
                request_id: "req-1".to_string(),
                page_id: "18841604".to_string(),
                state: PipelineState::Fetch,
                timestamp: "2026-03-06T10:00:00Z".to_string(),
            },
            payload: FetchInput {
                page_id: "18841604".to_string(),
                edit_intent: "update section".to_string(),
                scope_selectors: vec!["heading:Overview".to_string()],
            },
        };

        envelope.validate_meta().unwrap();
        let serialized = serde_json::to_string(&envelope).unwrap();
        assert!(serialized.contains("\"request_id\":\"req-1\""));
        assert!(serialized.contains("\"state\":\"fetch\""));

        let invalid = StateEnvelope {
            meta: EnvelopeMeta {
                request_id: String::new(),
                page_id: "p".to_string(),
                state: PipelineState::Fetch,
                timestamp: "ts".to_string(),
            },
            payload: serde_json::json!({}),
        };

        assert_eq!(
            invalid.validate_meta(),
            Err(ContractError::MissingField("request_id"))
        );
    }

    #[test]
    fn markdown_mapping_must_be_one_to_one_and_in_scope() {
        let markdown_blocks = vec![
            MarkdownBlock {
                md_block_id: "/content/1".to_string(),
                markdown: "Hello".to_string(),
            },
            MarkdownBlock {
                md_block_id: "/content/2".to_string(),
                markdown: "World".to_string(),
            },
        ];

        let valid_map = vec![
            MarkdownMapEntry {
                md_block_id: "/content/1".to_string(),
                adf_path: "/content/1".to_string(),
            },
            MarkdownMapEntry {
                md_block_id: "/content/2".to_string(),
                adf_path: "/content/2".to_string(),
            },
        ];

        assert!(
            validate_markdown_mapping(
                &markdown_blocks,
                &valid_map,
                &["/content/1".to_string(), "/content/2".to_string()],
                &["/content".to_string()]
            )
            .is_ok()
        );

        let duplicate_block_map = vec![
            MarkdownMapEntry {
                md_block_id: "/content/1".to_string(),
                adf_path: "/content/1".to_string(),
            },
            MarkdownMapEntry {
                md_block_id: "/content/1".to_string(),
                adf_path: "/content/2".to_string(),
            },
        ];

        assert!(matches!(
            validate_markdown_mapping(
                &markdown_blocks,
                &duplicate_block_map,
                &["/content/1".to_string(), "/content/2".to_string()],
                &["/content".to_string()]
            ),
            Err(ContractError::InvalidMarkdownMapping(_))
        ));
    }

    #[test]
    fn prose_changed_paths_must_stay_within_mapped_paths() {
        let mapped_paths = vec!["/content/1".to_string(), "/content/2".to_string()];

        assert!(
            validate_prose_changed_paths(&["/content/1/text".to_string()], &mapped_paths).is_ok()
        );
        assert_eq!(
            validate_prose_changed_paths(&["/content/99".to_string()], &mapped_paths),
            Err(ContractError::UnmappedProsePath("/content/99".to_string()))
        );
    }

    #[test]
    fn prose_route_payload_serialization_is_deterministic() {
        let input = MdAssistEditInput {
            markdown_blocks: vec![MarkdownBlock {
                md_block_id: "/content/1".to_string(),
                markdown: "Initial prose".to_string(),
            }],
            md_to_adf_map: vec![MarkdownMapEntry {
                md_block_id: "/content/1".to_string(),
                adf_path: "/content/1".to_string(),
            }],
            editable_prose_paths: vec!["/content/1".to_string()],
            allowed_scope_paths: vec!["/content".to_string()],
            edit_intent: "Update one section".to_string(),
        };

        let first = serde_json::to_string(&input).unwrap();
        let second = serde_json::to_string(&input).unwrap();
        assert_eq!(first, second);
        assert!(first.contains("\"md_to_adf_map\""));
        assert!(first.contains("\"allowed_scope_paths\""));
    }

    #[test]
    fn table_candidates_allowlist_and_order_are_enforced() {
        let allowed = vec![TableOperation::CellTextUpdate];
        let valid = vec![
            TableChangeCandidate {
                op: TableOperation::CellTextUpdate,
                path: "/content/2/content/0/content/0/content/0/text".to_string(),
                value: serde_json::json!("A"),
                source_route: "table_adf".to_string(),
            },
            TableChangeCandidate {
                op: TableOperation::CellTextUpdate,
                path: "/content/2/content/0/content/0/content/1/text".to_string(),
                value: serde_json::json!("B"),
                source_route: "table_adf".to_string(),
            },
        ];
        assert!(validate_table_candidates(&valid, &allowed).is_ok());

        let forbidden = vec![TableChangeCandidate {
            op: TableOperation::RowAdd,
            path: "/content/2/content/0".to_string(),
            value: serde_json::json!({}),
            source_route: "table_adf".to_string(),
        }];
        assert_eq!(
            validate_table_candidates(&forbidden, &allowed),
            Err(ContractError::TableOperationNotAllowed(
                "row_add".to_string()
            ))
        );

        let unsorted = vec![
            TableChangeCandidate {
                op: TableOperation::CellTextUpdate,
                path: "/content/2/content/0/content/0/content/1/text".to_string(),
                value: serde_json::json!("B"),
                source_route: "table_adf".to_string(),
            },
            TableChangeCandidate {
                op: TableOperation::CellTextUpdate,
                path: "/content/2/content/0/content/0/content/0/text".to_string(),
                value: serde_json::json!("A"),
                source_route: "table_adf".to_string(),
            },
        ];
        assert_eq!(
            validate_table_candidates(&unsorted, &allowed),
            Err(ContractError::TableCandidateOrder)
        );
    }

    #[test]
    fn table_payload_serialization_is_deterministic() {
        let payload = AdfTableEditOutput {
            table_candidates: vec![TableChangeCandidate {
                op: TableOperation::CellTextUpdate,
                path: "/content/2/content/0/content/0/content/0/text".to_string(),
                value: serde_json::json!("Updated"),
                source_route: "table_adf".to_string(),
            }],
            table_changed_paths: vec!["/content/2/content/0/content/0/content/0/text".to_string()],
            allowed_ops: vec![TableOperation::CellTextUpdate],
        };

        let first = serde_json::to_string(&payload).unwrap();
        let second = serde_json::to_string(&payload).unwrap();
        assert_eq!(first, second);
        assert!(first.contains("\"cell_text_update\""));
    }

    #[test]
    fn run_summary_telemetry_validation_requires_kpi_fields() {
        let mut summary = RunSummary {
            success: true,
            run_id: "run-1".to_string(),
            request_id: "req-1".to_string(),
            page_id: "18841604".to_string(),
            flow: FLOW_OPTIMIZED.to_string(),
            pattern: PATTERN_A.to_string(),
            edit_intent_hash: "hash-1".to_string(),
            scope_selectors: vec!["heading:Overview".to_string()],
            scope_resolution_failed: false,
            full_page_fetch: false,
            pipeline_version: PIPELINE_VERSION.to_string(),
            state_token_usage: BTreeMap::from([
                ("fetch".to_string(), 0_u64),
                ("verify".to_string(), 0_u64),
                ("publish".to_string(), 0_u64),
            ]),
            total_tokens: 0,
            retry_count: 0,
            retry_tokens: 0,
            verify_result: "pass".to_string(),
            verify_error_codes: Vec::new(),
            publish_result: "published".to_string(),
            publish_error_code: None,
            new_version: Some(2),
            start_ts: "2026-03-06T10:00:00Z".to_string(),
            verify_end_ts: "2026-03-06T10:00:01Z".to_string(),
            publish_end_ts: "2026-03-06T10:00:02Z".to_string(),
            latency_ms: 200,
            locked_node_mutation: false,
            telemetry_complete: true,
            applied_paths: Vec::new(),
            blocked_paths: Vec::new(),
            error_codes: Vec::new(),
            token_metrics: BTreeMap::new(),
            failure_state: None,
        };

        assert!(validate_run_summary_telemetry(&summary).is_ok());

        summary.flow = "unknown".to_string();
        assert_eq!(
            validate_run_summary_telemetry(&summary),
            Err(ContractError::TelemetryIncomplete("flow".to_string()))
        );
    }
}
