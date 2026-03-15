use std::collections::BTreeSet;

use atlassy_contracts::Operation;
use serde_json::Value;

use crate::path::{is_json_pointer, is_within_allowed_scope};
use crate::AdfError;

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
        match operation {
            Operation::Replace { path, .. } => {
                if !is_json_pointer(path) {
                    return Err(AdfError::InvalidPath(path.clone()));
                }
                if path == "/" || path.is_empty() {
                    return Err(AdfError::WholeBodyRewriteDisallowed);
                }
                if !is_within_allowed_scope(path, allowed_scope_paths) {
                    return Err(AdfError::OutOfScope(path.clone()));
                }
            }
            Operation::Insert { parent_path, .. } => {
                if !is_json_pointer(parent_path) {
                    return Err(AdfError::InvalidPath(parent_path.clone()));
                }
                if !is_within_allowed_scope(parent_path, allowed_scope_paths) {
                    return Err(AdfError::OutOfScope(parent_path.clone()));
                }
            }
            Operation::Remove { target_path } => {
                if !is_json_pointer(target_path) {
                    return Err(AdfError::InvalidPath(target_path.clone()));
                }
                if target_path == "/" || target_path.is_empty() {
                    return Err(AdfError::WholeBodyRewriteDisallowed);
                }
                if !is_within_allowed_scope(target_path, allowed_scope_paths) {
                    return Err(AdfError::OutOfScope(target_path.clone()));
                }
            }
        }
    }
    Ok(())
}

pub fn apply_operations(base: &Value, operations: &[Operation]) -> Result<Value, AdfError> {
    let mut candidate = base.clone();
    for operation in operations {
        match operation {
            Operation::Replace { path, value } => {
                if !is_json_pointer(path) {
                    return Err(AdfError::InvalidPath(path.clone()));
                }
                if path == "/" || path.is_empty() {
                    return Err(AdfError::WholeBodyRewriteDisallowed);
                }
                let target = candidate.pointer_mut(path).ok_or_else(|| {
                    AdfError::MappingIntegrity(format!("path `{path}` does not resolve"))
                })?;
                *target = value.clone();
            }
            Operation::Insert {
                parent_path,
                index,
                block,
            } => apply_insert(&mut candidate, parent_path, *index, block)?,
            Operation::Remove { target_path } => apply_remove(&mut candidate, target_path)?,
        }
    }
    Ok(candidate)
}

pub fn apply_insert(
    candidate: &mut Value,
    parent_path: &str,
    index: usize,
    block: &Value,
) -> Result<(), AdfError> {
    let parent = candidate.pointer_mut(parent_path).ok_or_else(|| {
        AdfError::InsertPositionInvalid(format!("parent path `{parent_path}` does not resolve"))
    })?;
    let parent_array = parent.as_array_mut().ok_or_else(|| {
        AdfError::InsertPositionInvalid(format!("parent path `{parent_path}` is not an array"))
    })?;

    if index > parent_array.len() {
        return Err(AdfError::InsertPositionInvalid(format!(
            "index {index} out of bounds for parent `{parent_path}` with length {}",
            parent_array.len()
        )));
    }

    parent_array.insert(index, block.clone());
    Ok(())
}

pub fn apply_remove(candidate: &mut Value, target_path: &str) -> Result<(), AdfError> {
    let (parent_path, index) = split_parent_index(target_path)
        .map_err(|_| AdfError::RemoveTargetNotFound(format!("target path `{target_path}`")))?;

    let parent = candidate.pointer_mut(&parent_path).ok_or_else(|| {
        AdfError::RemoveTargetNotFound(format!(
            "parent path `{parent_path}` does not resolve for target `{target_path}`"
        ))
    })?;

    let parent_array = parent.as_array_mut().ok_or_else(|| {
        AdfError::RemoveTargetNotFound(format!(
            "parent path `{parent_path}` is not an array for target `{target_path}`"
        ))
    })?;

    if index >= parent_array.len() {
        return Err(AdfError::RemoveTargetNotFound(format!(
            "index {index} out of bounds for target `{target_path}` with length {}",
            parent_array.len()
        )));
    }

    parent_array.remove(index);
    Ok(())
}

pub fn split_parent_index(path: &str) -> Result<(String, usize), AdfError> {
    if !is_json_pointer(path) || path == "/" {
        return Err(AdfError::InvalidPath(path.to_string()));
    }

    let (parent, index_segment) = path
        .rsplit_once('/')
        .ok_or_else(|| AdfError::InvalidPath(path.to_string()))?;
    if index_segment.is_empty() {
        return Err(AdfError::InvalidPath(path.to_string()));
    }

    let index = index_segment
        .parse::<usize>()
        .map_err(|_| AdfError::InvalidPath(path.to_string()))?;
    let parent_path = if parent.is_empty() {
        "/".to_string()
    } else {
        parent.to_string()
    };

    Ok((parent_path, index))
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
