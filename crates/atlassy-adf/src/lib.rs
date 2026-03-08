use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeResolution {
    pub scoped_adf: Value,
    pub allowed_scope_paths: Vec<String>,
    pub node_path_index: BTreeMap<String, String>,
    pub scope_resolution_failed: bool,
    pub full_page_fetch: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCandidate {
    pub path: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOperation {
    pub op: String,
    pub path: String,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetRoute {
    Prose,
    TableCell,
}

impl std::fmt::Display for TargetRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TargetRoute::Prose => "prose",
            TargetRoute::TableCell => "table_cell",
        };
        f.write_str(value)
    }
}

pub const EDITABLE_PROSE_TYPES: &[&str] = &[
    "paragraph",
    "heading",
    "bulletList",
    "orderedList",
    "listItem",
    "blockquote",
    "codeBlock",
];

/// Node types used as scope anchors. Text under these ancestors is excluded from prose
/// auto-discovery and must be targeted explicitly.
pub const SCOPE_ANCHOR_TYPES: &[&str] = &["heading"];

pub fn document_order_sort(paths: &mut [String]) {
    paths.sort_by(|left, right| compare_path_segments(left, right));
}

fn compare_path_segments(left: &str, right: &str) -> Ordering {
    let mut left_segments = left.split('/');
    let mut right_segments = right.split('/');

    loop {
        match (left_segments.next(), right_segments.next()) {
            (Some(left_segment), Some(right_segment)) => {
                let ordering = match (
                    left_segment.parse::<usize>(),
                    right_segment.parse::<usize>(),
                ) {
                    (Ok(left_number), Ok(right_number)) => left_number.cmp(&right_number),
                    _ => left_segment.cmp(right_segment),
                };
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdfError {
    #[error("scope resolution failed")]
    ScopeResolutionFailed,
    #[error("invalid selector format: {0}")]
    InvalidSelector(String),
    #[error("invalid JSON pointer path: {0}")]
    InvalidPath(String),
    #[error("duplicate path in node index: {0}")]
    DuplicatePath(String),
    #[error("whole-body rewrite is not allowed")]
    WholeBodyRewriteDisallowed,
    #[error("path `{0}` is outside allowed scope")]
    OutOfScope(String),
    #[error("mapping integrity failure: {0}")]
    MappingIntegrity(String),
    #[error("no valid {route} target found in scope at index {index} (found {found})")]
    TargetDiscoveryFailed {
        route: String,
        index: usize,
        found: usize,
    },
}

pub fn resolve_scope(adf: &Value, selectors: &[String]) -> Result<ScopeResolution, AdfError> {
    if selectors.is_empty() {
        return full_page_resolution(adf, Some("no_scope_selectors".to_string()));
    }

    let mut matched_paths = Vec::new();
    let mut matched_heading_count = 0usize;
    let mut expanded_heading_found = false;
    for selector in selectors {
        let (kind, value) = selector
            .split_once(':')
            .ok_or_else(|| AdfError::InvalidSelector(selector.clone()))?;
        match kind {
            "heading" => {
                for heading_path in find_heading_paths(adf, value, String::new()) {
                    matched_heading_count += 1;
                    let expanded = expand_heading_to_section(adf, &heading_path);
                    if expanded.is_empty() {
                        continue;
                    }
                    expanded_heading_found = true;
                    matched_paths.extend(expanded);
                }
            }
            "block" => matched_paths.extend(find_block_paths(adf, value, String::new())),
            _ => return Err(AdfError::InvalidSelector(selector.clone())),
        }
    }

    if matched_heading_count > 0 && !expanded_heading_found && matched_paths.is_empty() {
        return full_page_resolution(adf, Some("nested_heading_scope_unsupported".to_string()));
    }

    if matched_paths.is_empty() {
        return full_page_resolution(adf, Some("scope_selector_not_found".to_string()));
    }

    document_order_sort(&mut matched_paths);
    matched_paths.dedup();

    let node_path_index = build_node_path_index(adf)?;
    Ok(ScopeResolution {
        scoped_adf: adf.clone(),
        allowed_scope_paths: matched_paths,
        node_path_index,
        scope_resolution_failed: false,
        full_page_fetch: false,
        fallback_reason: None,
    })
}

pub fn build_node_path_index(adf: &Value) -> Result<BTreeMap<String, String>, AdfError> {
    let mut index = BTreeMap::new();
    build_node_path_index_inner(adf, String::new(), &mut index)?;
    Ok(index)
}

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

pub fn canonicalize_mapped_path(
    path: &str,
    allowed_scope_paths: &[String],
) -> Result<String, AdfError> {
    if !is_json_pointer(path) {
        return Err(AdfError::InvalidPath(path.to_string()));
    }

    if allowed_scope_paths.iter().any(|allowed| allowed == "/") {
        return Ok(path.to_string());
    }

    if is_within_allowed_scope(path, allowed_scope_paths) {
        return Ok(path.to_string());
    }

    if allowed_scope_paths.len() == 1 {
        let root = allowed_scope_paths[0].trim_end_matches('/');
        if path == "/" {
            return Ok(root.to_string());
        }
        let tail = path.trim_start_matches('/');
        let canonical = format!("{root}/{tail}");
        if is_within_allowed_scope(&canonical, allowed_scope_paths) {
            return Ok(canonical);
        }
    }

    Err(AdfError::OutOfScope(path.to_string()))
}

pub fn is_path_within_or_descendant(path: &str, mapped_path: &str) -> bool {
    path == mapped_path
        || path
            .strip_prefix(mapped_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

pub fn markdown_for_path(adf: &Value, path: &str) -> Result<String, AdfError> {
    if !is_json_pointer(path) {
        return Err(AdfError::InvalidPath(path.to_string()));
    }

    let node = if path == "/" {
        adf
    } else {
        adf.pointer(path)
            .ok_or_else(|| AdfError::MappingIntegrity(format!("path `{path}` does not resolve")))?
    };

    Ok(collect_text(node))
}

pub fn path_has_ancestor_type(
    path: &str,
    node_path_index: &BTreeMap<String, String>,
    candidate_types: &[&str],
) -> bool {
    let mut current = path.to_string();
    while let Some(parent) = parent_path(&current) {
        if let Some(node_type) = node_path_index.get(&parent)
            && candidate_types
                .iter()
                .any(|candidate| *candidate == node_type)
        {
            return true;
        }
        current = parent;
    }
    false
}

pub fn discover_target_path(
    node_path_index: &BTreeMap<String, String>,
    allowed_scope_paths: &[String],
    route: TargetRoute,
    target_index: usize,
) -> Result<String, AdfError> {
    let mut candidates = node_path_index
        .iter()
        .filter(|(_, node_type)| *node_type == "text")
        .map(|(path, _)| path)
        .filter(|path| is_within_allowed_scope(path, allowed_scope_paths))
        .filter(|path| {
            let in_table =
                path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"]);
            match route {
                TargetRoute::Prose => {
                    path_has_ancestor_type(path, node_path_index, EDITABLE_PROSE_TYPES)
                        && !path_has_ancestor_type(path, node_path_index, SCOPE_ANCHOR_TYPES)
                        && !in_table
                }
                TargetRoute::TableCell => in_table,
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    document_order_sort(&mut candidates);
    let found = candidates.len();
    let selected = candidates
        .get(target_index)
        .ok_or_else(|| AdfError::TargetDiscoveryFailed {
            route: route.to_string(),
            index: target_index,
            found,
        })?;

    Ok(format!("{selected}/text"))
}

pub fn is_table_cell_text_path(path: &str, node_path_index: &BTreeMap<String, String>) -> bool {
    path.ends_with("/text")
        && path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"])
}

pub fn is_table_shape_or_attr_path(path: &str, node_path_index: &BTreeMap<String, String>) -> bool {
    if !path_has_ancestor_type(path, node_path_index, &["table", "tableRow", "tableCell"]) {
        return false;
    }

    if path.contains("/attrs") {
        return true;
    }

    !path.ends_with("/text")
}

pub fn is_page_effectively_empty(adf: &Value) -> bool {
    let content = match adf.get("content") {
        Some(Value::Array(arr)) => arr,
        _ => return true,
    };

    if content.is_empty() {
        return true;
    }

    content.iter().all(|node| {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or_default();

        if node_type != "paragraph" {
            return false;
        }

        match node.get("content") {
            None => true,
            Some(Value::Array(children)) if children.is_empty() => true,
            Some(Value::Array(children)) => children.iter().all(|child| {
                let child_type = child
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if child_type != "text" {
                    return false;
                }
                match child.get("text").and_then(Value::as_str) {
                    None => true,
                    Some(text) => text.is_empty(),
                }
            }),
            _ => false,
        }
    })
}

pub fn bootstrap_scaffold() -> Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": ""}]
            },
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": ""}]
            }
        ]
    })
}

fn full_page_resolution(adf: &Value, reason: Option<String>) -> Result<ScopeResolution, AdfError> {
    let node_path_index = build_node_path_index(adf)?;
    Ok(ScopeResolution {
        scoped_adf: adf.clone(),
        allowed_scope_paths: vec!["/".to_string()],
        node_path_index,
        scope_resolution_failed: true,
        full_page_fetch: true,
        fallback_reason: reason,
    })
}

fn heading_level(node: &Value) -> u8 {
    node.get("attrs")
        .and_then(Value::as_object)
        .and_then(|attrs| attrs.get("level"))
        .and_then(Value::as_u64)
        .and_then(|level| u8::try_from(level).ok())
        .filter(|level| (1..=6).contains(level))
        .unwrap_or(6)
}

fn expand_heading_to_section(adf: &Value, heading_path: &str) -> Vec<String> {
    let Some(index_str) = heading_path.strip_prefix("/content/") else {
        return Vec::new();
    };
    if index_str.is_empty() || index_str.contains('/') {
        return Vec::new();
    }

    let Ok(start_index) = index_str.parse::<usize>() else {
        return Vec::new();
    };

    let Some(content) = adf.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };
    let Some(heading_node) = content.get(start_index) else {
        return Vec::new();
    };
    if heading_node.get("type").and_then(Value::as_str) != Some("heading") {
        return Vec::new();
    }

    let base_level = heading_level(heading_node);
    let mut section_paths = vec![heading_path.to_string()];
    for (index, node) in content.iter().enumerate().skip(start_index + 1) {
        if node.get("type").and_then(Value::as_str) == Some("heading")
            && heading_level(node) <= base_level
        {
            break;
        }
        section_paths.push(format!("/content/{index}"));
    }

    section_paths
}

fn build_node_path_index_inner(
    value: &Value,
    path: String,
    index: &mut BTreeMap<String, String>,
) -> Result<(), AdfError> {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(node_type)) = map.get("type") {
                let effective_path = if path.is_empty() {
                    "/".to_string()
                } else {
                    path.clone()
                };
                if index
                    .insert(effective_path.clone(), node_type.clone())
                    .is_some()
                {
                    return Err(AdfError::DuplicatePath(effective_path));
                }
            }
            for (key, child) in map {
                let child_path = format!("{}/{}", path, escape_pointer_segment(key));
                build_node_path_index_inner(child, child_path, index)?;
            }
        }
        Value::Array(list) => {
            for (idx, child) in list.iter().enumerate() {
                let child_path = format!("{}/{}", path, idx);
                build_node_path_index_inner(child, child_path, index)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn find_heading_paths(value: &Value, heading_text: &str, path: String) -> Vec<String> {
    let mut matches = Vec::new();
    if let Value::Object(map) = value {
        if map.get("type") == Some(&Value::String("heading".to_string())) {
            let text = collect_text(value);
            if text == heading_text {
                matches.push(if path.is_empty() {
                    "/".to_string()
                } else {
                    path.clone()
                });
            }
        }
        for (key, child) in map {
            let child_path = format!("{}/{}", path, escape_pointer_segment(key));
            matches.extend(find_heading_paths(child, heading_text, child_path));
        }
    } else if let Value::Array(items) = value {
        for (idx, child) in items.iter().enumerate() {
            let child_path = format!("{}/{}", path, idx);
            matches.extend(find_heading_paths(child, heading_text, child_path));
        }
    }
    matches
}

fn find_block_paths(value: &Value, block_id: &str, path: String) -> Vec<String> {
    let mut matches = Vec::new();
    if let Value::Object(map) = value {
        if let Some(Value::Object(attrs)) = map.get("attrs") {
            let has_match = attrs
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id == block_id)
                || attrs
                    .get("localId")
                    .and_then(Value::as_str)
                    .is_some_and(|id| id == block_id);
            if has_match {
                matches.push(if path.is_empty() {
                    "/".to_string()
                } else {
                    path.clone()
                });
            }
        }
        for (key, child) in map {
            let child_path = format!("{}/{}", path, escape_pointer_segment(key));
            matches.extend(find_block_paths(child, block_id, child_path));
        }
    } else if let Value::Array(items) = value {
        for (idx, child) in items.iter().enumerate() {
            let child_path = format!("{}/{}", path, idx);
            matches.extend(find_block_paths(child, block_id, child_path));
        }
    }
    matches
}

fn collect_text(value: &Value) -> String {
    let mut text = String::new();
    if let Value::Object(map) = value {
        if map.get("type") == Some(&Value::String("text".to_string()))
            && let Some(Value::String(content)) = map.get("text")
        {
            text.push_str(content);
        }
        for child in map.values() {
            text.push_str(&collect_text(child));
        }
    } else if let Value::Array(items) = value {
        for child in items {
            text.push_str(&collect_text(child));
        }
    }
    text
}

pub fn is_within_allowed_scope(path: &str, allowed_scope_paths: &[String]) -> bool {
    allowed_scope_paths.iter().any(|allowed| {
        if allowed == "/" {
            return true;
        }
        path == allowed
            || path
                .strip_prefix(allowed)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

fn is_json_pointer(path: &str) -> bool {
    path.starts_with('/')
}

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn parent_path(path: &str) -> Option<String> {
    if path == "/" {
        return None;
    }
    let (parent, _) = path.rsplit_once('/')?;
    if parent.is_empty() {
        return Some("/".to_string());
    }
    Some(parent.to_string())
}
