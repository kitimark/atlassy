use atlassy_adf::{canonicalize_mapped_path, is_within_allowed_scope, markdown_for_path};
use atlassy_contracts::{
    ClassifyOutput, Diagnostics, ExtractProseInput, ExtractProseOutput, FetchOutput, MarkdownBlock,
    MarkdownMapEntry, PipelineState, StateEnvelope, validate_markdown_mapping,
};

use crate::error_map::to_hard_error;
use crate::util::meta;
use crate::{ArtifactStore, PipelineError, RunRequest, StateTracker};

pub(crate) fn run_extract_prose_state(
    artifact_store: &ArtifactStore,
    request: &RunRequest,
    tracker: &mut StateTracker,
    fetch: &StateEnvelope<FetchOutput>,
    classify: &StateEnvelope<ClassifyOutput>,
) -> Result<StateEnvelope<ExtractProseOutput>, PipelineError> {
    tracker.transition_to(PipelineState::ExtractProse)?;
    let input = StateEnvelope {
        meta: meta(request, PipelineState::ExtractProse),
        payload: ExtractProseInput {
            node_manifest: classify.payload.node_manifest.clone(),
            scoped_adf: fetch.payload.scoped_adf.clone(),
        },
    };

    let mut prose_nodes = Vec::new();
    let mut map_entries = Vec::new();

    for node in classify
        .payload
        .node_manifest
        .iter()
        .filter(|node| node.route == "editable_prose")
        .filter(|node| is_within_allowed_scope(&node.path, &fetch.payload.allowed_scope_paths))
    {
        let canonical_path =
            canonicalize_mapped_path(&node.path, &fetch.payload.allowed_scope_paths)
                .map_err(|error| to_hard_error(PipelineState::ExtractProse, error))?;
        let markdown = markdown_for_path(&fetch.payload.scoped_adf, &node.path)
            .map_err(|error| to_hard_error(PipelineState::ExtractProse, error))?;

        prose_nodes.push(MarkdownBlock {
            md_block_id: canonical_path.clone(),
            markdown,
        });

        map_entries.push(MarkdownMapEntry {
            md_block_id: canonical_path.clone(),
            adf_path: canonical_path,
        });
    }

    prose_nodes.sort_by(|left, right| left.md_block_id.cmp(&right.md_block_id));
    map_entries.sort_by(|left, right| left.md_block_id.cmp(&right.md_block_id));
    let editable_prose_paths = map_entries
        .iter()
        .map(|entry| entry.adf_path.clone())
        .collect::<Vec<_>>();

    validate_markdown_mapping(
        &prose_nodes,
        &map_entries,
        &editable_prose_paths,
        &fetch.payload.allowed_scope_paths,
    )?;

    let output = StateEnvelope {
        meta: meta(request, PipelineState::ExtractProse),
        payload: ExtractProseOutput {
            markdown_blocks: prose_nodes,
            md_to_adf_map: map_entries,
            editable_prose_paths,
        },
    };

    artifact_store.persist_state(
        &request.request_id,
        PipelineState::ExtractProse,
        &input,
        &output,
        &Diagnostics::default(),
    )?;
    Ok(output)
}
