use atlassy_adf::{
    ensure_paths_in_scope, is_table_cell_text_path, is_table_shape_or_attr_path,
    path_has_ancestor_type,
};
use atlassy_contracts::{
    Diagnostics, ErrorCode, ErrorInfo, FetchOutput, MergeCandidatesOutput, PatchOutput,
    PipelineState, StateEnvelope, VerifyInput, VerifyOutput, VerifyResult,
};

use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_verify_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
    patch: &StateEnvelope<PatchOutput>,
    merged: &StateEnvelope<MergeCandidatesOutput>,
) -> Result<StateEnvelope<VerifyOutput>, PipelineError> {
    tracker.transition_to(PipelineState::Verify)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::Verify),
        payload: VerifyInput {
            original_scoped_adf: fetch.payload.scoped_adf.clone(),
            candidate_page_adf: patch.payload.candidate_page_adf.clone(),
            allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
            changed_paths: merged.payload.changed_paths.clone(),
        },
    };

    let mut diagnostics = Diagnostics::default();
    let verify_result = if request.force_verify_fail {
        diagnostics.errors.push(ErrorInfo {
            code: ErrorCode::SchemaInvalid.to_string(),
            message: "forced verify failure".to_string(),
            recovery: "fix candidate payload".to_string(),
        });
        VerifyResult::Fail
    } else if let Some(path) = input.payload.changed_paths.iter().find(|path| {
        is_table_shape_or_attr_path(path, &fetch.payload.node_path_index)
            || (path_has_ancestor_type(
                path,
                &fetch.payload.node_path_index,
                &["table", "tableRow", "tableCell"],
            ) && !is_table_cell_text_path(path, &fetch.payload.node_path_index))
    }) {
        diagnostics.errors.push(ErrorInfo {
            code: ErrorCode::TableShapeChange.to_string(),
            message: format!("forbidden table shape or attribute mutation at `{path}`"),
            recovery: "limit table updates to cell text paths only".to_string(),
        });
        VerifyResult::Fail
    } else if let Err(error) = ensure_paths_in_scope(
        &input.payload.changed_paths,
        &input.payload.allowed_scope_paths,
    ) {
        diagnostics.errors.push(ErrorInfo {
            code: ErrorCode::OutOfScopeMutation.to_string(),
            message: error.to_string(),
            recovery: "restrict changes to allowed_scope_paths".to_string(),
        });
        VerifyResult::Fail
    } else {
        VerifyResult::Pass
    };

    let output = StateEnvelope {
        meta: meta(request, PipelineState::Verify),
        payload: VerifyOutput {
            verify_result,
            diagnostics: diagnostics.clone(),
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::Verify,
        &input,
        &output,
        &diagnostics,
    )?;
    Ok(output)
}
