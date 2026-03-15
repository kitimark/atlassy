use atlassy_adf::{apply_operations, sort_operations, validate_operations};
use atlassy_contracts::{
    AdfTableEditOutput, Diagnostics, FetchOutput, MdAssistEditOutput, MergeCandidatesOutput,
    Operation, PatchInput, PatchOutput, PipelineState, StateEnvelope,
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
    md_edit: &StateEnvelope<MdAssistEditOutput>,
    table_edit: &StateEnvelope<AdfTableEditOutput>,
) -> Result<StateEnvelope<PatchOutput>, PipelineError> {
    tracker.transition_to(PipelineState::Patch)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::Patch),
        payload: PatchInput {
            scoped_adf: fetch.payload.scoped_adf.clone(),
            changed_paths: merged.payload.changed_paths.clone(),
        },
    };

    let operations = md_edit
        .payload
        .prose_change_candidates
        .iter()
        .map(|candidate| Operation::Replace {
            path: candidate.path.clone(),
            value: serde_json::Value::String(candidate.markdown.clone()),
        })
        .chain(
            table_edit
                .payload
                .table_candidates
                .iter()
                .map(|candidate| Operation::Replace {
                    path: candidate.path.clone(),
                    value: candidate.value.clone(),
                }),
        )
        .collect::<Vec<_>>();
    validate_operations(&operations, &fetch.payload.allowed_scope_paths)
        .map_err(|error| to_hard_error(PipelineState::Patch, error))?;
    let patch_ops = sort_operations(&operations);
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
