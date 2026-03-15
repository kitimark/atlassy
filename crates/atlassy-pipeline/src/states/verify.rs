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
    let verify_result = check_forced_fail(request.force_verify_fail, &mut diagnostics)
        .or_else(|| {
            check_table_shape_integrity(
                &input.payload.changed_paths,
                &fetch.payload.node_path_index,
                &mut diagnostics,
            )
        })
        .or_else(|| {
            check_scope_containment(
                &input.payload.changed_paths,
                &input.payload.allowed_scope_paths,
                &mut diagnostics,
            )
        })
        .unwrap_or(VerifyResult::Pass);

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

fn check_forced_fail(
    force_verify_fail: bool,
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    if !force_verify_fail {
        return None;
    }

    diagnostics.errors.push(ErrorInfo {
        code: ErrorCode::SchemaInvalid.to_string(),
        message: "forced verify failure".to_string(),
        recovery: "fix candidate payload".to_string(),
    });
    Some(VerifyResult::Fail)
}

fn check_table_shape_integrity(
    changed_paths: &[String],
    node_path_index: &std::collections::BTreeMap<String, String>,
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    let violating_path = changed_paths.iter().find(|path| {
        is_table_shape_or_attr_path(path, node_path_index)
            || (path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"])
                && !is_table_cell_text_path(path, node_path_index))
    })?;

    diagnostics.errors.push(ErrorInfo {
        code: ErrorCode::TableShapeChange.to_string(),
        message: format!("forbidden table shape or attribute mutation at `{violating_path}`"),
        recovery: "limit table updates to cell text paths only".to_string(),
    });
    Some(VerifyResult::Fail)
}

fn check_scope_containment(
    changed_paths: &[String],
    allowed_scope_paths: &[String],
    diagnostics: &mut Diagnostics,
) -> Option<VerifyResult> {
    let error = ensure_paths_in_scope(changed_paths, allowed_scope_paths).err()?;
    diagnostics.errors.push(ErrorInfo {
        code: ErrorCode::OutOfScopeMutation.to_string(),
        message: error.to_string(),
        recovery: "restrict changes to allowed_scope_paths".to_string(),
    });
    Some(VerifyResult::Fail)
}
