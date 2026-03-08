use atlassy_contracts::{
    AdfTableEditOutput, ClassifyOutput, Diagnostics, ErrorCode, MdAssistEditOutput,
    MergeCandidatesInput, MergeCandidatesOutput, PipelineState, StateEnvelope,
};

use crate::error_map::to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_merge_candidates_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    classify: &StateEnvelope<ClassifyOutput>,
    md_edit: &StateEnvelope<MdAssistEditOutput>,
    table_edit: &StateEnvelope<AdfTableEditOutput>,
) -> Result<StateEnvelope<MergeCandidatesOutput>, PipelineError> {
    tracker.transition_to(PipelineState::MergeCandidates)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::MergeCandidates),
        payload: MergeCandidatesInput {
            prose_changed_paths: md_edit.payload.prose_changed_paths.clone(),
            table_changed_paths: table_edit.payload.table_changed_paths.clone(),
        },
    };

    for prose_path in &input.payload.prose_changed_paths {
        for table_path in &input.payload.table_changed_paths {
            if prose_path == table_path {
                return Err(PipelineError::Hard {
                    state: PipelineState::MergeCandidates,
                    code: ErrorCode::RouteViolation,
                    message: format!(
                        "merge collision: duplicate changed path across routes `{prose_path}`"
                    ),
                });
            }
            if paths_overlap(prose_path, table_path) {
                return Err(PipelineError::Hard {
                    state: PipelineState::MergeCandidates,
                    code: ErrorCode::RouteViolation,
                    message: format!(
                        "cross-route conflict: prose path `{prose_path}` overlaps table path `{table_path}`"
                    ),
                });
            }
        }
    }

    let locked_paths = classify
        .payload
        .node_manifest
        .iter()
        .filter(|node| {
            node.route == "locked_structural"
                && node.path != "/"
                && node.node_type.as_str() != "doc"
        })
        .map(|node| node.path.as_str())
        .collect::<Vec<_>>();

    for table_path in &input.payload.table_changed_paths {
        if locked_paths
            .iter()
            .any(|locked_path| paths_overlap(table_path, locked_path))
        {
            return Err(PipelineError::Hard {
                state: PipelineState::MergeCandidates,
                code: ErrorCode::RouteViolation,
                message: format!(
                    "table candidate path `{table_path}` overlaps locked structural boundary"
                ),
            });
        }
    }

    let mut merged = input.payload.prose_changed_paths.clone();
    merged.extend(input.payload.table_changed_paths.clone());
    let changed_paths = atlassy_adf::normalize_changed_paths(&merged)
        .map_err(|error| to_hard_error(PipelineState::MergeCandidates, error))?;

    let output = StateEnvelope {
        meta: meta(request, PipelineState::MergeCandidates),
        payload: MergeCandidatesOutput { changed_paths },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::MergeCandidates,
        &input,
        &output,
        &Diagnostics::default(),
    )?;
    Ok(output)
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
}
