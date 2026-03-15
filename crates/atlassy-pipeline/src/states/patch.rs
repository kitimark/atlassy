use atlassy_adf::{apply_operations, sort_operations, validate_operations};
use atlassy_contracts::{
    Diagnostics, FetchOutput, MergeCandidatesOutput, PatchInput, PatchOutput, PipelineState,
    StateEnvelope,
};

use crate::error_map::to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_patch_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
    merged: &StateEnvelope<MergeCandidatesOutput>,
) -> Result<StateEnvelope<PatchOutput>, PipelineError> {
    tracker.transition_to(PipelineState::Patch)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::Patch),
        payload: PatchInput {
            scoped_adf: fetch.payload.scoped_adf.clone(),
            operations: merged.payload.operations.clone(),
        },
    };

    let operations = merged.payload.operations.clone();
    validate_operations(&operations, &fetch.payload.allowed_scope_paths)
        .map_err(|error| to_hard_error(PipelineState::Patch, error))?;
    let patch_ops =
        sort_operations(&operations).map_err(|error| to_hard_error(PipelineState::Patch, error))?;
    let candidate_page_adf = apply_operations(&fetch.payload.scoped_adf, &patch_ops)
        .map_err(|error| to_hard_error(PipelineState::Patch, error))?;
    let patch_ops_bytes = serde_json::to_vec(&patch_ops)
        .map(|value| value.len() as u64)
        .unwrap_or(0);

    let output = StateEnvelope {
        meta: meta(request, PipelineState::Patch),
        payload: PatchOutput {
            patch_ops,
            candidate_page_adf,
            patch_ops_bytes,
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::Patch,
        &input,
        &output,
        &Diagnostics::default(),
    )?;
    Ok(output)
}
