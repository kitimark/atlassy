use atlassy_adf::{
    TargetRoute, canonicalize_mapped_path, discover_target_path, is_path_within_or_descendant,
    normalize_changed_paths,
};
use atlassy_contracts::{
    Diagnostics, ErrorCode, ExtractProseOutput, FetchOutput, MdAssistEditInput, MdAssistEditOutput,
    PipelineState, ProseChangeCandidate, RunSummary, StateEnvelope, validate_markdown_mapping,
    validate_prose_changed_paths,
};

use crate::error_map::to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunMode, RunRequest, StateTracker};

pub(crate) fn run_md_assist_edit_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
    extract: &StateEnvelope<ExtractProseOutput>,
    summary: &mut RunSummary,
) -> Result<StateEnvelope<MdAssistEditOutput>, PipelineError> {
    tracker.transition_to(PipelineState::MdAssistEdit)?;
    let input = StateEnvelope {
        meta: meta(request, PipelineState::MdAssistEdit),
        payload: MdAssistEditInput {
            markdown_blocks: extract.payload.markdown_blocks.clone(),
            md_to_adf_map: extract.payload.md_to_adf_map.clone(),
            editable_prose_paths: extract.payload.editable_prose_paths.clone(),
            allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
            edit_intent: request.edit_intent.clone(),
        },
    };

    validate_markdown_mapping(
        &input.payload.markdown_blocks,
        &input.payload.md_to_adf_map,
        &input.payload.editable_prose_paths,
        &input.payload.allowed_scope_paths,
    )?;

    let mut edited_markdown_blocks = input.payload.markdown_blocks.clone();
    let mut prose_changed_paths = Vec::new();
    let mut prose_change_candidates = Vec::new();

    match &request.run_mode {
        RunMode::NoOp => {}
        RunMode::SimpleScopedUpdate {
            target_path,
            new_value,
        } => {
            let markdown = new_value
                .as_str()
                .map(ToString::to_string)
                .unwrap_or_else(|| new_value.to_string());
            project_prose_candidate(
                target_path,
                &markdown,
                &input.payload.editable_prose_paths,
                &input.payload.allowed_scope_paths,
                &mut prose_changed_paths,
                &mut prose_change_candidates,
            )?;
        }
        RunMode::SimpleScopedProseUpdate {
            target_path,
            markdown,
        } => {
            let resolved_target_path = if let Some(path) = target_path {
                path.clone()
            } else {
                let discovered_path = discover_target_path(
                    &fetch.payload.node_path_index,
                    &input.payload.allowed_scope_paths,
                    TargetRoute::Prose,
                    request.target_index,
                )
                .map_err(|error| PipelineError::Hard {
                    state: PipelineState::MdAssistEdit,
                    code: ErrorCode::TargetDiscoveryFailed,
                    message: error.to_string(),
                })?;
                summary.discovered_target_path = Some(discovered_path.clone());
                discovered_path
            };

            project_prose_candidate(
                &resolved_target_path,
                markdown,
                &input.payload.editable_prose_paths,
                &input.payload.allowed_scope_paths,
                &mut prose_changed_paths,
                &mut prose_change_candidates,
            )?;
        }
        RunMode::SimpleScopedTableCellUpdate { .. } => {}
        RunMode::ForbiddenTableOperation { .. } => {}
        RunMode::SyntheticRouteConflict { table_path, .. } => {
            prose_changed_paths.push(table_path.clone());
            prose_change_candidates.push(ProseChangeCandidate {
                path: table_path.clone(),
                markdown: "Synthetic prose conflict candidate".to_string(),
            });
        }
        RunMode::SyntheticTableShapeDrift { .. } => {}
    }

    prose_changed_paths = normalize_changed_paths(&prose_changed_paths)
        .map_err(|error| to_hard_error(PipelineState::MdAssistEdit, error))?;
    if !matches!(request.run_mode, RunMode::SyntheticRouteConflict { .. }) {
        validate_prose_changed_paths(&prose_changed_paths, &input.payload.editable_prose_paths)?;
    }

    for candidate in &prose_change_candidates {
        if let Some(block) = edited_markdown_blocks
            .iter_mut()
            .find(|block| is_path_within_or_descendant(&candidate.path, &block.md_block_id))
        {
            block.markdown = candidate.markdown.clone();
        }
    }

    let output = StateEnvelope {
        meta: meta(request, PipelineState::MdAssistEdit),
        payload: MdAssistEditOutput {
            edited_markdown_blocks,
            prose_changed_paths,
            prose_change_candidates,
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::MdAssistEdit,
        &input,
        &output,
        &Diagnostics::default(),
    )?;
    Ok(output)
}

fn project_prose_candidate(
    target_path: &str,
    markdown: &str,
    editable_prose_paths: &[String],
    allowed_scope_paths: &[String],
    prose_changed_paths: &mut Vec<String>,
    prose_change_candidates: &mut Vec<ProseChangeCandidate>,
) -> Result<(), PipelineError> {
    let canonical_path = canonicalize_mapped_path(target_path, allowed_scope_paths)
        .map_err(|error| to_hard_error(PipelineState::MdAssistEdit, error))?;

    let mapped_root = editable_prose_paths
        .iter()
        .find(|path| is_path_within_or_descendant(&canonical_path, path))
        .cloned()
        .ok_or_else(|| PipelineError::Hard {
            state: PipelineState::MdAssistEdit,
            code: ErrorCode::RouteViolation,
            message: format!("target path `{canonical_path}` is not mapped to editable prose"),
        })?;

    if canonical_path == mapped_root || canonical_path.ends_with("/type") {
        return Err(PipelineError::Hard {
            state: PipelineState::MdAssistEdit,
            code: ErrorCode::SchemaInvalid,
            message: format!(
                "target path `{canonical_path}` violates prose boundary or top-level type constraints"
            ),
        });
    }

    prose_changed_paths.push(canonical_path.clone());
    prose_change_candidates.push(ProseChangeCandidate {
        path: canonical_path,
        markdown: markdown.to_string(),
    });
    Ok(())
}
