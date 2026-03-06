use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use atlassy_adf::{
    AdfError, PatchCandidate, build_patch_ops, canonicalize_mapped_path, ensure_paths_in_scope,
    is_path_within_or_descendant, is_table_cell_text_path, is_table_shape_or_attr_path,
    markdown_for_path, normalize_changed_paths, path_has_ancestor_type, resolve_scope,
};
use atlassy_confluence::{ConfluenceClient, ConfluenceError};
use atlassy_contracts::{
    AdfTableEditInput, AdfTableEditOutput, ClassifyInput, ClassifyOutput, ContractError,
    Diagnostics, ERR_CONFLICT_RETRY_EXHAUSTED, ERR_OUT_OF_SCOPE_MUTATION, ERR_ROUTE_VIOLATION,
    ERR_SCHEMA_INVALID, ERR_TABLE_SHAPE_CHANGE, EnvelopeMeta, ErrorInfo, ExtractProseInput,
    ExtractProseOutput, FetchInput, FetchOutput, MarkdownBlock, MarkdownMapEntry,
    MdAssistEditInput, MdAssistEditOutput, MergeCandidatesInput, MergeCandidatesOutput, PatchInput,
    PatchOp, PatchOutput, PipelineState, ProseChangeCandidate, PublishInput, PublishOutput,
    PublishResult, RunSummary, StateEnvelope, TableChangeCandidate, TableOperation, VerifyInput,
    VerifyOutput, VerifyResult, validate_markdown_mapping, validate_prose_changed_paths,
    validate_table_candidates,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum RunMode {
    NoOp,
    SimpleScopedUpdate {
        target_path: String,
        new_value: serde_json::Value,
    },
    SimpleScopedProseUpdate {
        target_path: String,
        markdown: String,
    },
    SimpleScopedTableCellUpdate {
        target_path: String,
        text: String,
    },
    ForbiddenTableOperation {
        target_path: String,
        operation: TableOperation,
    },
    SyntheticRouteConflict {
        prose_path: String,
        table_path: String,
    },
    SyntheticTableShapeDrift {
        path: String,
    },
}

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub request_id: String,
    pub page_id: String,
    pub edit_intent: String,
    pub scope_selectors: Vec<String>,
    pub timestamp: String,
    pub run_mode: RunMode,
    pub force_verify_fail: bool,
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("contract error: {0}")]
    Contract(#[from] ContractError),
    #[error("pipeline hard error in `{state}`: {code} ({message})")]
    Hard {
        state: PipelineState,
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct StateTracker {
    current: Option<PipelineState>,
}

impl StateTracker {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn transition_to(&mut self, next: PipelineState) -> Result<(), ContractError> {
        let expected = PipelineState::expected_next(self.current)
            .map(|state| state.as_str().to_string())
            .unwrap_or_else(|| "<done>".to_string());
        if expected != next.as_str() {
            return Err(ContractError::InvalidTransition {
                expected,
                actual: next.as_str().to_string(),
            });
        }
        self.current = Some(next);
        Ok(())
    }
}

impl Default for StateTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ArtifactStore {
    root: PathBuf,
}

impl ArtifactStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn persist_state<TInput: serde::Serialize, TOutput: serde::Serialize>(
        &self,
        run_id: &str,
        state: PipelineState,
        input: &StateEnvelope<TInput>,
        output: &StateEnvelope<TOutput>,
        diagnostics: &Diagnostics,
    ) -> Result<(), PipelineError> {
        let state_dir = self
            .root
            .join("artifacts")
            .join(run_id)
            .join(state.as_str());
        fs::create_dir_all(&state_dir)?;

        let input_file = state_dir.join("state_input.json");
        let output_file = state_dir.join("state_output.json");
        let diag_file = state_dir.join("diagnostics.json");

        fs::write(input_file, serde_json::to_string_pretty(input)?)?;
        fs::write(output_file, serde_json::to_string_pretty(output)?)?;
        fs::write(diag_file, serde_json::to_string_pretty(diagnostics)?)?;
        Ok(())
    }

    pub fn persist_summary(&self, run_id: &str, summary: &RunSummary) -> Result<(), PipelineError> {
        let run_dir = self.root.join("artifacts").join(run_id);
        fs::create_dir_all(&run_dir)?;
        fs::write(
            run_dir.join("summary.json"),
            serde_json::to_string_pretty(summary)?,
        )?;
        Ok(())
    }
}

pub struct Orchestrator<C: ConfluenceClient> {
    client: C,
    artifact_store: ArtifactStore,
}

impl<C: ConfluenceClient> Orchestrator<C> {
    pub fn new(client: C, artifact_root: impl AsRef<Path>) -> Self {
        Self {
            client,
            artifact_store: ArtifactStore::new(artifact_root),
        }
    }

    pub fn client(&self) -> &C {
        &self.client
    }

    pub fn client_mut(&mut self) -> &mut C {
        &mut self.client
    }

    pub fn run(&mut self, request: RunRequest) -> Result<RunSummary, PipelineError> {
        let mut tracker = StateTracker::new();
        let mut run_summary = RunSummary {
            success: false,
            request_id: request.request_id.clone(),
            page_id: request.page_id.clone(),
            pipeline_version: atlassy_contracts::PIPELINE_VERSION.to_string(),
            applied_paths: Vec::new(),
            blocked_paths: Vec::new(),
            error_codes: Vec::new(),
            token_metrics: BTreeMap::new(),
            failure_state: None,
        };

        let result = self.run_internal(&request, &mut tracker, &mut run_summary);
        if result.is_ok() {
            run_summary.success = true;
        }
        self.artifact_store
            .persist_summary(&request.request_id, &run_summary)?;

        match result {
            Ok(()) => Ok(run_summary),
            Err(error) => Err(error),
        }
    }

    fn run_internal(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
        summary: &mut RunSummary,
    ) -> Result<(), PipelineError> {
        let fetch = self
            .run_fetch_state(request, tracker)
            .map_err(|error| self.hard_fail(summary, PipelineState::Fetch, error))?;

        let classify = self
            .run_classify_state(request, tracker, &fetch)
            .map_err(|error| self.hard_fail(summary, PipelineState::Classify, error))?;

        let extract = self
            .run_extract_prose_state(request, tracker, &fetch, &classify)
            .map_err(|error| self.hard_fail(summary, PipelineState::ExtractProse, error))?;

        let md_edit = self
            .run_md_assist_edit_state(request, tracker, &fetch, &extract)
            .map_err(|error| self.hard_fail(summary, PipelineState::MdAssistEdit, error))?;

        let table_edit = self
            .run_adf_table_edit_state(request, tracker, &fetch, &classify)
            .map_err(|error| self.hard_fail(summary, PipelineState::AdfTableEdit, error))?;

        let merged = self
            .run_merge_candidates_state(request, tracker, &classify, &md_edit, &table_edit)
            .map_err(|error| self.hard_fail(summary, PipelineState::MergeCandidates, error))?;

        let patch = self
            .run_patch_state(request, tracker, &fetch, &merged, &md_edit, &table_edit)
            .map_err(|error| self.hard_fail(summary, PipelineState::Patch, error))?;

        let verify = self
            .run_verify_state(request, tracker, &fetch, &patch, &merged)
            .map_err(|error| self.hard_fail(summary, PipelineState::Verify, error))?;

        if verify.payload.verify_result == VerifyResult::Fail {
            summary.failure_state = Some(PipelineState::Verify);
            let codes = verify
                .payload
                .diagnostics
                .errors
                .iter()
                .map(|error| error.code.clone())
                .collect::<Vec<_>>();
            summary.error_codes.extend(codes);
            let primary_code = verify
                .payload
                .diagnostics
                .errors
                .first()
                .map(|error| error.code.clone())
                .unwrap_or_else(|| ERR_SCHEMA_INVALID.to_string());
            return Err(PipelineError::Hard {
                state: PipelineState::Verify,
                code: primary_code,
                message: "verification failed".to_string(),
            });
        }

        let publish = self
            .run_publish_state(request, tracker, &fetch, &patch)
            .map_err(|error| self.hard_fail(summary, PipelineState::Publish, error))?;

        if publish.payload.publish_result == PublishResult::Failed {
            summary.failure_state = Some(PipelineState::Publish);
            summary
                .error_codes
                .push(ERR_CONFLICT_RETRY_EXHAUSTED.to_string());
            return Err(PipelineError::Hard {
                state: PipelineState::Publish,
                code: ERR_CONFLICT_RETRY_EXHAUSTED.to_string(),
                message: "publish failed after retry".to_string(),
            });
        }

        summary.applied_paths = merged.payload.changed_paths;
        summary.token_metrics.insert("fetch".to_string(), 0);
        summary.token_metrics.insert("verify".to_string(), 0);
        summary.token_metrics.insert("publish".to_string(), 0);
        Ok(())
    }

    fn run_fetch_state(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
    ) -> Result<StateEnvelope<FetchOutput>, PipelineError> {
        tracker.transition_to(PipelineState::Fetch)?;

        let input = StateEnvelope {
            meta: meta(request, PipelineState::Fetch),
            payload: FetchInput {
                page_id: request.page_id.clone(),
                edit_intent: request.edit_intent.clone(),
                scope_selectors: request.scope_selectors.clone(),
            },
        };
        input.validate_meta()?;

        let page = self
            .client
            .fetch_page(&request.page_id)
            .map_err(|error| to_hard_error(PipelineState::Fetch, error))?;

        let scope = resolve_scope(&page.adf, &request.scope_selectors)
            .map_err(|error| to_hard_error(PipelineState::Fetch, error))?;

        let output = StateEnvelope {
            meta: meta(request, PipelineState::Fetch),
            payload: FetchOutput {
                scoped_adf: scope.scoped_adf,
                page_version: page.page_version,
                allowed_scope_paths: scope.allowed_scope_paths,
                node_path_index: scope.node_path_index,
                scope_resolution_failed: scope.scope_resolution_failed,
                full_page_fetch: scope.full_page_fetch,
                fallback_reason: scope.fallback_reason,
            },
        };

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::Fetch,
            &input,
            &output,
            &Diagnostics::default(),
        )?;

        Ok(output)
    }

    fn run_classify_state(
        &mut self,
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

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::Classify,
            &input,
            &output,
            &Diagnostics::default(),
        )?;
        Ok(output)
    }

    fn run_extract_prose_state(
        &mut self,
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

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::ExtractProse,
            &input,
            &output,
            &Diagnostics::default(),
        )?;
        Ok(output)
    }

    fn run_md_assist_edit_state(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
        fetch: &StateEnvelope<FetchOutput>,
        extract: &StateEnvelope<ExtractProseOutput>,
    ) -> Result<StateEnvelope<MdAssistEditOutput>, PipelineError> {
        tracker.transition_to(PipelineState::MdAssistEdit)?;
        let input = StateEnvelope {
            meta: meta(request, PipelineState::MdAssistEdit),
            payload: MdAssistEditInput {
                markdown_blocks: extract.payload.markdown_blocks.clone(),
                md_to_adf_map: extract.payload.md_to_adf_map.clone(),
                editable_prose_paths: extract.payload.editable_prose_paths.clone(),
                allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
                edit_intent: request.edit_intent.clone(),
            },
        };

        validate_markdown_mapping(
            &input.payload.markdown_blocks,
            &input.payload.md_to_adf_map,
            &input.payload.editable_prose_paths,
            &input.payload.allowed_scope_paths,
        )?;

        let mut edited_markdown_blocks = input.payload.markdown_blocks.clone();
        let mut prose_changed_paths = Vec::new();
        let mut prose_change_candidates = Vec::new();

        match &request.run_mode {
            RunMode::NoOp => {}
            RunMode::SimpleScopedUpdate {
                target_path,
                new_value,
            } => {
                let markdown = new_value
                    .as_str()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| new_value.to_string());
                project_prose_candidate(
                    target_path,
                    &markdown,
                    &input.payload.editable_prose_paths,
                    &input.payload.allowed_scope_paths,
                    &mut prose_changed_paths,
                    &mut prose_change_candidates,
                )?;
            }
            RunMode::SimpleScopedProseUpdate {
                target_path,
                markdown,
            } => {
                project_prose_candidate(
                    target_path,
                    markdown,
                    &input.payload.editable_prose_paths,
                    &input.payload.allowed_scope_paths,
                    &mut prose_changed_paths,
                    &mut prose_change_candidates,
                )?;
            }
            RunMode::SimpleScopedTableCellUpdate { .. } => {}
            RunMode::ForbiddenTableOperation { .. } => {}
            RunMode::SyntheticRouteConflict { table_path, .. } => {
                prose_changed_paths.push(table_path.clone());
                prose_change_candidates.push(ProseChangeCandidate {
                    path: table_path.clone(),
                    markdown: "Synthetic prose conflict candidate".to_string(),
                });
            }
            RunMode::SyntheticTableShapeDrift { .. } => {}
        }

        prose_changed_paths = normalize_changed_paths(&prose_changed_paths)
            .map_err(|error| to_hard_error(PipelineState::MdAssistEdit, error))?;
        if !matches!(request.run_mode, RunMode::SyntheticRouteConflict { .. }) {
            validate_prose_changed_paths(
                &prose_changed_paths,
                &input.payload.editable_prose_paths,
            )?;
        }

        for candidate in &prose_change_candidates {
            if let Some(block) = edited_markdown_blocks
                .iter_mut()
                .find(|block| is_path_within_or_descendant(&candidate.path, &block.md_block_id))
            {
                block.markdown = candidate.markdown.clone();
            }
        }

        let output = StateEnvelope {
            meta: meta(request, PipelineState::MdAssistEdit),
            payload: MdAssistEditOutput {
                edited_markdown_blocks,
                prose_changed_paths,
                prose_change_candidates,
            },
        };

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::MdAssistEdit,
            &input,
            &output,
            &Diagnostics::default(),
        )?;
        Ok(output)
    }

    fn run_adf_table_edit_state(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
        fetch: &StateEnvelope<FetchOutput>,
        classify: &StateEnvelope<ClassifyOutput>,
    ) -> Result<StateEnvelope<AdfTableEditOutput>, PipelineError> {
        tracker.transition_to(PipelineState::AdfTableEdit)?;
        let table_nodes = classify
            .payload
            .node_manifest
            .iter()
            .filter(|node| node.route == "table_adf")
            .cloned()
            .collect::<Vec<_>>();

        let input = StateEnvelope {
            meta: meta(request, PipelineState::AdfTableEdit),
            payload: AdfTableEditInput {
                table_nodes,
                allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
                edit_intent: request.edit_intent.clone(),
            },
        };

        let allowed_ops = vec![TableOperation::CellTextUpdate];
        let mut table_candidates = Vec::new();

        match &request.run_mode {
            RunMode::NoOp | RunMode::SimpleScopedProseUpdate { .. } => {}
            RunMode::SimpleScopedUpdate {
                target_path,
                new_value,
            } => {
                if let Some(text) = new_value.as_str() {
                    project_table_candidate(
                        target_path,
                        text,
                        &fetch.payload.allowed_scope_paths,
                        &fetch.payload.node_path_index,
                        &allowed_ops,
                        &mut table_candidates,
                    )?;
                }
            }
            RunMode::SimpleScopedTableCellUpdate { target_path, text } => {
                project_table_candidate(
                    target_path,
                    text,
                    &fetch.payload.allowed_scope_paths,
                    &fetch.payload.node_path_index,
                    &allowed_ops,
                    &mut table_candidates,
                )?;
            }
            RunMode::ForbiddenTableOperation {
                target_path,
                operation,
            } => {
                if *operation != TableOperation::CellTextUpdate {
                    return Err(PipelineError::Hard {
                        state: PipelineState::AdfTableEdit,
                        code: ERR_TABLE_SHAPE_CHANGE.to_string(),
                        message: format!(
                            "forbidden table operation requested: {} at {}",
                            operation.as_str(),
                            target_path
                        ),
                    });
                }

                project_table_candidate(
                    target_path,
                    "Allowed table operation",
                    &fetch.payload.allowed_scope_paths,
                    &fetch.payload.node_path_index,
                    &allowed_ops,
                    &mut table_candidates,
                )?;
            }
            RunMode::SyntheticRouteConflict { table_path, .. } => {
                project_table_candidate(
                    table_path,
                    "Synthetic table conflict candidate",
                    &fetch.payload.allowed_scope_paths,
                    &fetch.payload.node_path_index,
                    &allowed_ops,
                    &mut table_candidates,
                )?;
            }
            RunMode::SyntheticTableShapeDrift { path } => {
                table_candidates.push(TableChangeCandidate {
                    op: TableOperation::CellTextUpdate,
                    path: path.clone(),
                    value: serde_json::Value::String("Synthetic drift".to_string()),
                    source_route: "table_adf".to_string(),
                });
            }
        }

        table_candidates.sort_by(|left, right| left.path.cmp(&right.path));
        validate_table_candidates(&table_candidates, &allowed_ops)?;

        let table_changed_paths = normalize_changed_paths(
            &table_candidates
                .iter()
                .map(|candidate| candidate.path.clone())
                .collect::<Vec<_>>(),
        )
        .map_err(|error| to_hard_error(PipelineState::AdfTableEdit, error))?;

        let output = StateEnvelope {
            meta: meta(request, PipelineState::AdfTableEdit),
            payload: AdfTableEditOutput {
                table_candidates,
                table_changed_paths,
                allowed_ops,
            },
        };

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::AdfTableEdit,
            &input,
            &output,
            &Diagnostics::default(),
        )?;
        Ok(output)
    }

    fn run_merge_candidates_state(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
        classify: &StateEnvelope<ClassifyOutput>,
        md_edit: &StateEnvelope<MdAssistEditOutput>,
        table_edit: &StateEnvelope<AdfTableEditOutput>,
    ) -> Result<StateEnvelope<MergeCandidatesOutput>, PipelineError> {
        tracker.transition_to(PipelineState::MergeCandidates)?;

        let input = StateEnvelope {
            meta: meta(request, PipelineState::MergeCandidates),
            payload: MergeCandidatesInput {
                prose_changed_paths: md_edit.payload.prose_changed_paths.clone(),
                table_changed_paths: table_edit.payload.table_changed_paths.clone(),
            },
        };

        for prose_path in &input.payload.prose_changed_paths {
            for table_path in &input.payload.table_changed_paths {
                if prose_path == table_path {
                    return Err(PipelineError::Hard {
                        state: PipelineState::MergeCandidates,
                        code: ERR_ROUTE_VIOLATION.to_string(),
                        message: format!(
                            "merge collision: duplicate changed path across routes `{prose_path}`"
                        ),
                    });
                }
                if paths_overlap(prose_path, table_path) {
                    return Err(PipelineError::Hard {
                        state: PipelineState::MergeCandidates,
                        code: ERR_ROUTE_VIOLATION.to_string(),
                        message: format!(
                            "cross-route conflict: prose path `{prose_path}` overlaps table path `{table_path}`"
                        ),
                    });
                }
            }
        }

        let locked_paths = classify
            .payload
            .node_manifest
            .iter()
            .filter(|node| {
                node.route == "locked_structural"
                    && node.path != "/"
                    && node.node_type.as_str() != "doc"
            })
            .map(|node| node.path.as_str())
            .collect::<Vec<_>>();

        for table_path in &input.payload.table_changed_paths {
            if locked_paths
                .iter()
                .any(|locked_path| paths_overlap(table_path, locked_path))
            {
                return Err(PipelineError::Hard {
                    state: PipelineState::MergeCandidates,
                    code: ERR_ROUTE_VIOLATION.to_string(),
                    message: format!(
                        "table candidate path `{table_path}` overlaps locked structural boundary"
                    ),
                });
            }
        }

        let mut merged = input.payload.prose_changed_paths.clone();
        merged.extend(input.payload.table_changed_paths.clone());
        let changed_paths = normalize_changed_paths(&merged)
            .map_err(|error| to_hard_error(PipelineState::MergeCandidates, error))?;

        let output = StateEnvelope {
            meta: meta(request, PipelineState::MergeCandidates),
            payload: MergeCandidatesOutput { changed_paths },
        };

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::MergeCandidates,
            &input,
            &output,
            &Diagnostics::default(),
        )?;
        Ok(output)
    }

    fn run_patch_state(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
        fetch: &StateEnvelope<FetchOutput>,
        merged: &StateEnvelope<MergeCandidatesOutput>,
        md_edit: &StateEnvelope<MdAssistEditOutput>,
        table_edit: &StateEnvelope<AdfTableEditOutput>,
    ) -> Result<StateEnvelope<PatchOutput>, PipelineError> {
        tracker.transition_to(PipelineState::Patch)?;

        let input = StateEnvelope {
            meta: meta(request, PipelineState::Patch),
            payload: PatchInput {
                scoped_adf: fetch.payload.scoped_adf.clone(),
                changed_paths: merged.payload.changed_paths.clone(),
            },
        };

        let candidates = md_edit
            .payload
            .prose_change_candidates
            .iter()
            .map(|candidate| PatchCandidate {
                path: candidate.path.clone(),
                value: serde_json::Value::String(candidate.markdown.clone()),
            })
            .chain(
                table_edit
                    .payload
                    .table_candidates
                    .iter()
                    .map(|candidate| PatchCandidate {
                        path: candidate.path.clone(),
                        value: candidate.value.clone(),
                    }),
            )
            .collect::<Vec<_>>();
        let patch_ops = build_patch_ops(&candidates, &fetch.payload.allowed_scope_paths)
            .map_err(|error| to_hard_error(PipelineState::Patch, error))?
            .into_iter()
            .map(|op| PatchOp {
                op: op.op,
                path: op.path,
                value: op.value,
            })
            .collect::<Vec<_>>();

        let output = StateEnvelope {
            meta: meta(request, PipelineState::Patch),
            payload: PatchOutput {
                patch_ops,
                candidate_page_adf: fetch.payload.scoped_adf.clone(),
            },
        };

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::Patch,
            &input,
            &output,
            &Diagnostics::default(),
        )?;
        Ok(output)
    }

    fn run_verify_state(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
        fetch: &StateEnvelope<FetchOutput>,
        patch: &StateEnvelope<PatchOutput>,
        merged: &StateEnvelope<MergeCandidatesOutput>,
    ) -> Result<StateEnvelope<VerifyOutput>, PipelineError> {
        tracker.transition_to(PipelineState::Verify)?;

        let input = StateEnvelope {
            meta: meta(request, PipelineState::Verify),
            payload: VerifyInput {
                original_scoped_adf: fetch.payload.scoped_adf.clone(),
                candidate_page_adf: patch.payload.candidate_page_adf.clone(),
                allowed_scope_paths: fetch.payload.allowed_scope_paths.clone(),
                changed_paths: merged.payload.changed_paths.clone(),
            },
        };

        let mut diagnostics = Diagnostics::default();
        let verify_result = if request.force_verify_fail {
            diagnostics.errors.push(ErrorInfo {
                code: ERR_SCHEMA_INVALID.to_string(),
                message: "forced verify failure".to_string(),
                recovery: "fix candidate payload".to_string(),
            });
            VerifyResult::Fail
        } else if let Some(path) = input.payload.changed_paths.iter().find(|path| {
            is_table_shape_or_attr_path(path, &fetch.payload.node_path_index)
                || (path_has_ancestor_type(
                    path,
                    &fetch.payload.node_path_index,
                    &["table", "tableRow", "tableCell"],
                ) && !is_table_cell_text_path(path, &fetch.payload.node_path_index))
        }) {
            diagnostics.errors.push(ErrorInfo {
                code: ERR_TABLE_SHAPE_CHANGE.to_string(),
                message: format!("forbidden table shape or attribute mutation at `{path}`"),
                recovery: "limit table updates to cell text paths only".to_string(),
            });
            VerifyResult::Fail
        } else if let Err(error) = ensure_paths_in_scope(
            &input.payload.changed_paths,
            &input.payload.allowed_scope_paths,
        ) {
            diagnostics.errors.push(ErrorInfo {
                code: ERR_OUT_OF_SCOPE_MUTATION.to_string(),
                message: error.to_string(),
                recovery: "restrict changes to allowed_scope_paths".to_string(),
            });
            VerifyResult::Fail
        } else {
            VerifyResult::Pass
        };

        let output = StateEnvelope {
            meta: meta(request, PipelineState::Verify),
            payload: VerifyOutput {
                verify_result,
                diagnostics: diagnostics.clone(),
            },
        };

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::Verify,
            &input,
            &output,
            &diagnostics,
        )?;
        Ok(output)
    }

    fn run_publish_state(
        &mut self,
        request: &RunRequest,
        tracker: &mut StateTracker,
        fetch: &StateEnvelope<FetchOutput>,
        patch: &StateEnvelope<PatchOutput>,
    ) -> Result<StateEnvelope<PublishOutput>, PipelineError> {
        tracker.transition_to(PipelineState::Publish)?;

        let input = StateEnvelope {
            meta: meta(request, PipelineState::Publish),
            payload: PublishInput {
                candidate_page_adf: patch.payload.candidate_page_adf.clone(),
                page_version: fetch.payload.page_version,
            },
        };

        let first_attempt = self.client.publish_page(
            &request.page_id,
            fetch.payload.page_version,
            &patch.payload.candidate_page_adf,
        );

        let (publish_result, new_version, diagnostics) = match first_attempt {
            Ok(response) => (
                PublishResult::Published,
                Some(response.new_version),
                Diagnostics::default(),
            ),
            Err(ConfluenceError::Conflict(_)) => {
                let latest = self
                    .client
                    .fetch_page(&request.page_id)
                    .map_err(|error| to_hard_error(PipelineState::Publish, error))?;

                match self.client.publish_page(
                    &request.page_id,
                    latest.page_version,
                    &patch.payload.candidate_page_adf,
                ) {
                    Ok(response) => (
                        PublishResult::Published,
                        Some(response.new_version),
                        Diagnostics::default(),
                    ),
                    Err(ConfluenceError::Conflict(_)) => {
                        let mut diagnostics = Diagnostics::default();
                        diagnostics.errors.push(ErrorInfo {
                            code: ERR_CONFLICT_RETRY_EXHAUSTED.to_string(),
                            message: "conflict after scoped retry".to_string(),
                            recovery: "return reviewer artifact".to_string(),
                        });
                        (PublishResult::Failed, None, diagnostics)
                    }
                    Err(other) => return Err(to_hard_error(PipelineState::Publish, other)),
                }
            }
            Err(other) => return Err(to_hard_error(PipelineState::Publish, other)),
        };

        let output = StateEnvelope {
            meta: meta(request, PipelineState::Publish),
            payload: PublishOutput {
                publish_result,
                new_version,
                diagnostics: diagnostics.clone(),
            },
        };

        self.artifact_store.persist_state(
            &request.request_id,
            PipelineState::Publish,
            &input,
            &output,
            &diagnostics,
        )?;
        Ok(output)
    }

    fn hard_fail(
        &self,
        summary: &mut RunSummary,
        state: PipelineState,
        error: PipelineError,
    ) -> PipelineError {
        summary.failure_state = Some(state);
        if let PipelineError::Hard { code, .. } = &error {
            summary.error_codes.push(code.clone());
        }
        error
    }
}

fn meta(request: &RunRequest, state: PipelineState) -> EnvelopeMeta {
    EnvelopeMeta {
        request_id: request.request_id.clone(),
        page_id: request.page_id.clone(),
        state,
        timestamp: request.timestamp.clone(),
    }
}

fn project_prose_candidate(
    target_path: &str,
    markdown: &str,
    editable_prose_paths: &[String],
    allowed_scope_paths: &[String],
    prose_changed_paths: &mut Vec<String>,
    prose_change_candidates: &mut Vec<ProseChangeCandidate>,
) -> Result<(), PipelineError> {
    let canonical_path = canonicalize_mapped_path(target_path, allowed_scope_paths)
        .map_err(|error| to_hard_error(PipelineState::MdAssistEdit, error))?;

    let mapped_root = editable_prose_paths
        .iter()
        .find(|path| is_path_within_or_descendant(&canonical_path, path))
        .cloned()
        .ok_or_else(|| PipelineError::Hard {
            state: PipelineState::MdAssistEdit,
            code: ERR_ROUTE_VIOLATION.to_string(),
            message: format!("target path `{canonical_path}` is not mapped to editable prose"),
        })?;

    if canonical_path == mapped_root || canonical_path.ends_with("/type") {
        return Err(PipelineError::Hard {
            state: PipelineState::MdAssistEdit,
            code: ERR_SCHEMA_INVALID.to_string(),
            message: format!(
                "target path `{canonical_path}` violates prose boundary or top-level type constraints"
            ),
        });
    }

    prose_changed_paths.push(canonical_path.clone());
    prose_change_candidates.push(ProseChangeCandidate {
        path: canonical_path,
        markdown: markdown.to_string(),
    });
    Ok(())
}

fn project_table_candidate(
    target_path: &str,
    text: &str,
    allowed_scope_paths: &[String],
    node_path_index: &BTreeMap<String, String>,
    allowed_ops: &[TableOperation],
    table_candidates: &mut Vec<TableChangeCandidate>,
) -> Result<(), PipelineError> {
    let canonical_path = canonicalize_mapped_path(target_path, allowed_scope_paths)
        .map_err(|error| to_hard_error(PipelineState::AdfTableEdit, error))?;

    if !path_has_ancestor_type(
        &canonical_path,
        node_path_index,
        &["table", "tableRow", "tableCell"],
    ) {
        return Err(PipelineError::Hard {
            state: PipelineState::AdfTableEdit,
            code: ERR_ROUTE_VIOLATION.to_string(),
            message: format!("target path `{canonical_path}` is not in table route"),
        });
    }

    if !is_table_cell_text_path(&canonical_path, node_path_index)
        || is_table_shape_or_attr_path(&canonical_path, node_path_index)
    {
        return Err(PipelineError::Hard {
            state: PipelineState::AdfTableEdit,
            code: ERR_TABLE_SHAPE_CHANGE.to_string(),
            message: format!(
                "target path `{canonical_path}` is not an allowed table cell text update path"
            ),
        });
    }

    let candidate = TableChangeCandidate {
        op: TableOperation::CellTextUpdate,
        path: canonical_path,
        value: serde_json::Value::String(text.to_string()),
        source_route: "table_adf".to_string(),
    };

    validate_table_candidates(std::slice::from_ref(&candidate), allowed_ops).map_err(|error| {
        PipelineError::Hard {
            state: PipelineState::AdfTableEdit,
            code: ERR_TABLE_SHAPE_CHANGE.to_string(),
            message: error.to_string(),
        }
    })?;

    table_candidates.push(candidate);
    Ok(())
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
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

    match node_type {
        "paragraph" | "heading" | "bulletList" | "orderedList" | "listItem" | "blockquote"
        | "codeBlock" => "editable_prose",
        _ => "locked_structural",
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

fn to_hard_error(source_state: PipelineState, error: impl std::fmt::Display) -> PipelineError {
    let message = error.to_string();
    let code = if message.contains("out of scope") {
        ERR_OUT_OF_SCOPE_MUTATION
    } else if message.contains("table") && message.contains("shape") {
        ERR_TABLE_SHAPE_CHANGE
    } else if message.contains("whole-body rewrite") {
        ERR_ROUTE_VIOLATION
    } else if message.contains("scope") {
        atlassy_contracts::ERR_SCOPE_MISS
    } else {
        ERR_SCHEMA_INVALID
    };

    PipelineError::Hard {
        state: source_state,
        code: code.to_string(),
        message,
    }
}

impl From<AdfError> for PipelineError {
    fn from(error: AdfError) -> Self {
        to_hard_error(PipelineState::Patch, error)
    }
}

impl From<ConfluenceError> for PipelineError {
    fn from(error: ConfluenceError) -> Self {
        to_hard_error(PipelineState::Fetch, error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_tracker_blocks_out_of_order_transitions() {
        let mut tracker = StateTracker::new();
        assert!(tracker.transition_to(PipelineState::Fetch).is_ok());
        let err = tracker.transition_to(PipelineState::Patch).unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidTransition {
                expected: "classify".to_string(),
                actual: "patch".to_string(),
            }
        );
    }
}
