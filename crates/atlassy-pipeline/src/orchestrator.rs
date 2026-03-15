use std::collections::BTreeMap;
use std::path::Path;

use atlassy_adf::{bootstrap_scaffold, build_node_path_index, is_page_effectively_empty};
use atlassy_confluence::ConfluenceClient;
use atlassy_contracts::{
    validate_run_summary_telemetry, ErrorCode, Operation, PipelineState, PublishResult, RunSummary,
    VerifyResult,
};

use crate::error_map::to_hard_error;
use crate::util::{add_duration_suffix, compute_section_bytes, estimate_tokens};
use crate::{states, ArtifactStore, PipelineError, RunRequest, StateTracker};

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
        let run_started = std::time::Instant::now();
        let mut tracker = StateTracker::new();
        let mut run_summary = RunSummary {
            success: false,
            run_id: request.request_id.clone(),
            request_id: request.request_id.clone(),
            page_id: request.page_id.clone(),
            flow: request.flow.clone(),
            pattern: request.pattern.clone(),
            edit_intent_hash: request.edit_intent_hash.clone(),
            scope_selectors: request.scope_selectors.clone(),
            scope_resolution_failed: false,
            full_page_fetch: false,
            full_page_adf_bytes: 0,
            scoped_adf_bytes: 0,
            context_reduction_ratio: 0.0,
            pipeline_version: request.provenance.pipeline_version.clone(),
            git_commit_sha: request.provenance.git_commit_sha.clone(),
            git_dirty: request.provenance.git_dirty,
            runtime_mode: request.provenance.runtime_mode.clone(),
            state_token_usage: BTreeMap::new(),
            total_tokens: 0,
            retry_count: 0,
            retry_tokens: 0,
            patch_ops_bytes: 0,
            verify_result: String::new(),
            verify_error_codes: Vec::new(),
            publish_result: String::new(),
            publish_error_code: None,
            new_version: None,
            start_ts: request.timestamp.clone(),
            verify_end_ts: String::new(),
            publish_end_ts: String::new(),
            latency_ms: 0,
            locked_node_mutation: false,
            out_of_scope_mutation: false,
            telemetry_complete: false,
            discovered_target_path: None,
            applied_paths: Vec::new(),
            blocked_paths: Vec::new(),
            error_codes: Vec::new(),
            token_metrics: BTreeMap::new(),
            failure_state: None,
            empty_page_detected: false,
            bootstrap_applied: false,
        };

        let result = self.run_internal(&request, &mut tracker, &mut run_summary, &run_started);
        if result.is_ok() {
            run_summary.success = true;
        }

        run_summary.total_tokens = run_summary.state_token_usage.values().copied().sum();
        run_summary.latency_ms = run_started.elapsed().as_millis() as u64;
        if run_summary.verify_end_ts.is_empty() {
            run_summary.verify_end_ts =
                add_duration_suffix(&request.timestamp, run_summary.latency_ms);
        }
        if run_summary.publish_end_ts.is_empty() {
            run_summary.publish_end_ts =
                add_duration_suffix(&request.timestamp, run_summary.latency_ms);
        }

        run_summary.locked_node_mutation = run_summary
            .error_codes
            .iter()
            .any(|code| code == ErrorCode::LockedNodeMutation.as_str());
        run_summary.out_of_scope_mutation = run_summary
            .error_codes
            .iter()
            .any(|code| code == ErrorCode::OutOfScopeMutation.as_str());
        run_summary.telemetry_complete = validate_run_summary_telemetry(&run_summary).is_ok();

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
        run_started: &std::time::Instant,
    ) -> Result<(), PipelineError> {
        let mut fetch =
            states::run_fetch_state(&mut self.client, &self.artifact_store, request, tracker)
                .map_err(|error| self.hard_fail(summary, PipelineState::Fetch, error))?;
        summary.state_token_usage.insert(
            PipelineState::Fetch.as_str().to_string(),
            estimate_tokens(&fetch)?,
        );

        // Bootstrap empty-page detection: evaluate the four-path behavior matrix
        // between fetch and classify, without adding a new pipeline state.
        let page_empty = is_page_effectively_empty(&fetch.payload.scoped_adf);
        summary.empty_page_detected = page_empty;

        match (page_empty, request.bootstrap_empty_page) {
            (true, false) => {
                return Err(self.hard_fail(
                    summary,
                    PipelineState::Fetch,
                    PipelineError::Hard {
                        state: PipelineState::Fetch,
                        code: ErrorCode::BootstrapRequired,
                        message:
                            "page is effectively empty; use --bootstrap-empty-page to bootstrap"
                                .to_string(),
                    },
                ));
            }
            (false, true) => {
                return Err(self.hard_fail(
                    summary,
                    PipelineState::Fetch,
                    PipelineError::Hard {
                        state: PipelineState::Fetch,
                        code: ErrorCode::BootstrapInvalidState,
                        message: "page is not empty; --bootstrap-empty-page is not valid for non-empty pages".to_string(),
                    },
                ));
            }
            (true, true) => {
                // Inject minimal prose scaffold and rebuild index
                let scaffold = bootstrap_scaffold();
                let node_path_index = build_node_path_index(&scaffold).map_err(|error| {
                    self.hard_fail(
                        summary,
                        PipelineState::Fetch,
                        to_hard_error(PipelineState::Fetch, error),
                    )
                })?;
                let allowed_scope_paths: Vec<String> = node_path_index.keys().cloned().collect();
                fetch.payload.scoped_adf = scaffold;
                fetch.payload.node_path_index = node_path_index;
                fetch.payload.allowed_scope_paths = allowed_scope_paths;
                fetch.payload.full_page_fetch = true;
                fetch.payload.scope_resolution_failed = true;
                fetch.payload.fallback_reason = Some("bootstrap_scaffold".to_string());
                summary.bootstrap_applied = true;
            }
            (false, false) => {
                // Normal v1 flow - unchanged
            }
        }

        summary.full_page_adf_bytes = fetch.payload.full_page_adf_bytes;
        summary.scoped_adf_bytes = compute_section_bytes(
            &fetch.payload.scoped_adf,
            &fetch.payload.allowed_scope_paths,
        );
        summary.context_reduction_ratio = if summary.full_page_adf_bytes > 0 {
            1.0 - (summary.scoped_adf_bytes as f64 / summary.full_page_adf_bytes as f64)
        } else {
            0.0
        };

        let classify = states::run_classify_state(&self.artifact_store, request, tracker, &fetch)
            .map_err(|error| self.hard_fail(summary, PipelineState::Classify, error))?;
        summary.state_token_usage.insert(
            PipelineState::Classify.as_str().to_string(),
            estimate_tokens(&classify)?,
        );

        let extract = states::run_extract_prose_state(
            &self.artifact_store,
            request,
            tracker,
            &fetch,
            &classify,
        )
        .map_err(|error| self.hard_fail(summary, PipelineState::ExtractProse, error))?;
        summary.state_token_usage.insert(
            PipelineState::ExtractProse.as_str().to_string(),
            estimate_tokens(&extract)?,
        );

        let md_edit = states::run_md_assist_edit_state(
            &self.artifact_store,
            request,
            tracker,
            &fetch,
            &extract,
            summary,
        )
        .map_err(|error| self.hard_fail(summary, PipelineState::MdAssistEdit, error))?;
        summary.state_token_usage.insert(
            PipelineState::MdAssistEdit.as_str().to_string(),
            estimate_tokens(&md_edit)?,
        );

        let table_edit = states::run_adf_table_edit_state(
            &self.artifact_store,
            request,
            tracker,
            &fetch,
            &classify,
            summary,
        )
        .map_err(|error| self.hard_fail(summary, PipelineState::AdfTableEdit, error))?;
        summary.state_token_usage.insert(
            PipelineState::AdfTableEdit.as_str().to_string(),
            estimate_tokens(&table_edit)?,
        );

        let adf_block_ops =
            states::run_adf_block_ops_state(&self.artifact_store, request, tracker, &fetch)
                .map_err(|error| self.hard_fail(summary, PipelineState::AdfBlockOps, error))?;
        summary.state_token_usage.insert(
            PipelineState::AdfBlockOps.as_str().to_string(),
            estimate_tokens(&adf_block_ops)?,
        );

        let merged = states::run_merge_candidates_state(
            &self.artifact_store,
            request,
            tracker,
            &classify,
            &md_edit,
            &table_edit,
            &adf_block_ops,
        )
        .map_err(|error| self.hard_fail(summary, PipelineState::MergeCandidates, error))?;
        summary.state_token_usage.insert(
            PipelineState::MergeCandidates.as_str().to_string(),
            estimate_tokens(&merged)?,
        );

        let patch =
            states::run_patch_state(&self.artifact_store, request, tracker, &fetch, &merged)
                .map_err(|error| self.hard_fail(summary, PipelineState::Patch, error))?;
        summary.state_token_usage.insert(
            PipelineState::Patch.as_str().to_string(),
            estimate_tokens(&patch)?,
        );

        let verify = states::run_verify_state(
            &self.artifact_store,
            request,
            tracker,
            &fetch,
            &patch,
            &merged,
        )
        .map_err(|error| self.hard_fail(summary, PipelineState::Verify, error))?;
        summary.state_token_usage.insert(
            PipelineState::Verify.as_str().to_string(),
            estimate_tokens(&verify)?,
        );

        summary.scope_resolution_failed = fetch.payload.scope_resolution_failed;
        summary.full_page_fetch = fetch.payload.full_page_fetch;
        summary.patch_ops_bytes = patch.payload.patch_ops_bytes;
        summary.verify_result = match verify.payload.verify_result {
            VerifyResult::Pass => "pass".to_string(),
            VerifyResult::Fail => "fail".to_string(),
        };
        summary.verify_error_codes = verify
            .payload
            .diagnostics
            .errors
            .iter()
            .map(|error| error.code.clone())
            .collect();
        summary.verify_end_ts =
            add_duration_suffix(&request.timestamp, run_started.elapsed().as_millis() as u64);

        if verify.payload.verify_result == VerifyResult::Fail {
            summary.failure_state = Some(PipelineState::Verify);
            summary
                .error_codes
                .extend(summary.verify_error_codes.clone());
            let primary_code = verify
                .payload
                .diagnostics
                .errors
                .first()
                .map(|error| match error.code.as_str() {
                    code if code == ErrorCode::TableShapeChange.as_str() => {
                        ErrorCode::TableShapeChange
                    }
                    code if code == ErrorCode::RouteViolation.as_str() => ErrorCode::RouteViolation,
                    code if code == ErrorCode::OutOfScopeMutation.as_str() => {
                        ErrorCode::OutOfScopeMutation
                    }
                    code if code == ErrorCode::InsertPositionInvalid.as_str() => {
                        ErrorCode::InsertPositionInvalid
                    }
                    code if code == ErrorCode::RemoveAnchorMissing.as_str() => {
                        ErrorCode::RemoveAnchorMissing
                    }
                    code if code == ErrorCode::PostMutationSchemaInvalid.as_str() => {
                        ErrorCode::PostMutationSchemaInvalid
                    }
                    code if code == ErrorCode::SectionBoundaryInvalid.as_str() => {
                        ErrorCode::SectionBoundaryInvalid
                    }
                    code if code == ErrorCode::StructuralCompositionFailed.as_str() => {
                        ErrorCode::StructuralCompositionFailed
                    }
                    code if code == ErrorCode::TableRowInvalid.as_str() => {
                        ErrorCode::TableRowInvalid
                    }
                    code if code == ErrorCode::TableColumnInvalid.as_str() => {
                        ErrorCode::TableColumnInvalid
                    }
                    code if code == ErrorCode::AttrUpdateBlocked.as_str() => {
                        ErrorCode::AttrUpdateBlocked
                    }
                    code if code == ErrorCode::AttrSchemaViolation.as_str() => {
                        ErrorCode::AttrSchemaViolation
                    }
                    _ => ErrorCode::SchemaInvalid,
                })
                .unwrap_or(ErrorCode::SchemaInvalid);
            return Err(PipelineError::Hard {
                state: PipelineState::Verify,
                code: primary_code,
                message: "verification failed".to_string(),
            });
        }

        let publish = states::run_publish_state(
            &mut self.client,
            &self.artifact_store,
            request,
            tracker,
            &fetch,
            &patch,
        )
        .map_err(|error| self.hard_fail(summary, PipelineState::Publish, error))?;
        summary.state_token_usage.insert(
            PipelineState::Publish.as_str().to_string(),
            estimate_tokens(&publish)?,
        );

        summary.publish_result = match publish.payload.publish_result {
            PublishResult::Published => "published".to_string(),
            PublishResult::Failed => "failed".to_string(),
        };
        summary.publish_error_code = publish
            .payload
            .diagnostics
            .errors
            .first()
            .map(|error| error.code.clone());
        summary.new_version = publish.payload.new_version;
        summary.retry_count = publish.payload.retry_count;
        summary.publish_end_ts =
            add_duration_suffix(&request.timestamp, run_started.elapsed().as_millis() as u64);

        if publish.payload.publish_result == PublishResult::Failed {
            summary.failure_state = Some(PipelineState::Publish);
            summary
                .error_codes
                .push(ErrorCode::ConflictRetryExhausted.to_string());
            return Err(PipelineError::Hard {
                state: PipelineState::Publish,
                code: ErrorCode::ConflictRetryExhausted,
                message: "publish failed after retry".to_string(),
            });
        }

        summary.applied_paths = merged
            .payload
            .operations
            .iter()
            .flat_map(operation_paths)
            .collect();
        summary.token_metrics = summary.state_token_usage.clone();
        summary.retry_tokens = if summary.retry_count > 0 {
            summary
                .state_token_usage
                .get(PipelineState::Publish.as_str())
                .copied()
                .unwrap_or(0)
                * u64::from(summary.retry_count)
        } else {
            0
        };
        Ok(())
    }

    fn hard_fail(
        &self,
        summary: &mut RunSummary,
        state: PipelineState,
        error: PipelineError,
    ) -> PipelineError {
        summary.failure_state = Some(state);
        if let PipelineError::Hard { code, .. } = &error {
            summary.error_codes.push(code.to_string());
        }
        error
    }
}

fn operation_paths(operation: &Operation) -> Vec<String> {
    match operation {
        Operation::Replace { path, .. } => vec![path.clone()],
        Operation::Insert {
            parent_path, index, ..
        } => vec![format!("{parent_path}/{index}")],
        Operation::Remove { target_path } => vec![target_path.clone()],
        Operation::UpdateAttrs { target_path, .. } => vec![target_path.clone()],
    }
}
