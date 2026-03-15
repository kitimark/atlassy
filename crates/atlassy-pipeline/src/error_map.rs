use atlassy_adf::AdfError;
use atlassy_confluence::ConfluenceError;
use atlassy_contracts::{ContractError, ErrorCode, PipelineState};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("contract error: {0}")]
    Contract(#[from] ContractError),
    #[error("pipeline hard error in `{state}`: {code} ({message})")]
    Hard {
        state: PipelineState,
        code: ErrorCode,
        message: String,
    },
}

pub(crate) fn confluence_error_to_hard_error(
    source_state: PipelineState,
    error: ConfluenceError,
) -> PipelineError {
    match error {
        ConfluenceError::Conflict(page_id) => PipelineError::Hard {
            state: source_state,
            code: ErrorCode::ConflictRetryExhausted,
            message: format!("version conflict on page: {page_id}"),
        },
        ConfluenceError::NotFound(page_id) => PipelineError::Hard {
            state: source_state,
            code: ErrorCode::RuntimeBackend,
            message: format!("page not found in runtime backend: {page_id}"),
        },
        ConfluenceError::Transport(message) => PipelineError::Hard {
            state: source_state,
            code: ErrorCode::RuntimeBackend,
            message,
        },
        ConfluenceError::NotImplemented => PipelineError::Hard {
            state: source_state,
            code: ErrorCode::RuntimeUnmappedHard,
            message: "runtime backend operation is not implemented".to_string(),
        },
    }
}

pub(crate) fn to_hard_error(source_state: PipelineState, error: AdfError) -> PipelineError {
    let message = error.to_string();
    let code = match error {
        AdfError::OutOfScope(_) => ErrorCode::OutOfScopeMutation,
        AdfError::WholeBodyRewriteDisallowed => ErrorCode::RouteViolation,
        AdfError::InsertPositionInvalid(_) => ErrorCode::InsertPositionInvalid,
        AdfError::RemoveTargetNotFound(_) => ErrorCode::RemoveAnchorMissing,
        AdfError::PostMutationInvalid(_) => ErrorCode::PostMutationSchemaInvalid,
        AdfError::OperationConflict(_) => ErrorCode::RouteViolation,
        AdfError::ScopeResolutionFailed => ErrorCode::ScopeMiss,
        AdfError::TargetDiscoveryFailed { .. } => ErrorCode::TargetDiscoveryFailed,
        AdfError::InvalidSelector(_)
        | AdfError::InvalidPath(_)
        | AdfError::DuplicatePath(_)
        | AdfError::MappingIntegrity(_) => ErrorCode::SchemaInvalid,
    };

    PipelineError::Hard {
        state: source_state,
        code,
        message,
    }
}

impl From<AdfError> for PipelineError {
    fn from(error: AdfError) -> Self {
        to_hard_error(PipelineState::Patch, error)
    }
}

impl From<ConfluenceError> for PipelineError {
    fn from(error: ConfluenceError) -> Self {
        confluence_error_to_hard_error(PipelineState::Fetch, error)
    }
}
