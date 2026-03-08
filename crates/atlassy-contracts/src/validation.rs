use std::collections::BTreeMap;

use crate::{
    ContractError, FLOW_BASELINE, FLOW_OPTIMIZED, MarkdownBlock, MarkdownMapEntry, PATTERN_A,
    PATTERN_B, PATTERN_C, ProvenanceStamp, RUNTIME_LIVE, RUNTIME_STUB, RunSummary,
    TableChangeCandidate, TableOperation,
};

pub fn validate_changed_paths(paths: &[String]) -> Result<(), ContractError> {
    let mut prev: Option<&str> = None;
    for path in paths {
        if !is_json_pointer(path) {
            return Err(ContractError::InvalidPath(path.clone()));
        }
        if let Some(previous) = prev
            && previous >= path
        {
            return Err(ContractError::InvalidChangedPaths);
        }
        prev = Some(path);
    }
    Ok(())
}

pub fn normalize_changed_paths(paths: &[String]) -> Result<Vec<String>, ContractError> {
    let mut normalized = paths.to_vec();
    normalized.sort();
    normalized.dedup();
    validate_changed_paths(&normalized)?;
    Ok(normalized)
}

pub fn is_json_pointer(path: &str) -> bool {
    path.starts_with('/')
}

pub fn validate_markdown_mapping(
    markdown_blocks: &[MarkdownBlock],
    md_to_adf_map: &[MarkdownMapEntry],
    editable_prose_paths: &[String],
    allowed_scope_paths: &[String],
) -> Result<(), ContractError> {
    let mut block_seen: BTreeMap<&str, usize> = BTreeMap::new();
    for block in markdown_blocks {
        *block_seen.entry(block.md_block_id.as_str()).or_insert(0) += 1;
    }
    for (block_id, count) in block_seen {
        if count != 1 {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "markdown block `{block_id}` appears {count} times"
            )));
        }
    }

    let mut map_seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut path_seen: BTreeMap<&str, usize> = BTreeMap::new();
    for entry in md_to_adf_map {
        if !is_json_pointer(&entry.adf_path) {
            return Err(ContractError::InvalidPath(entry.adf_path.clone()));
        }
        if !is_within_scope(&entry.adf_path, allowed_scope_paths) {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "mapped path `{}` is outside allowed scope",
                entry.adf_path
            )));
        }

        *map_seen.entry(entry.md_block_id.as_str()).or_insert(0) += 1;
        *path_seen.entry(entry.adf_path.as_str()).or_insert(0) += 1;
    }

    for block in markdown_blocks {
        if !map_seen.contains_key(block.md_block_id.as_str()) {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "missing map entry for markdown block `{}`",
                block.md_block_id
            )));
        }
    }

    for (block_id, count) in map_seen {
        if count != 1 {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "mapping for markdown block `{block_id}` appears {count} times"
            )));
        }
    }

    for (path, count) in path_seen {
        if count != 1 {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "mapped path `{path}` appears {count} times"
            )));
        }
    }

    for path in editable_prose_paths {
        if !md_to_adf_map.iter().any(|entry| entry.adf_path == *path) {
            return Err(ContractError::InvalidMarkdownMapping(format!(
                "editable prose path `{path}` is missing from map"
            )));
        }
    }

    Ok(())
}

pub fn validate_prose_changed_paths(
    prose_changed_paths: &[String],
    mapped_paths: &[String],
) -> Result<(), ContractError> {
    for path in prose_changed_paths {
        if !mapped_paths.iter().any(|mapped| {
            path == mapped
                || path
                    .strip_prefix(mapped)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        }) {
            return Err(ContractError::UnmappedProsePath(path.clone()));
        }
    }
    Ok(())
}

pub fn validate_table_candidates(
    candidates: &[TableChangeCandidate],
    allowed_ops: &[TableOperation],
) -> Result<(), ContractError> {
    let allowed = allowed_ops.iter().map(|op| op.as_str()).collect::<Vec<_>>();

    let mut previous: Option<&str> = None;
    for candidate in candidates {
        if !allowed.iter().any(|op| *op == candidate.op.as_str()) {
            return Err(ContractError::TableOperationNotAllowed(
                candidate.op.as_str().to_string(),
            ));
        }
        if !is_json_pointer(&candidate.path) {
            return Err(ContractError::InvalidPath(candidate.path.clone()));
        }
        if let Some(prev) = previous
            && prev >= candidate.path.as_str()
        {
            return Err(ContractError::TableCandidateOrder);
        }
        previous = Some(candidate.path.as_str());
    }
    Ok(())
}

pub fn validate_run_summary_telemetry(summary: &RunSummary) -> Result<(), ContractError> {
    if summary.run_id.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete("run_id".to_string()));
    }
    if summary.page_id.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete("page_id".to_string()));
    }
    if !matches!(summary.flow.as_str(), FLOW_BASELINE | FLOW_OPTIMIZED) {
        return Err(ContractError::TelemetryIncomplete("flow".to_string()));
    }
    if !matches!(summary.pattern.as_str(), PATTERN_A | PATTERN_B | PATTERN_C) {
        return Err(ContractError::TelemetryIncomplete("pattern".to_string()));
    }
    if summary.edit_intent_hash.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "edit_intent_hash".to_string(),
        ));
    }
    if summary.start_ts.trim().is_empty()
        || summary.verify_end_ts.trim().is_empty()
        || summary.publish_end_ts.trim().is_empty()
    {
        return Err(ContractError::TelemetryIncomplete("timestamps".to_string()));
    }
    if summary.verify_result.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "verify_result".to_string(),
        ));
    }
    if summary.publish_result.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "publish_result".to_string(),
        ));
    }
    if summary.pipeline_version.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "pipeline_version".to_string(),
        ));
    }
    if !is_valid_git_sha(&summary.git_commit_sha) {
        return Err(ContractError::TelemetryIncomplete(
            "git_commit_sha".to_string(),
        ));
    }
    if !matches!(summary.runtime_mode.as_str(), RUNTIME_STUB | RUNTIME_LIVE) {
        return Err(ContractError::TelemetryIncomplete(
            "runtime_mode".to_string(),
        ));
    }
    if !summary.context_reduction_ratio.is_finite() {
        return Err(ContractError::TelemetryIncomplete(
            "context_reduction_ratio".to_string(),
        ));
    }
    if summary.state_token_usage.is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "state_token_usage".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_provenance_stamp(stamp: &ProvenanceStamp) -> Result<(), ContractError> {
    if !is_valid_git_sha(&stamp.git_commit_sha) {
        return Err(ContractError::TelemetryIncomplete(
            "git_commit_sha".to_string(),
        ));
    }
    if stamp.pipeline_version.trim().is_empty() {
        return Err(ContractError::TelemetryIncomplete(
            "pipeline_version".to_string(),
        ));
    }
    if !matches!(stamp.runtime_mode.as_str(), RUNTIME_STUB | RUNTIME_LIVE) {
        return Err(ContractError::TelemetryIncomplete(
            "runtime_mode".to_string(),
        ));
    }
    Ok(())
}

fn is_valid_git_sha(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_within_scope(path: &str, allowed_scope_paths: &[String]) -> bool {
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
