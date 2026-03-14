use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use atlassy_confluence::{ConfluenceError, LiveConfluenceClient, StubConfluenceClient, StubPage};
use atlassy_contracts::{
    ErrorCode, FLOW_OPTIMIZED, PATTERN_A, PipelineState, RUNTIME_LIVE, RUNTIME_STUB,
};
use atlassy_pipeline::{Orchestrator, PipelineError, RunMode, RunRequest};
use chrono::Utc;

use crate::{DynError, collect_provenance, demo_page};

fn map_live_startup_error(error: ConfluenceError) -> PipelineError {
    PipelineError::Hard {
        state: PipelineState::Fetch,
        code: ErrorCode::RuntimeBackend,
        message: format!("live runtime startup failure: {error}"),
    }
}

pub fn run_single_request(
    request: RunRequest,
    artifacts_dir: PathBuf,
    runtime_mode: &str,
) -> Result<(), DynError> {
    match runtime_mode {
        RUNTIME_STUB => {
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
        RUNTIME_LIVE => {
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
        _ => {
            return Err(format!("invalid runtime mode `{runtime_mode}`").into());
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn execute_run_command(
    request_id: String,
    page_id: String,
    edit_intent: String,
    scope_selectors: Vec<String>,
    artifacts_dir: PathBuf,
    mode: &str,
    target_path: Option<String>,
    target_index: Option<usize>,
    new_value: Option<String>,
    force_verify_fail: bool,
    bootstrap_empty_page: bool,
    runtime_mode: &str,
) -> Result<(), DynError> {
    let provenance = collect_provenance(runtime_mode)?;
    let run_mode = match mode {
        "no-op" => RunMode::NoOp,
        "simple-scoped-prose-update" => RunMode::SimpleScopedProseUpdate {
            target_path,
            markdown: new_value.unwrap_or_else(|| "Updated prose body".to_string()),
        },
        "simple-scoped-table-cell-update" => RunMode::SimpleScopedTableCellUpdate {
            target_path,
            text: new_value.unwrap_or_else(|| "Updated table cell".to_string()),
        },
        _ => return Err(format!("invalid CLI mode `{mode}`").into()),
    };

    let request = RunRequest {
        request_id,
        page_id,
        edit_intent_hash: hash_edit_intent(&edit_intent),
        flow: FLOW_OPTIMIZED.to_string(),
        pattern: PATTERN_A.to_string(),
        edit_intent,
        scope_selectors,
        timestamp: Utc::now().to_rfc3339(),
        provenance,
        run_mode,
        target_index: target_index.unwrap_or_default(),
        force_verify_fail,
        bootstrap_empty_page,
    };

    run_single_request(request, artifacts_dir, runtime_mode)
}

pub fn hash_edit_intent(edit_intent: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    edit_intent.hash(&mut hasher);
    format!("h{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

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
                assert_eq!(code, ErrorCode::RuntimeBackend);
                assert!(message.contains("live runtime startup failure"));
                assert!(message.contains("missing ATLASSY_CONFLUENCE_API_TOKEN"));
            }
            other => panic!("unexpected mapped error: {other:?}"),
        }
    }
}
