use atlassy_contracts::{BlockOp, Diagnostics, Operation, PipelineState, StateEnvelope};
use serde::{Deserialize, Serialize};

use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdfBlockOpsInput {
    block_ops: Vec<BlockOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct AdfBlockOpsOutput {
    pub operations: Vec<Operation>,
}

pub(crate) fn run_adf_block_ops_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
) -> Result<StateEnvelope<AdfBlockOpsOutput>, PipelineError> {
    tracker.transition_to(PipelineState::AdfBlockOps)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::AdfBlockOps),
        payload: AdfBlockOpsInput {
            block_ops: request.block_ops.clone(),
        },
    };

    let output = StateEnvelope {
        meta: meta(request, PipelineState::AdfBlockOps),
        payload: AdfBlockOpsOutput {
            operations: Vec::new(),
        },
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
