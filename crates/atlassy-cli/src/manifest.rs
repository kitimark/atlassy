use std::collections::{BTreeMap, BTreeSet};

use atlassy_contracts::{FLOW_BASELINE, FLOW_OPTIMIZED, PATTERN_A, PATTERN_B, PATTERN_C};
use atlassy_pipeline::RunMode;

use crate::{DynError, ManifestMode, ManifestRunEntry, RunManifest};

pub(crate) fn normalize_manifest(manifest: &mut RunManifest) {
    manifest.batch.required_scenario_ids.sort();
    manifest.batch.required_scenario_ids.dedup();
    manifest.batch.observed_scenario_ids.sort();
    manifest.batch.observed_scenario_ids.dedup();

    manifest.runs.sort_by(|left, right| {
        (
            left.page_id.as_str(),
            left.pattern.as_str(),
            left.edit_intent_hash.as_str(),
            left.flow.as_str(),
            left.run_id.as_str(),
        )
            .cmp(&(
                right.page_id.as_str(),
                right.pattern.as_str(),
                right.edit_intent_hash.as_str(),
                right.flow.as_str(),
                right.run_id.as_str(),
            ))
    });

    for run in &mut manifest.runs {
        run.scenario_ids.sort();
        run.scenario_ids.dedup();
    }
}

pub(crate) fn run_mode_from_manifest(entry: &ManifestRunEntry) -> RunMode {
    match entry.mode {
        ManifestMode::NoOp => RunMode::NoOp,
        ManifestMode::SimpleScopedUpdate => RunMode::SimpleScopedUpdate {
            target_path: entry
                .target_path
                .clone()
                .unwrap_or_else(|| "/content/1/content/0/text".to_string()),
            new_value: serde_json::json!(
                entry
                    .new_value
                    .clone()
                    .unwrap_or_else(|| "Updated text".to_string())
            ),
        },
        ManifestMode::SimpleScopedProseUpdate => RunMode::SimpleScopedProseUpdate {
            target_path: entry.target_path.clone(),
            markdown: entry
                .new_value
                .clone()
                .unwrap_or_else(|| "Updated prose body".to_string()),
        },
        ManifestMode::SimpleScopedTableCellUpdate => RunMode::SimpleScopedTableCellUpdate {
            target_path: entry.target_path.clone(),
            text: entry
                .new_value
                .clone()
                .unwrap_or_else(|| "Updated table cell".to_string()),
        },
    }
}

pub(crate) fn validate_manifest(manifest: &RunManifest) -> Result<(), DynError> {
    if manifest.runs.is_empty() {
        return Err("manifest must contain at least one run".into());
    }

    let mut run_ids = BTreeSet::new();
    let mut pair_map: BTreeMap<(String, String, String), BTreeSet<String>> = BTreeMap::new();

    for run in &manifest.runs {
        for (field, value) in [
            ("run_id", run.run_id.trim()),
            ("page_id", run.page_id.trim()),
            ("pattern", run.pattern.trim()),
            ("flow", run.flow.trim()),
            ("edit_intent", run.edit_intent.trim()),
            ("edit_intent_hash", run.edit_intent_hash.trim()),
            ("timestamp", run.timestamp.trim()),
        ] {
            if value.is_empty() {
                return Err(format!("manifest run missing required field `{field}`").into());
            }
        }

        if !run_ids.insert(run.run_id.clone()) {
            return Err(format!("duplicate run_id found: {}", run.run_id).into());
        }

        if !matches!(run.flow.as_str(), FLOW_BASELINE | FLOW_OPTIMIZED) {
            return Err(format!("invalid flow `{}`: expected baseline|optimized", run.flow).into());
        }
        if !matches!(run.pattern.as_str(), PATTERN_A | PATTERN_B | PATTERN_C) {
            return Err(format!("invalid pattern `{}`: expected A|B|C", run.pattern).into());
        }

        pair_map
            .entry((
                run.page_id.clone(),
                run.pattern.clone(),
                run.edit_intent_hash.clone(),
            ))
            .or_default()
            .insert(run.flow.clone());
    }

    for (key, flows) in pair_map {
        if !flows.contains(FLOW_BASELINE) || !flows.contains(FLOW_OPTIMIZED) {
            return Err(format!(
                "unmatched pair for key (page_id={}, pattern={}, edit_intent_hash={})",
                key.0, key.1, key.2
            )
            .into());
        }
    }

    Ok(())
}

pub(crate) fn observed_scenario_ids(manifest: &RunManifest) -> Vec<String> {
    let mut observed = manifest.batch.observed_scenario_ids.clone();
    for run in &manifest.runs {
        observed.extend(run.scenario_ids.clone());
    }
    observed.sort();
    observed.dedup();
    observed
}
