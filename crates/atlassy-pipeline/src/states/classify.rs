use std::collections::BTreeMap;

use atlassy_adf::EDITABLE_PROSE_TYPES;
use atlassy_contracts::{
    ClassifyInput, ClassifyOutput, Diagnostics, FetchOutput, PipelineState, StateEnvelope,
};

use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_classify_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
) -> Result<StateEnvelope<ClassifyOutput>, PipelineError> {
    tracker.transition_to(PipelineState::Classify)?;
    let input = StateEnvelope {
        meta: meta(request, PipelineState::Classify),
        payload: ClassifyInput {
            scoped_adf: fetch.payload.scoped_adf.clone(),
        },
    };

    let manifest = fetch
        .payload
        .node_path_index
        .iter()
        .map(|(path, node_type)| atlassy_contracts::NodeRef {
            path: path.clone(),
            node_type: node_type.clone(),
            route: route_for_node(path, node_type, &fetch.payload.node_path_index).to_string(),
        })
        .collect();

    let output = StateEnvelope {
        meta: meta(request, PipelineState::Classify),
        payload: ClassifyOutput {
            node_manifest: manifest,
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::Classify,
        &input,
        &output,
        &Diagnostics::default(),
    )?;
    Ok(output)
}

fn route_for_node(
    path: &str,
    node_type: &str,
    node_path_index: &BTreeMap<String, String>,
) -> &'static str {
    if matches!(node_type, "table" | "tableRow" | "tableCell")
        || has_table_ancestor(path, node_path_index)
    {
        return "table_adf";
    }

    if EDITABLE_PROSE_TYPES.contains(&node_type) {
        "editable_prose"
    } else {
        "locked_structural"
    }
}

fn has_table_ancestor(path: &str, node_path_index: &BTreeMap<String, String>) -> bool {
    let mut current = path.to_string();
    while let Some(parent) = parent_path(&current) {
        if let Some(node_type) = node_path_index.get(&parent)
            && matches!(node_type.as_str(), "table" | "tableRow" | "tableCell")
        {
            return true;
        }
        current = parent;
    }
    false
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
