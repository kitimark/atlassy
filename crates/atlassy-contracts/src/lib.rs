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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdAssistEditInput {
    pub markdown_blocks: Vec<MarkdownBlock>,
    pub edit_intent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdAssistEditOutput {
    pub edited_markdown_blocks: Vec<MarkdownBlock>,
    pub prose_changed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfTableEditInput {
    pub table_nodes: Vec<NodeRef>,
    pub edit_intent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfTableEditOutput {
    pub table_candidate_nodes: Vec<NodeRef>,
    pub table_changed_paths: Vec<String>,
    pub allowed_ops: Vec<String>,
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
    pub diagnostics: Diagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub success: bool,
    pub request_id: String,
    pub page_id: String,
    pub pipeline_version: String,
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
}
