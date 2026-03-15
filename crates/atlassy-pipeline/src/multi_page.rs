use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Instant;

use atlassy_confluence::{ConfluenceClient, ConfluenceError, CreatePageResponse};
use atlassy_contracts::{
    ErrorCode, FLOW_OPTIMIZED, MultiPageRequest, MultiPageSummary, PATTERN_A, PageResult,
    PageRunMode, PageSnapshot, PageTarget, PipelineState, RollbackResult, RunSummary,
};

use crate::error_map::confluence_error_to_hard_error;
use crate::{ArtifactStore, Orchestrator, PipelineError, RunMode, RunRequest};

pub struct MultiPageOrchestrator<C: ConfluenceClient> {
    client: C,
    artifact_root: PathBuf,
    artifact_store: ArtifactStore,
}

impl<C: ConfluenceClient> MultiPageOrchestrator<C> {
    pub fn new(client: C, artifact_root: impl AsRef<Path>) -> Self {
        let artifact_root = artifact_root.as_ref().to_path_buf();
        Self {
            client,
            artifact_store: ArtifactStore::new(&artifact_root),
            artifact_root,
        }
    }

    pub fn client(&self) -> &C {
        &self.client
    }

    pub fn client_mut(&mut self) -> &mut C {
        &mut self.client
    }

    pub fn run(
        &mut self,
        multi_request: MultiPageRequest,
    ) -> Result<MultiPageSummary, PipelineError> {
        let started = Instant::now();
        let execution_order = sort_page_targets(&multi_request.pages)?;

        let mut snapshots = HashMap::new();
        for page in &multi_request.pages {
            if let Some(page_id) = page.page_id.as_deref() {
                snapshots
                    .entry(page_id.to_string())
                    .or_insert(take_snapshot(&mut self.client, page_id)?);
            }
        }

        let mut page_results = Vec::new();
        let mut successful_pages = Vec::new();
        let mut failed = false;

        for index in execution_order {
            let target = &multi_request.pages[index];
            let run_id = format!("{}-{}", multi_request.plan_id, index);

            let (resolved_page_id, created) = match resolve_page_target(&mut self.client, target) {
                Ok((page_id, created)) => (page_id, created),
                Err(error) => {
                    let (error_code, error_message) = match &error {
                        PipelineError::Hard { code, message, .. } => (*code, message.clone()),
                        _ => (ErrorCode::PageCreationFailed, error.to_string()),
                    };
                    let page_id = target
                        .page_id
                        .clone()
                        .unwrap_or_else(|| plan_target_reference(index));
                    let mut summary = synthetic_failure_summary(
                        &multi_request,
                        target,
                        &run_id,
                        &page_id,
                        error_code,
                        &error_message,
                        started.elapsed().as_millis() as u64,
                    );
                    append_error_code(&mut summary, ErrorCode::MultiPagePartialFailure);
                    page_results.push(PageResult {
                        page_id,
                        created: false,
                        summary,
                    });
                    failed = true;
                    break;
                }
            };

            let run_request = build_run_request(
                &multi_request,
                target,
                &run_id,
                &resolved_page_id,
                index,
                created,
            );

            let run_result = {
                let mut orchestrator =
                    Orchestrator::new(ClientRef::new(&mut self.client), &self.artifact_root);
                orchestrator.run(run_request)
            };

            match run_result {
                Ok(summary) => {
                    if let Some(snapshot) = snapshots.get_mut(&resolved_page_id) {
                        snapshot.version_after = summary.new_version;
                    }
                    successful_pages.push(ExecutedPage {
                        page_id: resolved_page_id.clone(),
                        created,
                    });
                    page_results.push(PageResult {
                        page_id: resolved_page_id,
                        created,
                        summary,
                    });
                }
                Err(error) => {
                    let mut failed_summary = self.load_run_summary(&run_id).unwrap_or_else(|_| {
                        synthetic_failure_summary(
                            &multi_request,
                            target,
                            &run_id,
                            &resolved_page_id,
                            ErrorCode::MultiPagePartialFailure,
                            &error.to_string(),
                            started.elapsed().as_millis() as u64,
                        )
                    });
                    append_error_code(&mut failed_summary, ErrorCode::MultiPagePartialFailure);
                    page_results.push(PageResult {
                        page_id: resolved_page_id,
                        created,
                        summary: failed_summary,
                    });
                    failed = true;
                    break;
                }
            }
        }

        let rollback_results = if failed && multi_request.rollback_on_failure {
            let mut results = Vec::new();
            for executed_page in successful_pages.iter().rev() {
                if executed_page.created {
                    continue;
                }
                if let Some(snapshot) = snapshots.get(&executed_page.page_id) {
                    results.push(rollback_page(&mut self.client, snapshot));
                }
            }
            results
        } else {
            Vec::new()
        };

        let failed_pages = page_results
            .iter()
            .filter(|result| !result.summary.success)
            .count();
        let succeeded_pages = page_results.len().saturating_sub(failed_pages);
        let rolled_back_pages = rollback_results
            .iter()
            .filter(|result| result.success)
            .count();

        let summary = MultiPageSummary {
            plan_id: multi_request.plan_id.clone(),
            success: failed_pages == 0,
            page_results,
            rollback_results,
            total_pages: multi_request.pages.len(),
            succeeded_pages,
            failed_pages,
            rolled_back_pages,
            latency_ms: started.elapsed().as_millis() as u64,
        };

        self.artifact_store
            .persist_multi_page_summary(&multi_request.plan_id, &summary)?;

        Ok(summary)
    }

    fn load_run_summary(&self, run_id: &str) -> Result<RunSummary, PipelineError> {
        let path = self
            .artifact_root
            .join("artifacts")
            .join(run_id)
            .join("summary.json");
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }
}

fn append_error_code(summary: &mut RunSummary, code: ErrorCode) {
    let code = code.as_str().to_string();
    if !summary.error_codes.contains(&code) {
        summary.error_codes.push(code);
    }
}

fn resolve_page_target<C: ConfluenceClient>(
    client: &mut C,
    target: &PageTarget,
) -> Result<(String, bool), PipelineError> {
    if let Some(create) = &target.create {
        let CreatePageResponse { page_id, .. } = client
            .create_page(&create.title, &create.parent_page_id, &create.space_key)
            .map_err(|error| PipelineError::Hard {
                state: PipelineState::Fetch,
                code: ErrorCode::PageCreationFailed,
                message: format!("page creation failed: {error}"),
            })?;
        return Ok((page_id, true));
    }

    let page_id = target.page_id.clone().ok_or_else(|| PipelineError::Hard {
        state: PipelineState::Fetch,
        code: ErrorCode::SchemaInvalid,
        message: "page target must specify either `page_id` or `create`".to_string(),
    })?;
    Ok((page_id, false))
}

fn build_run_request(
    multi_request: &MultiPageRequest,
    target: &PageTarget,
    run_id: &str,
    page_id: &str,
    target_index: usize,
    created: bool,
) -> RunRequest {
    RunRequest {
        request_id: run_id.to_string(),
        page_id: page_id.to_string(),
        edit_intent: target.edit_intent.clone(),
        edit_intent_hash: hash_edit_intent(&target.edit_intent),
        flow: FLOW_OPTIMIZED.to_string(),
        pattern: PATTERN_A.to_string(),
        scope_selectors: target.scope_selectors.clone(),
        timestamp: multi_request.timestamp.clone(),
        provenance: multi_request.provenance.clone(),
        run_mode: to_run_mode(&target.run_mode),
        target_index,
        block_ops: target.block_ops.clone(),
        force_verify_fail: false,
        bootstrap_empty_page: if created {
            true
        } else {
            target.bootstrap_empty_page
        },
    }
}

fn to_run_mode(mode: &PageRunMode) -> RunMode {
    match mode {
        PageRunMode::NoOp => RunMode::NoOp,
        PageRunMode::SimpleScopedProseUpdate {
            target_path,
            markdown,
        } => RunMode::SimpleScopedProseUpdate {
            target_path: target_path.clone(),
            markdown: markdown.clone(),
        },
        PageRunMode::SimpleScopedTableCellUpdate { target_path, text } => {
            RunMode::SimpleScopedTableCellUpdate {
                target_path: target_path.clone(),
                text: text.clone(),
            }
        }
        PageRunMode::ForbiddenTableOperation {
            target_path,
            operation,
        } => RunMode::ForbiddenTableOperation {
            target_path: target_path.clone(),
            operation: *operation,
        },
        PageRunMode::SyntheticRouteConflict {
            prose_path,
            table_path,
        } => RunMode::SyntheticRouteConflict {
            prose_path: prose_path.clone(),
            table_path: table_path.clone(),
        },
        PageRunMode::SyntheticTableShapeDrift { path } => {
            RunMode::SyntheticTableShapeDrift { path: path.clone() }
        }
    }
}

fn hash_edit_intent(edit_intent: &str) -> String {
    let mut hasher = DefaultHasher::new();
    edit_intent.hash(&mut hasher);
    format!("h{:016x}", hasher.finish())
}

fn synthetic_failure_summary(
    multi_request: &MultiPageRequest,
    target: &PageTarget,
    run_id: &str,
    page_id: &str,
    code: ErrorCode,
    _message: &str,
    latency_ms: u64,
) -> RunSummary {
    RunSummary {
        success: false,
        run_id: run_id.to_string(),
        request_id: run_id.to_string(),
        page_id: page_id.to_string(),
        flow: FLOW_OPTIMIZED.to_string(),
        pattern: PATTERN_A.to_string(),
        edit_intent_hash: hash_edit_intent(&target.edit_intent),
        scope_selectors: target.scope_selectors.clone(),
        scope_resolution_failed: false,
        full_page_fetch: false,
        full_page_adf_bytes: 0,
        scoped_adf_bytes: 0,
        context_reduction_ratio: 0.0,
        pipeline_version: multi_request.provenance.pipeline_version.clone(),
        git_commit_sha: multi_request.provenance.git_commit_sha.clone(),
        git_dirty: multi_request.provenance.git_dirty,
        runtime_mode: multi_request.provenance.runtime_mode.clone(),
        state_token_usage: BTreeMap::new(),
        total_tokens: 0,
        retry_count: 0,
        retry_tokens: 0,
        patch_ops_bytes: 0,
        verify_result: "fail".to_string(),
        verify_error_codes: vec![code.as_str().to_string()],
        publish_result: "failed".to_string(),
        publish_error_code: Some(code.as_str().to_string()),
        new_version: None,
        start_ts: multi_request.timestamp.clone(),
        verify_end_ts: multi_request.timestamp.clone(),
        publish_end_ts: multi_request.timestamp.clone(),
        latency_ms,
        locked_node_mutation: false,
        out_of_scope_mutation: false,
        telemetry_complete: false,
        discovered_target_path: None,
        applied_paths: vec![],
        blocked_paths: vec![],
        error_codes: vec![code.as_str().to_string()],
        token_metrics: BTreeMap::new(),
        failure_state: Some(PipelineState::Fetch),
        empty_page_detected: false,
        bootstrap_applied: false,
    }
}

fn plan_target_reference(index: usize) -> String {
    format!("@{index}")
}

pub fn sort_page_targets(pages: &[PageTarget]) -> Result<Vec<usize>, PipelineError> {
    if pages.is_empty() {
        return Ok(Vec::new());
    }

    let mut id_to_index = HashMap::<String, usize>::new();
    for (index, page) in pages.iter().enumerate() {
        id_to_index.insert(plan_target_reference(index), index);
        if let Some(page_id) = page.page_id.as_deref()
            && let Some(previous_index) = id_to_index.insert(page_id.to_string(), index)
            && previous_index != index
        {
            return Err(PipelineError::Hard {
                state: PipelineState::Fetch,
                code: ErrorCode::SchemaInvalid,
                message: format!("duplicate dependency identifier `{page_id}` in page targets"),
            });
        }
    }

    let mut edges = vec![Vec::new(); pages.len()];
    let mut indegree = vec![0usize; pages.len()];

    for (index, page) in pages.iter().enumerate() {
        for dependency in &page.depends_on {
            let dependency_index =
                id_to_index
                    .get(dependency)
                    .copied()
                    .ok_or_else(|| PipelineError::Hard {
                        state: PipelineState::Fetch,
                        code: ErrorCode::SchemaInvalid,
                        message: format!(
                            "unresolvable dependency `{dependency}` for page target {index}"
                        ),
                    })?;
            edges[dependency_index].push(index);
            indegree[index] += 1;
        }
    }

    let mut ready = BTreeSet::new();
    for (index, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            ready.insert(index);
        }
    }

    let mut order = Vec::with_capacity(pages.len());
    while let Some(index) = ready.pop_first() {
        order.push(index);
        for dependent in &edges[index] {
            indegree[*dependent] -= 1;
            if indegree[*dependent] == 0 {
                ready.insert(*dependent);
            }
        }
    }

    if order.len() != pages.len() {
        return Err(PipelineError::Hard {
            state: PipelineState::Fetch,
            code: ErrorCode::DependencyCycle,
            message: "dependency cycle detected in multi-page plan".to_string(),
        });
    }

    Ok(order)
}

pub fn take_snapshot<C: ConfluenceClient>(
    client: &mut C,
    page_id: &str,
) -> Result<PageSnapshot, PipelineError> {
    let response = client
        .fetch_page(page_id)
        .map_err(|error| confluence_error_to_hard_error(PipelineState::Fetch, error))?;
    Ok(PageSnapshot {
        page_id: page_id.to_string(),
        version_before: response.page_version,
        adf_before: response.adf,
        version_after: None,
    })
}

pub fn rollback_page<C: ConfluenceClient>(
    client: &mut C,
    snapshot: &PageSnapshot,
) -> RollbackResult {
    let current = match client.fetch_page(&snapshot.page_id) {
        Ok(response) => response,
        Err(error) => {
            return RollbackResult {
                page_id: snapshot.page_id.clone(),
                success: false,
                conflict: false,
                error: Some(error.to_string()),
            };
        }
    };

    let expected_version = match snapshot.version_after {
        Some(version) => version,
        None => {
            return RollbackResult {
                page_id: snapshot.page_id.clone(),
                success: false,
                conflict: false,
                error: Some("missing snapshot version_after".to_string()),
            };
        }
    };

    if current.page_version != expected_version {
        return RollbackResult {
            page_id: snapshot.page_id.clone(),
            success: false,
            conflict: true,
            error: Some(ErrorCode::RollbackConflict.as_str().to_string()),
        };
    }

    match client.publish_page(
        &snapshot.page_id,
        current.page_version,
        &snapshot.adf_before,
    ) {
        Ok(_) => RollbackResult {
            page_id: snapshot.page_id.clone(),
            success: true,
            conflict: false,
            error: None,
        },
        Err(error) => RollbackResult {
            page_id: snapshot.page_id.clone(),
            success: false,
            conflict: matches!(error, ConfluenceError::Conflict(_)),
            error: Some(error.to_string()),
        },
    }
}

struct ExecutedPage {
    page_id: String,
    created: bool,
}

struct ClientRef<'a, C: ConfluenceClient> {
    inner: &'a mut C,
}

impl<'a, C: ConfluenceClient> ClientRef<'a, C> {
    fn new(inner: &'a mut C) -> Self {
        Self { inner }
    }
}

impl<C: ConfluenceClient> ConfluenceClient for ClientRef<'_, C> {
    fn fetch_page(
        &mut self,
        page_id: &str,
    ) -> Result<atlassy_confluence::FetchPageResponse, ConfluenceError> {
        self.inner.fetch_page(page_id)
    }

    fn publish_page(
        &mut self,
        page_id: &str,
        page_version: u64,
        candidate_adf: &serde_json::Value,
    ) -> Result<atlassy_confluence::PublishPageResponse, ConfluenceError> {
        self.inner
            .publish_page(page_id, page_version, candidate_adf)
    }

    fn create_page(
        &mut self,
        title: &str,
        parent_page_id: &str,
        space_key: &str,
    ) -> Result<CreatePageResponse, ConfluenceError> {
        self.inner.create_page(title, parent_page_id, space_key)
    }

    fn publish_attempts(&self) -> usize {
        self.inner.publish_attempts()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use atlassy_confluence::{
        FetchPageResponse, PublishPageResponse, StubConfluenceClient, StubPage,
    };
    use atlassy_contracts::{
        BlockOp, CreatePageTarget, PIPELINE_VERSION, PageRunMode, ProvenanceStamp, RUNTIME_STUB,
        TableOperation,
    };

    use super::*;

    fn sample_target(page_id: Option<&str>, deps: &[&str]) -> PageTarget {
        PageTarget {
            page_id: page_id.map(ToString::to_string),
            create: None,
            edit_intent: "update".to_string(),
            scope_selectors: vec![],
            run_mode: PageRunMode::NoOp,
            block_ops: vec![],
            bootstrap_empty_page: false,
            depends_on: deps.iter().map(|dep| dep.to_string()).collect(),
        }
    }

    #[test]
    fn sort_page_targets_orders_linear_chain() {
        let pages = vec![
            sample_target(Some("A"), &[]),
            sample_target(Some("B"), &["A"]),
            sample_target(Some("C"), &["B"]),
        ];
        assert_eq!(sort_page_targets(&pages).unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn sort_page_targets_orders_diamond_graph() {
        let pages = vec![
            sample_target(Some("A"), &[]),
            sample_target(Some("B"), &["A"]),
            sample_target(Some("C"), &["A"]),
            sample_target(Some("D"), &["B", "C"]),
        ];
        let order = sort_page_targets(&pages).unwrap();
        let a = order.iter().position(|index| *index == 0).unwrap();
        let b = order.iter().position(|index| *index == 1).unwrap();
        let c = order.iter().position(|index| *index == 2).unwrap();
        let d = order.iter().position(|index| *index == 3).unwrap();
        assert!(a < b);
        assert!(a < c);
        assert!(b < d);
        assert!(c < d);
    }

    #[test]
    fn sort_page_targets_supports_independent_and_empty_plans() {
        let pages = vec![
            sample_target(Some("A"), &[]),
            sample_target(Some("B"), &[]),
            sample_target(Some("C"), &[]),
        ];
        let order = sort_page_targets(&pages).unwrap();
        assert_eq!(order.len(), 3);
        assert_eq!(sort_page_targets(&[]).unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn sort_page_targets_detects_direct_and_indirect_cycles() {
        let direct = vec![
            sample_target(Some("A"), &["B"]),
            sample_target(Some("B"), &["A"]),
        ];
        let indirect = vec![
            sample_target(Some("A"), &["C"]),
            sample_target(Some("B"), &["A"]),
            sample_target(Some("C"), &["B"]),
        ];

        for pages in [direct, indirect] {
            let error = sort_page_targets(&pages).unwrap_err();
            match error {
                PipelineError::Hard { code, .. } => {
                    assert_eq!(code, ErrorCode::DependencyCycle);
                }
                other => panic!("unexpected error: {other:?}"),
            }
        }
    }

    #[test]
    fn sort_page_targets_detects_self_cycle_and_unresolvable_dependency() {
        let self_cycle = vec![sample_target(Some("A"), &["A"])];
        let unresolved = vec![sample_target(Some("A"), &["missing"])];

        let cycle_error = sort_page_targets(&self_cycle).unwrap_err();
        match cycle_error {
            PipelineError::Hard { code, .. } => assert_eq!(code, ErrorCode::DependencyCycle),
            other => panic!("unexpected error: {other:?}"),
        }

        let unresolved_error = sort_page_targets(&unresolved).unwrap_err();
        match unresolved_error {
            PipelineError::Hard { code, .. } => assert_eq!(code, ErrorCode::SchemaInvalid),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn sort_page_targets_supports_plan_local_reference_for_created_pages() {
        let created = PageTarget {
            page_id: None,
            create: Some(CreatePageTarget {
                title: "child".to_string(),
                parent_page_id: "parent".to_string(),
                space_key: "PROJ".to_string(),
            }),
            edit_intent: "create".to_string(),
            scope_selectors: vec![],
            run_mode: PageRunMode::NoOp,
            block_ops: vec![],
            bootstrap_empty_page: false,
            depends_on: vec![],
        };
        let dependent = sample_target(Some("A"), &["@0"]);
        assert_eq!(
            sort_page_targets(&[created, dependent]).unwrap(),
            vec![0, 1]
        );
    }

    #[test]
    fn take_snapshot_captures_page_state_and_fetch_failure() {
        let mut pages = HashMap::new();
        pages.insert(
            "page-1".to_string(),
            StubPage {
                version: 4,
                adf: serde_json::json!({"type": "doc", "content": []}),
            },
        );
        let mut client = StubConfluenceClient::new(pages);

        let snapshot = take_snapshot(&mut client, "page-1").unwrap();
        assert_eq!(snapshot.page_id, "page-1");
        assert_eq!(snapshot.version_before, 4);
        assert_eq!(snapshot.version_after, None);

        let fetch_error = take_snapshot(&mut client, "missing").unwrap_err();
        match fetch_error {
            PipelineError::Hard { code, .. } => assert_eq!(code, ErrorCode::RuntimeBackend),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rollback_page_handles_success_conflict_and_fetch_failure() {
        let original = serde_json::json!({"type": "doc", "content": [{"type": "paragraph"}]});

        let mut success_pages = HashMap::new();
        success_pages.insert(
            "page-1".to_string(),
            StubPage {
                version: 5,
                adf: serde_json::json!({"type": "doc", "content": [{"type": "text"}]}),
            },
        );
        let mut success_client = StubConfluenceClient::new(success_pages);
        let snapshot = PageSnapshot {
            page_id: "page-1".to_string(),
            version_before: 4,
            adf_before: original.clone(),
            version_after: Some(5),
        };

        let success = rollback_page(&mut success_client, &snapshot);
        assert!(success.success);
        assert!(!success.conflict);

        let restored = success_client.fetch_page("page-1").unwrap();
        assert_eq!(restored.adf, original);
        assert_eq!(restored.page_version, 6);

        let mut conflict_pages = HashMap::new();
        conflict_pages.insert(
            "page-1".to_string(),
            StubPage {
                version: 6,
                adf: serde_json::json!({"type": "doc", "content": []}),
            },
        );
        let mut conflict_client = StubConfluenceClient::new(conflict_pages);
        let conflict = rollback_page(&mut conflict_client, &snapshot);
        assert!(!conflict.success);
        assert!(conflict.conflict);
        assert_eq!(
            conflict.error.as_deref(),
            Some(ErrorCode::RollbackConflict.as_str())
        );

        let mut missing_client = StubConfluenceClient::new(HashMap::new());
        let missing = rollback_page(&mut missing_client, &snapshot);
        assert!(!missing.success);
        assert!(!missing.conflict);
        assert!(missing.error.unwrap().contains("page not found"));
    }

    #[derive(Debug, Clone)]
    struct PublishConflictClient {
        fetch_response: FetchPageResponse,
        publish_attempts: usize,
    }

    impl ConfluenceClient for PublishConflictClient {
        fn fetch_page(&mut self, _page_id: &str) -> Result<FetchPageResponse, ConfluenceError> {
            Ok(self.fetch_response.clone())
        }

        fn publish_page(
            &mut self,
            page_id: &str,
            _page_version: u64,
            _candidate_adf: &serde_json::Value,
        ) -> Result<PublishPageResponse, ConfluenceError> {
            self.publish_attempts += 1;
            Err(ConfluenceError::Conflict(page_id.to_string()))
        }

        fn create_page(
            &mut self,
            _title: &str,
            _parent_page_id: &str,
            _space_key: &str,
        ) -> Result<CreatePageResponse, ConfluenceError> {
            Err(ConfluenceError::NotImplemented)
        }

        fn publish_attempts(&self) -> usize {
            self.publish_attempts
        }
    }

    #[test]
    fn rollback_page_sets_conflict_on_publish_conflict_error() {
        let mut client = PublishConflictClient {
            fetch_response: FetchPageResponse {
                page_version: 5,
                adf: serde_json::json!({"type": "doc", "content": []}),
            },
            publish_attempts: 0,
        };
        let snapshot = PageSnapshot {
            page_id: "page-1".to_string(),
            version_before: 4,
            adf_before: serde_json::json!({"type": "doc", "content": []}),
            version_after: Some(5),
        };

        let result = rollback_page(&mut client, &snapshot);
        assert!(!result.success);
        assert!(result.conflict);
        assert!(result.error.unwrap().contains("version conflict"));
    }

    #[test]
    fn to_run_mode_maps_all_variants() {
        let cases = vec![
            PageRunMode::NoOp,
            PageRunMode::SimpleScopedProseUpdate {
                target_path: Some("/content/0/content/0/text".to_string()),
                markdown: "updated".to_string(),
            },
            PageRunMode::SimpleScopedTableCellUpdate {
                target_path: Some("/content/1/content/0/content/0/content/0/text".to_string()),
                text: "table".to_string(),
            },
            PageRunMode::ForbiddenTableOperation {
                target_path: "/content/1/content/0".to_string(),
                operation: TableOperation::RowAdd,
            },
            PageRunMode::SyntheticRouteConflict {
                prose_path: "/content/0/content/0/text".to_string(),
                table_path: "/content/1/content/0/content/0/content/0/text".to_string(),
            },
            PageRunMode::SyntheticTableShapeDrift {
                path: "/content/1/content/0".to_string(),
            },
        ];

        for mode in cases {
            let _ = to_run_mode(&mode);
        }
    }

    #[test]
    fn build_run_request_for_created_page_forces_bootstrap() {
        let request = MultiPageRequest {
            plan_id: "plan".to_string(),
            pages: vec![],
            rollback_on_failure: true,
            provenance: ProvenanceStamp {
                git_commit_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
                git_dirty: false,
                pipeline_version: PIPELINE_VERSION.to_string(),
                runtime_mode: RUNTIME_STUB.to_string(),
            },
            timestamp: "2026-03-15T00:00:00Z".to_string(),
        };
        let target = PageTarget {
            page_id: Some("page-1".to_string()),
            create: Some(CreatePageTarget {
                title: "child".to_string(),
                parent_page_id: "parent".to_string(),
                space_key: "PROJ".to_string(),
            }),
            edit_intent: "intent".to_string(),
            scope_selectors: vec![],
            run_mode: PageRunMode::NoOp,
            block_ops: vec![BlockOp::Insert {
                parent_path: "/content".to_string(),
                index: 0,
                block: serde_json::json!({"type":"paragraph"}),
            }],
            bootstrap_empty_page: false,
            depends_on: vec![],
        };

        let built = build_run_request(&request, &target, "run-1", "page-1", 0, true);
        assert!(built.bootstrap_empty_page);
    }
}
