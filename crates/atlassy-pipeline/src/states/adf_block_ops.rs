use atlassy_adf::{is_within_allowed_scope, AdfError, EDITABLE_PROSE_TYPES};
use atlassy_contracts::{
    BlockOp, Diagnostics, FetchOutput, Operation, PipelineState, StateEnvelope,
};
use serde::{Deserialize, Serialize};

use crate::error_map::to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdfBlockOpsInput {
    block_ops: Vec<BlockOp>,
    allowed_scope_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct AdfBlockOpsOutput {
    pub operations: Vec<Operation>,
}

pub(crate) fn run_adf_block_ops_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
) -> Result<StateEnvelope<AdfBlockOpsOutput>, PipelineError> {
    tracker.transition_to(PipelineState::AdfBlockOps)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::AdfBlockOps),
        payload: AdfBlockOpsInput {
            block_ops: request.block_ops.clone(),
            allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
        },
    };

    let operations = request
        .block_ops
        .iter()
        .map(|block_op| translate_block_op(block_op, &fetch.payload.allowed_scope_paths))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| to_hard_error(PipelineState::AdfBlockOps, error))?;

    let output = StateEnvelope {
        meta: meta(request, PipelineState::AdfBlockOps),
        payload: AdfBlockOpsOutput { operations },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::AdfBlockOps,
        &input,
        &output,
        &Diagnostics::default(),
    )?;

    Ok(output)
}

fn translate_block_op(
    block_op: &BlockOp,
    allowed_scope_paths: &[String],
) -> Result<Operation, AdfError> {
    match block_op {
        BlockOp::Insert {
            parent_path,
            index,
            block,
        } => {
            if !is_within_allowed_scope(parent_path, allowed_scope_paths) {
                return Err(AdfError::OutOfScope(parent_path.clone()));
            }

            let block_type = block.get("type").and_then(serde_json::Value::as_str);
            if !block_type.is_some_and(|value| EDITABLE_PROSE_TYPES.contains(&value)) {
                return Err(AdfError::InsertPositionInvalid(format!(
                    "insert block type must be one of {:?}, got {:?}",
                    EDITABLE_PROSE_TYPES, block_type
                )));
            }

            Ok(Operation::Insert {
                parent_path: parent_path.clone(),
                index: *index,
                block: block.clone(),
            })
        }
        BlockOp::Remove { target_path } => {
            if !is_within_allowed_scope(target_path, allowed_scope_paths) {
                return Err(AdfError::OutOfScope(target_path.clone()));
            }

            Ok(Operation::Remove {
                target_path: target_path.clone(),
            })
        }
    }
}
