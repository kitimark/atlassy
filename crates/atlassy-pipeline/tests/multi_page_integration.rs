use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use atlassy_confluence::{
    ConfluenceClient, ConfluenceError, CreatePageResponse, FetchPageResponse, PublishPageResponse,
    StubConfluenceClient, StubPage,
};
use atlassy_contracts::{
    BlockOp, ErrorCode, MultiPageRequest, PageRunMode, PageTarget, PipelineState, ProvenanceStamp,
    FLOW_OPTIMIZED, PATTERN_A, PIPELINE_VERSION, RUNTIME_STUB,
};
use atlassy_pipeline::{MultiPageOrchestrator, Orchestrator, PipelineError, RunMode, RunRequest};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load_fixture(name: &str) -> serde_json::Value {
    let text = fs::read_to_string(fixture_path(name)).expect("fixture file should be readable");
    serde_json::from_str(&text).expect("fixture should be valid JSON")
}

fn sample_provenance() -> ProvenanceStamp {
    ProvenanceStamp {
        git_commit_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
        git_dirty: false,
        pipeline_version: PIPELINE_VERSION.to_string(),
        runtime_mode: RUNTIME_STUB.to_string(),
    }
}

fn sample_multi_request(pages: Vec<PageTarget>, rollback_on_failure: bool) -> MultiPageRequest {
    MultiPageRequest {
        plan_id: "plan-1".to_string(),
        pages,
        rollback_on_failure,
        provenance: sample_provenance(),
        timestamp: "2026-03-15T12:00:00Z".to_string(),
    }
}

fn existing_page_target(page_id: &str, markdown: &str, depends_on: Vec<&str>) -> PageTarget {
    PageTarget {
        page_id: Some(page_id.to_string()),
        create: None,
        edit_intent: format!("update page {page_id}"),
        scope_selectors: vec![],
        run_mode: PageRunMode::SimpleScopedProseUpdate {
            target_path: Some("/content/1/content/0/text".to_string()),
            markdown: markdown.to_string(),
        },
        block_ops: vec![],
        bootstrap_empty_page: false,
        depends_on: depends_on.into_iter().map(|dep| dep.to_string()).collect(),
    }
}

fn failing_page_target(page_id: &str, depends_on: Vec<&str>) -> PageTarget {
    PageTarget {
        page_id: Some(page_id.to_string()),
        create: None,
        edit_intent: format!("fail page {page_id}"),
        scope_selectors: vec![],
        run_mode: PageRunMode::SimpleScopedProseUpdate {
            target_path: Some("/content/99/content/0/text".to_string()),
            markdown: "invalid".to_string(),
        },
        block_ops: vec![],
        bootstrap_empty_page: false,
        depends_on: depends_on.into_iter().map(|dep| dep.to_string()).collect(),
    }
}

fn seeded_pages(ids: &[&str]) -> HashMap<String, StubPage> {
    let mut pages = HashMap::new();
    for id in ids {
        pages.insert(
            (*id).to_string(),
            StubPage {
                version: 7,
                adf: load_fixture("prose_only_adf.json"),
            },
        );
    }
    pages
}

fn response_text(response: &FetchPageResponse) -> String {
    response
        .adf
        .pointer("/content/1/content/0/text")
        .and_then(serde_json::Value::as_str)
        .expect("paragraph text should exist")
        .to_string()
}

#[derive(Debug, Clone)]
struct ConflictAfterBFailureClient {
    pages: HashMap<String, StubPage>,
    publish_attempts: usize,
    conflict_injected: bool,
    page_b_fetch_count: usize,
}

impl ConflictAfterBFailureClient {
    fn new(pages: HashMap<String, StubPage>) -> Self {
        Self {
            pages,
            publish_attempts: 0,
            conflict_injected: false,
            page_b_fetch_count: 0,
        }
    }
}

impl ConfluenceClient for ConflictAfterBFailureClient {
    fn fetch_page(&mut self, page_id: &str) -> Result<FetchPageResponse, ConfluenceError> {
        if page_id == "page-b" {
            self.page_b_fetch_count += 1;
        }

        if page_id == "page-b" && self.page_b_fetch_count >= 2 && !self.conflict_injected {
            if let Some(page_a) = self.pages.get_mut("page-a") {
                page_a.version += 1;
                page_a.adf = serde_json::json!({
                    "type": "doc",
                    "version": 1,
                    "content": [{
                        "type": "paragraph",
                        "content": [{"type": "text", "text": "Concurrent edit"}]
                    }]
                });
            }
            self.conflict_injected = true;
            return Err(ConfluenceError::Transport(
                "simulated page-b fetch failure".to_string(),
            ));
        }

        let page = self
            .pages
            .get(page_id)
            .ok_or_else(|| ConfluenceError::NotFound(page_id.to_string()))?;
        Ok(FetchPageResponse {
            page_version: page.version,
            adf: page.adf.clone(),
        })
    }

    fn publish_page(
        &mut self,
        page_id: &str,
        page_version: u64,
        candidate_adf: &serde_json::Value,
    ) -> Result<PublishPageResponse, ConfluenceError> {
        self.publish_attempts += 1;
        let page = self
            .pages
            .get_mut(page_id)
            .ok_or_else(|| ConfluenceError::NotFound(page_id.to_string()))?;
        if page.version != page_version {
            return Err(ConfluenceError::Conflict(page_id.to_string()));
        }

        page.version += 1;
        page.adf = candidate_adf.clone();
        Ok(PublishPageResponse {
            new_version: page.version,
        })
    }

    fn create_page(
        &mut self,
        title: &str,
        parent_page_id: &str,
        _space_key: &str,
    ) -> Result<CreatePageResponse, ConfluenceError> {
        if !self.pages.contains_key(parent_page_id) {
            return Err(ConfluenceError::NotFound(parent_page_id.to_string()));
        }

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        title.hash(&mut hasher);
        let page_id = format!("stub-{}", hasher.finish());

        self.pages.insert(
            page_id.clone(),
            StubPage {
                version: 1,
                adf: serde_json::json!({"type": "doc", "version": 1, "content": []}),
            },
        );

        Ok(CreatePageResponse {
            page_id,
            page_version: 1,
        })
    }

    fn publish_attempts(&self) -> usize {
        self.publish_attempts
    }
}

#[test]
fn multi_page_two_pages_succeed_and_publish() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let pages = seeded_pages(&["page-a", "page-b"]);
    let mut orchestrator =
        MultiPageOrchestrator::new(StubConfluenceClient::new(pages), temp.path());

    let request = sample_multi_request(
        vec![
            existing_page_target("page-a", "updated a", vec![]),
            existing_page_target("page-b", "updated b", vec![]),
        ],
        true,
    );

    let summary = orchestrator
        .run(request)
        .expect("multi-page run should succeed");
    assert!(summary.success);
    assert_eq!(summary.total_pages, 2);
    assert_eq!(summary.succeeded_pages, 2);
    assert_eq!(summary.failed_pages, 0);
    assert_eq!(summary.rollback_results.len(), 0);
    assert_eq!(summary.page_results.len(), 2);
    assert_eq!(orchestrator.client().publish_attempts(), 2);

    let page_a = orchestrator
        .client_mut()
        .fetch_page("page-a")
        .expect("page a should be fetchable");
    let page_b = orchestrator
        .client_mut()
        .fetch_page("page-b")
        .expect("page b should be fetchable");
    assert_eq!(response_text(&page_a), "updated a");
    assert_eq!(response_text(&page_b), "updated b");
}

#[test]
fn multi_page_failure_rolls_back_prior_success() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let pages = seeded_pages(&["page-a", "page-b"]);
    let mut orchestrator =
        MultiPageOrchestrator::new(StubConfluenceClient::new(pages), temp.path());

    let request = sample_multi_request(
        vec![
            existing_page_target("page-a", "updated a", vec![]),
            failing_page_target("page-b", vec!["page-a"]),
        ],
        true,
    );

    let summary = orchestrator
        .run(request)
        .expect("multi-page run should return summary on partial failure");
    assert!(!summary.success);
    assert_eq!(summary.page_results.len(), 2);
    assert_eq!(summary.rollback_results.len(), 1);
    assert!(summary.rollback_results[0].success);
    assert!(!summary.rollback_results[0].conflict);

    let page_a = orchestrator
        .client_mut()
        .fetch_page("page-a")
        .expect("page a should still exist");
    assert_eq!(response_text(&page_a), "Initial prose body");
}

#[test]
fn multi_page_failure_reports_rollback_conflict_for_concurrent_edit() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let pages = seeded_pages(&["page-a", "page-b"]);
    let client = ConflictAfterBFailureClient::new(pages);
    let mut orchestrator = MultiPageOrchestrator::new(client, temp.path());

    let request = sample_multi_request(
        vec![
            existing_page_target("page-a", "updated a", vec![]),
            existing_page_target("page-b", "updated b", vec!["page-a"]),
        ],
        true,
    );

    let summary = orchestrator
        .run(request)
        .expect("run should return summary when page-b fails");
    assert!(!summary.success);
    assert_eq!(summary.rollback_results.len(), 1);
    assert!(!summary.rollback_results[0].success);
    assert!(summary.rollback_results[0].conflict);
    assert_eq!(
        summary.rollback_results[0].error.as_deref(),
        Some(ErrorCode::RollbackConflict.as_str())
    );
}

#[test]
fn multi_page_create_subpage_with_content_applies_block_ops() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut pages = seeded_pages(&["parent-1"]);
    pages.entry("parent-1".to_string()).or_insert(StubPage {
        version: 7,
        adf: load_fixture("prose_only_adf.json"),
    });
    let mut orchestrator =
        MultiPageOrchestrator::new(StubConfluenceClient::new(pages), temp.path());

    let request = sample_multi_request(
        vec![PageTarget {
            page_id: None,
            create: Some(atlassy_contracts::CreatePageTarget {
                title: "Report".to_string(),
                parent_page_id: "parent-1".to_string(),
                space_key: "PROJ".to_string(),
            }),
            edit_intent: "create report".to_string(),
            scope_selectors: vec![],
            run_mode: PageRunMode::NoOp,
            block_ops: vec![BlockOp::InsertSection {
                parent_path: "/content".to_string(),
                index: 0,
                heading_level: 2,
                heading_text: "Overview".to_string(),
                body_blocks: vec![serde_json::json!({
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Body"}]
                })],
            }],
            bootstrap_empty_page: false,
            depends_on: vec![],
        }],
        true,
    );

    let summary = orchestrator
        .run(request)
        .expect("create and populate page should succeed");
    assert!(summary.success);
    assert_eq!(summary.page_results.len(), 1);
    assert!(summary.page_results[0].created);

    let created_page_id = summary.page_results[0].page_id.clone();
    let created_page = orchestrator
        .client_mut()
        .fetch_page(&created_page_id)
        .expect("created page should be fetchable");
    assert_eq!(
        created_page
            .adf
            .pointer("/content/0/type")
            .and_then(serde_json::Value::as_str),
        Some("heading")
    );
}

#[test]
fn multi_page_dependency_order_is_respected() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let pages = seeded_pages(&["page-a", "page-b"]);
    let mut orchestrator =
        MultiPageOrchestrator::new(StubConfluenceClient::new(pages), temp.path());

    let request = sample_multi_request(
        vec![
            existing_page_target("page-b", "updated b", vec!["page-a"]),
            existing_page_target("page-a", "updated a", vec![]),
        ],
        true,
    );

    let summary = orchestrator.run(request).expect("run should succeed");
    assert!(summary.success);
    assert_eq!(summary.page_results.len(), 2);
    assert_eq!(summary.page_results[0].page_id, "page-a");
    assert_eq!(summary.page_results[1].page_id, "page-b");
}

#[test]
fn multi_page_cycle_is_rejected_before_execution() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let pages = seeded_pages(&["page-a", "page-b"]);
    let mut orchestrator =
        MultiPageOrchestrator::new(StubConfluenceClient::new(pages), temp.path());

    let request = sample_multi_request(
        vec![
            existing_page_target("page-a", "updated a", vec!["page-b"]),
            existing_page_target("page-b", "updated b", vec!["page-a"]),
        ],
        true,
    );

    let error = orchestrator
        .run(request)
        .expect_err("dependency cycle should fail validation");
    match error {
        PipelineError::Hard { state, code, .. } => {
            assert_eq!(state, PipelineState::Fetch);
            assert_eq!(code, ErrorCode::DependencyCycle);
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(orchestrator.client().publish_attempts(), 0);
}

#[test]
fn multi_page_failure_without_rollback_leaves_prior_success_published() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let pages = seeded_pages(&["page-a", "page-b"]);
    let mut orchestrator =
        MultiPageOrchestrator::new(StubConfluenceClient::new(pages), temp.path());

    let request = sample_multi_request(
        vec![
            existing_page_target("page-a", "updated a", vec![]),
            failing_page_target("page-b", vec!["page-a"]),
        ],
        false,
    );

    let summary = orchestrator
        .run(request)
        .expect("partial failure should still return summary");
    assert!(!summary.success);
    assert_eq!(summary.rollback_results.len(), 0);

    let page_a = orchestrator
        .client_mut()
        .fetch_page("page-a")
        .expect("page a should exist");
    assert_eq!(response_text(&page_a), "updated a");
}

fn sample_single_request(run_id: &str, page_id: &str) -> RunRequest {
    RunRequest {
        request_id: run_id.to_string(),
        page_id: page_id.to_string(),
        edit_intent: "single-page update".to_string(),
        edit_intent_hash: "hash-single".to_string(),
        flow: FLOW_OPTIMIZED.to_string(),
        pattern: PATTERN_A.to_string(),
        scope_selectors: vec![],
        timestamp: "2026-03-15T12:00:00Z".to_string(),
        provenance: sample_provenance(),
        run_mode: RunMode::SimpleScopedProseUpdate {
            target_path: Some("/content/1/content/0/text".to_string()),
            markdown: "single updated".to_string(),
        },
        target_index: 0,
        block_ops: vec![],
        force_verify_fail: false,
        bootstrap_empty_page: false,
    }
}

#[test]
fn single_page_orchestrator_behavior_remains_unchanged() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let mut pages = HashMap::new();
    pages.insert(
        "single-page".to_string(),
        StubPage {
            version: 7,
            adf: load_fixture("prose_only_adf.json"),
        },
    );

    let mut orchestrator = Orchestrator::new(StubConfluenceClient::new(pages), temp.path());
    let summary = orchestrator
        .run(sample_single_request("single-run", "single-page"))
        .expect("single-page run should still succeed");

    assert!(summary.success);
    assert_eq!(summary.page_id, "single-page");
    assert_eq!(orchestrator.client().publish_attempts(), 1);
}
