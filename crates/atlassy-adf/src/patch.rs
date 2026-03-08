use std::collections::BTreeSet;

use serde_json::Value;

use crate::path::{is_json_pointer, is_within_allowed_scope};
use crate::{AdfError, PatchCandidate, PatchOperation};

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

pub fn build_patch_ops(
    candidates: &[PatchCandidate],
    allowed_scope_paths: &[String],
) -> Result<Vec<PatchOperation>, AdfError> {
    let mut ops = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if candidate.path == "/" || candidate.path.is_empty() {
            return Err(AdfError::WholeBodyRewriteDisallowed);
        }
        if !is_within_allowed_scope(&candidate.path, allowed_scope_paths) {
            return Err(AdfError::OutOfScope(candidate.path.clone()));
        }
        ops.push(PatchOperation {
            op: "replace".to_string(),
            path: candidate.path.clone(),
            value: candidate.value.clone(),
        });
    }
    Ok(ops)
}

pub fn apply_patch_ops(base: &Value, patch_ops: &[PatchOperation]) -> Result<Value, AdfError> {
    let mut candidate = base.clone();
    for op in patch_ops {
        if op.op != "replace" {
            return Err(AdfError::MappingIntegrity(format!(
                "unsupported patch operation `{}`",
                op.op
            )));
        }
        if op.path == "/" || op.path.is_empty() {
            return Err(AdfError::WholeBodyRewriteDisallowed);
        }
        let target = candidate.pointer_mut(&op.path).ok_or_else(|| {
            AdfError::MappingIntegrity(format!("path `{}` does not resolve", op.path))
        })?;
        *target = op.value.clone();
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
