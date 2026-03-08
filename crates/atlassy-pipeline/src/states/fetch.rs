use atlassy_adf::resolve_scope;
use atlassy_confluence::ConfluenceClient;
use atlassy_contracts::{Diagnostics, FetchInput, FetchOutput, PipelineState, StateEnvelope};

use crate::error_map::{confluence_error_to_hard_error, to_hard_error};
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_fetch_state<C: ConfluenceClient>(
    client: &mut C,
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
) -> Result<StateEnvelope<FetchOutput>, PipelineError> {
    tracker.transition_to(PipelineState::Fetch)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::Fetch),
        payload: FetchInput {
            page_id: request.page_id.clone(),
            edit_intent: request.edit_intent.clone(),
            scope_selectors: request.scope_selectors.clone(),
        },
    };
    input.validate_meta()?;

    let page = client
        .fetch_page(&request.page_id)
        .map_err(|error| confluence_error_to_hard_error(PipelineState::Fetch, error))?;

    let full_page_adf_bytes = serde_json::to_vec(&page.adf)
        .map(|value| value.len() as u64)
        .unwrap_or(0);

    let scope = resolve_scope(&page.adf, &request.scope_selectors)
        .map_err(|error| to_hard_error(PipelineState::Fetch, error))?;

    let output = StateEnvelope {
        meta: meta(request, PipelineState::Fetch),
        payload: FetchOutput {
            scoped_adf: scope.scoped_adf,
            page_version: page.page_version,
            allowed_scope_paths: scope.allowed_scope_paths,
            node_path_index: scope.node_path_index,
            scope_resolution_failed: scope.scope_resolution_failed,
            full_page_fetch: scope.full_page_fetch,
            fallback_reason: scope.fallback_reason,
            full_page_adf_bytes,
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::Fetch,
        &input,
        &output,
        &Diagnostics::default(),
    )?;

    Ok(output)
}
