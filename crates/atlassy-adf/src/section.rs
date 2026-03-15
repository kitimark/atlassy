use serde_json::Value;

use crate::{AdfError, split_parent_index};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionRange {
    pub heading_index: usize,
    pub end_index: usize,
    pub block_count: usize,
    pub block_paths: Vec<String>,
}

pub fn find_section_range(adf: &Value, heading_path: &str) -> Result<SectionRange, AdfError> {
    let (parent_path, heading_index) = split_parent_index(heading_path).map_err(|_| {
        AdfError::SectionBoundaryInvalid(format!("invalid heading path `{heading_path}`"))
    })?;

    let parent = adf.pointer(&parent_path).ok_or_else(|| {
        AdfError::SectionBoundaryInvalid(format!(
            "parent path `{parent_path}` does not resolve for heading `{heading_path}`"
        ))
    })?;
    let siblings = parent.as_array().ok_or_else(|| {
        AdfError::SectionBoundaryInvalid(format!(
            "parent path `{parent_path}` is not an array for heading `{heading_path}`"
        ))
    })?;

    let heading_node = siblings.get(heading_index).ok_or_else(|| {
        AdfError::SectionBoundaryInvalid(format!(
            "heading index {heading_index} out of bounds for `{parent_path}`"
        ))
    })?;
    if heading_node.get("type").and_then(Value::as_str) != Some("heading") {
        return Err(AdfError::SectionBoundaryInvalid(format!(
            "target path `{heading_path}` is not a heading"
        )));
    }
    let base_heading_level = heading_level(heading_node).map_err(|_| {
        AdfError::SectionBoundaryInvalid(format!(
            "heading at `{heading_path}` has invalid attrs.level"
        ))
    })?;

    let mut end_index = siblings.len();
    for (idx, node) in siblings.iter().enumerate().skip(heading_index + 1) {
        if node.get("type").and_then(Value::as_str) != Some("heading") {
            continue;
        }

        let next_heading_level = heading_level(node).map_err(|_| {
            AdfError::SectionBoundaryInvalid(format!(
                "heading at `{parent_path}/{idx}` has invalid attrs.level"
            ))
        })?;
        if next_heading_level <= base_heading_level {
            end_index = idx;
            break;
        }
    }

    let block_paths = (heading_index..end_index)
        .map(|idx| format!("{parent_path}/{idx}"))
        .collect::<Vec<_>>();

    Ok(SectionRange {
        heading_index,
        end_index,
        block_count: block_paths.len(),
        block_paths,
    })
}

fn heading_level(node: &Value) -> Result<u8, AdfError> {
    let level = node
        .get("attrs")
        .and_then(Value::as_object)
        .and_then(|attrs| attrs.get("level"))
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or_else(|| AdfError::SectionBoundaryInvalid("missing attrs.level".to_string()))?;
    if !(1..=6).contains(&level) {
        return Err(AdfError::SectionBoundaryInvalid(format!(
            "heading level must be in [1, 6], got {level}"
        )));
    }

    Ok(level)
}
