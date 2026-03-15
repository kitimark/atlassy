use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

mod bootstrap;
mod builders;
mod index;
mod ordering;
mod patch;
mod path;
mod section;
mod scope;
mod structural_validity;
mod table_guard;
mod type_policy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeResolution {
    pub scoped_adf: Value,
    pub allowed_scope_paths: Vec<String>,
    pub node_path_index: BTreeMap<String, String>,
    pub scope_resolution_failed: bool,
    pub full_page_fetch: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetRoute {
    Prose,
    TableCell,
}

impl std::fmt::Display for TargetRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TargetRoute::Prose => "prose",
            TargetRoute::TableCell => "table_cell",
        };
        f.write_str(value)
    }
}

pub const EDITABLE_PROSE_TYPES: &[&str] = &[
    "paragraph",
    "heading",
    "bulletList",
    "orderedList",
    "listItem",
    "blockquote",
    "codeBlock",
];

pub const SCOPE_ANCHOR_TYPES: &[&str] = &["heading"];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdfError {
    #[error("scope resolution failed")]
    ScopeResolutionFailed,
    #[error("invalid selector format: {0}")]
    InvalidSelector(String),
    #[error("invalid JSON pointer path: {0}")]
    InvalidPath(String),
    #[error("duplicate path in node index: {0}")]
    DuplicatePath(String),
    #[error("whole-body rewrite is not allowed")]
    WholeBodyRewriteDisallowed,
    #[error("path `{0}` is outside allowed scope")]
    OutOfScope(String),
    #[error("mapping integrity failure: {0}")]
    MappingIntegrity(String),
    #[error("insert position invalid: {0}")]
    InsertPositionInvalid(String),
    #[error("remove target not found: {0}")]
    RemoveTargetNotFound(String),
    #[error("post-mutation ADF invalid: {0}")]
    PostMutationInvalid(String),
    #[error("operation conflict: {0}")]
    OperationConflict(String),
    #[error("section boundary invalid: {0}")]
    SectionBoundaryInvalid(String),
    #[error("structural composition failed: {0}")]
    StructuralCompositionFailed(String),
    #[error("no valid {route} target found in scope at index {index} (found {found})")]
    TargetDiscoveryFailed {
        route: String,
        index: usize,
        found: usize,
    },
}

pub use bootstrap::*;
pub use builders::*;
pub use index::*;
pub use ordering::*;
pub use patch::*;
pub use path::*;
pub use section::*;
pub use scope::*;
pub use structural_validity::*;
pub use table_guard::*;
pub use type_policy::*;
