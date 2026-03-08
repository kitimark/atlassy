use atlassy_contracts::{ProvenanceStamp, TableOperation};

mod artifact_store;
mod error_map;
mod orchestrator;
mod state_tracker;
mod states;
mod util;

pub use artifact_store::ArtifactStore;
pub use error_map::PipelineError;
pub use orchestrator::Orchestrator;
pub use state_tracker::StateTracker;

#[derive(Debug, Clone)]
pub enum RunMode {
    NoOp,
    SimpleScopedUpdate {
        target_path: String,
        new_value: serde_json::Value,
    },
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

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub request_id: String,
    pub page_id: String,
    pub edit_intent: String,
    pub edit_intent_hash: String,
    pub flow: String,
    pub pattern: String,
    pub scope_selectors: Vec<String>,
    pub timestamp: String,
    pub provenance: ProvenanceStamp,
    pub run_mode: RunMode,
    pub target_index: usize,
    pub force_verify_fail: bool,
    pub bootstrap_empty_page: bool,
}
