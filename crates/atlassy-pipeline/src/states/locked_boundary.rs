use atlassy_contracts::{ErrorCode, Operation, PipelineState};

use crate::PipelineError;

pub(crate) fn check_locked_boundary(
    operation: &Operation,
    locked_paths: &[&str],
) -> Option<PipelineError> {
    let violating_path = operation_paths(operation).into_iter().find(|path| {
        locked_paths
            .iter()
            .any(|locked_path| path_is_within_locked_boundary(path, locked_path))
    })?;

    Some(PipelineError::Hard {
        state: PipelineState::MergeCandidates,
        code: ErrorCode::RouteViolation,
        message: format!("operation path `{violating_path}` overlaps locked structural boundary"),
    })
}

fn path_is_within_locked_boundary(path: &str, locked_path: &str) -> bool {
    path == locked_path
        || path
            .strip_prefix(locked_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn operation_paths(operation: &Operation) -> Vec<String> {
    match operation {
        Operation::Replace { path, .. } => vec![path.clone()],
        Operation::Insert {
            parent_path, index, ..
        } => vec![format!("{parent_path}/{index}")],
        Operation::Remove { target_path } => vec![target_path.clone()],
    }
}
