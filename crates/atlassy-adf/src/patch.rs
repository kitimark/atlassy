use std::collections::BTreeSet;

use atlassy_contracts::Operation;
use serde_json::Value;

use crate::AdfError;
use crate::path::{is_json_pointer, is_within_allowed_scope};

pub fn normalize_changed_paths(paths: &[String]) -> Result<Vec<String>, AdfError> {
    let mut unique = BTreeSet::new();
    for path in paths {
        if !is_json_pointer(path) {
            return Err(AdfError::InvalidPath(path.clone()));
        }
        unique.insert(path.clone());
    }
    Ok(unique.into_iter().collect())
}

pub fn validate_operations(
    operations: &[Operation],
    allowed_scope_paths: &[String],
) -> Result<(), AdfError> {
    for operation in operations {
        let path = match operation {
            Operation::Replace { path, .. } => path,
        };

        if path == "/" || path.is_empty() {
            return Err(AdfError::WholeBodyRewriteDisallowed);
        }
        if !is_within_allowed_scope(path, allowed_scope_paths) {
            return Err(AdfError::OutOfScope(path.clone()));
        }
    }
    Ok(())
}

pub fn apply_operations(base: &Value, operations: &[Operation]) -> Result<Value, AdfError> {
    let mut candidate = base.clone();
    for operation in operations {
        match operation {
            Operation::Replace { path, value } => {
                if path == "/" || path.is_empty() {
                    return Err(AdfError::WholeBodyRewriteDisallowed);
                }
                let target = candidate.pointer_mut(path).ok_or_else(|| {
                    AdfError::MappingIntegrity(format!("path `{path}` does not resolve"))
                })?;
                *target = value.clone();
            }
        }
    }
    Ok(candidate)
}

pub fn ensure_paths_in_scope(
    paths: &[String],
    allowed_scope_paths: &[String],
) -> Result<(), AdfError> {
    for path in paths {
        if !is_within_allowed_scope(path, allowed_scope_paths) {
            return Err(AdfError::OutOfScope(path.clone()));
        }
    }
    Ok(())
}
