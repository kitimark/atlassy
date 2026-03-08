use atlassy_confluence::{ConfluenceClient, ConfluenceError};
use atlassy_contracts::{
    Diagnostics, ErrorCode, ErrorInfo, FetchOutput, PatchOutput, PipelineState, PublishInput,
    PublishOutput, PublishResult, StateEnvelope,
};

use crate::error_map::confluence_error_to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_publish_state<C: ConfluenceClient>(
    client: &mut C,
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
    patch: &StateEnvelope<PatchOutput>,
) -> Result<StateEnvelope<PublishOutput>, PipelineError> {
    tracker.transition_to(PipelineState::Publish)?;

    let input = StateEnvelope {
        meta: meta(request, PipelineState::Publish),
        payload: PublishInput {
            candidate_page_adf: patch.payload.candidate_page_adf.clone(),
            page_version: fetch.payload.page_version,
        },
    };

    let first_attempt = client.publish_page(
        &request.page_id,
        fetch.payload.page_version,
        &patch.payload.candidate_page_adf,
    );

    let (publish_result, new_version, retry_count, diagnostics) = match first_attempt {
        Ok(response) => (
            PublishResult::Published,
            Some(response.new_version),
            0,
            Diagnostics::default(),
        ),
        Err(ConfluenceError::Conflict(_)) => {
            let latest = client
                .fetch_page(&request.page_id)
                .map_err(|error| confluence_error_to_hard_error(PipelineState::Publish, error))?;

            match client.publish_page(
                &request.page_id,
                latest.page_version,
                &patch.payload.candidate_page_adf,
            ) {
                Ok(response) => (
                    PublishResult::Published,
                    Some(response.new_version),
                    1,
                    Diagnostics::default(),
                ),
                Err(ConfluenceError::Conflict(_)) => {
                    let mut diagnostics = Diagnostics::default();
                    diagnostics.errors.push(ErrorInfo {
                        code: ErrorCode::ConflictRetryExhausted.to_string(),
                        message: "conflict after scoped retry".to_string(),
                        recovery: "return reviewer artifact".to_string(),
                    });
                    (PublishResult::Failed, None, 2, diagnostics)
                }
                Err(other) => {
                    return Err(confluence_error_to_hard_error(
                        PipelineState::Publish,
                        other,
                    ));
                }
            }
        }
        Err(other) => {
            return Err(confluence_error_to_hard_error(
                PipelineState::Publish,
                other,
            ));
        }
    };

    let output = StateEnvelope {
        meta: meta(request, PipelineState::Publish),
        payload: PublishOutput {
            publish_result,
            new_version,
            retry_count,
            diagnostics: diagnostics.clone(),
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::Publish,
        &input,
        &output,
        &diagnostics,
    )?;
    Ok(output)
}
