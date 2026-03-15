use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use atlassy_confluence::{
    ConfluenceClient, CreatePageResponse, LiveConfluenceClient, StubConfluenceClient, StubPage,
};
use atlassy_contracts::{
    BlockOp, FLOW_OPTIMIZED, MultiPageRequest, MultiPageSummary, PATTERN_A, PIPELINE_VERSION,
    PageRunMode, PageTarget, ProvenanceStamp, RUNTIME_LIVE, RUNTIME_STUB, RunSummary,
};
use atlassy_pipeline::{MultiPageOrchestrator, Orchestrator, RunMode, RunRequest};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const TOOL_ATLASSY_RUN: &str = "atlassy_run";
const TOOL_ATLASSY_RUN_MULTI_PAGE: &str = "atlassy_run_multi_page";
const TOOL_ATLASSY_CREATE_SUBPAGE: &str = "atlassy_create_subpage";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = McpServer::from_env();

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(request) => server.handle_request(request),
            Err(error) => RpcResponse::error(Value::Null, -32700, format!("parse error: {error}")),
        };

        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct McpServer {
    runtime_backend: String,
    artifacts_dir: PathBuf,
}

impl McpServer {
    fn from_env() -> Self {
        let runtime_backend =
            std::env::var("ATLASSY_RUNTIME_BACKEND").unwrap_or_else(|_| RUNTIME_STUB.to_string());
        let artifacts_dir = std::env::var("ATLASSY_ARTIFACTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        Self {
            runtime_backend,
            artifacts_dir,
        }
    }

    #[cfg(test)]
    fn new_for_tests(runtime_backend: &str, artifacts_dir: PathBuf) -> Self {
        Self {
            runtime_backend: runtime_backend.to_string(),
            artifacts_dir,
        }
    }

    fn handle_request(&self, request: RpcRequest) -> RpcResponse {
        match self.dispatch(&request.method, request.params) {
            Ok(result) => RpcResponse::success(request.id, result),
            Err(error) => RpcResponse::error(request.id, -32602, error),
        }
    }

    fn dispatch(&self, method: &str, params: Value) -> Result<Value, String> {
        match method {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "atlassy-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            "tools/list" => Ok(json!({
                "tools": [
                    build_tool_descriptor(
                        TOOL_ATLASSY_RUN,
                        "Run a single-page Atlassy pipeline request",
                        atlassy_run_schema(),
                    ),
                    build_tool_descriptor(
                        TOOL_ATLASSY_RUN_MULTI_PAGE,
                        "Run an Atlassy multi-page orchestration request",
                        atlassy_run_multi_page_schema(),
                    ),
                    build_tool_descriptor(
                        TOOL_ATLASSY_CREATE_SUBPAGE,
                        "Create a Confluence subpage via configured backend",
                        atlassy_create_subpage_schema(),
                    )
                ]
            })),
            "tools/call" => {
                let call: ToolCallParams = serde_json::from_value(params)
                    .map_err(|error| format!("invalid tools/call params: {error}"))?;
                self.call_tool(call)
            }
            _ => Err(format!("unsupported MCP method `{method}`")),
        }
    }

    fn call_tool(&self, call: ToolCallParams) -> Result<Value, String> {
        match call.name.as_str() {
            TOOL_ATLASSY_RUN => {
                let input: AtlassyRunInput = serde_json::from_value(call.arguments)
                    .map_err(|error| format!("invalid `{TOOL_ATLASSY_RUN}` input: {error}"))?;
                let summary = self.run_tool_atlassy_run(input)?;
                Ok(tool_success_response(summary)?)
            }
            TOOL_ATLASSY_RUN_MULTI_PAGE => {
                let input: AtlassyRunMultiPageInput = serde_json::from_value(call.arguments)
                    .map_err(|error| {
                        format!("invalid `{TOOL_ATLASSY_RUN_MULTI_PAGE}` input: {error}")
                    })?;
                let summary = self.run_tool_atlassy_run_multi_page(input)?;
                Ok(tool_success_response(summary)?)
            }
            TOOL_ATLASSY_CREATE_SUBPAGE => {
                let input: AtlassyCreateSubpageInput = serde_json::from_value(call.arguments)
                    .map_err(|error| {
                        format!("invalid `{TOOL_ATLASSY_CREATE_SUBPAGE}` input: {error}")
                    })?;
                let response = self.run_tool_atlassy_create_subpage(input)?;
                Ok(tool_success_response(response)?)
            }
            _ => Err(format!("unknown MCP tool `{}`", call.name)),
        }
    }

    fn run_tool_atlassy_run(&self, input: AtlassyRunInput) -> Result<RunSummary, String> {
        let runtime_backend = runtime_backend_or_error(&self.runtime_backend)?;
        let request = input.into_run_request(runtime_backend);

        match runtime_backend {
            RUNTIME_STUB => {
                let mut pages = HashMap::new();
                pages.insert(
                    request.page_id.clone(),
                    StubPage {
                        version: 1,
                        adf: demo_page(),
                    },
                );
                let mut orchestrator =
                    Orchestrator::new(StubConfluenceClient::new(pages), &self.artifacts_dir);
                orchestrator
                    .run(request)
                    .map_err(|error| format!("pipeline failed: {error}"))
            }
            RUNTIME_LIVE => {
                let client = LiveConfluenceClient::from_env()
                    .map_err(|error| format!("live runtime startup failure: {error}"))?;
                let mut orchestrator = Orchestrator::new(client, &self.artifacts_dir);
                orchestrator
                    .run(request)
                    .map_err(|error| format!("pipeline failed: {error}"))
            }
            _ => Err(format!("unsupported runtime backend `{runtime_backend}`")),
        }
    }

    fn run_tool_atlassy_run_multi_page(
        &self,
        input: AtlassyRunMultiPageInput,
    ) -> Result<MultiPageSummary, String> {
        let runtime_backend = runtime_backend_or_error(&self.runtime_backend)?;
        let request = input.into_multi_page_request(runtime_backend);

        match runtime_backend {
            RUNTIME_STUB => {
                let pages = seed_stub_pages(&request.pages);
                let mut orchestrator = MultiPageOrchestrator::new(
                    StubConfluenceClient::new(pages),
                    &self.artifacts_dir,
                );
                orchestrator
                    .run(request)
                    .map_err(|error| format!("multi-page pipeline failed: {error}"))
            }
            RUNTIME_LIVE => {
                let client = LiveConfluenceClient::from_env()
                    .map_err(|error| format!("live runtime startup failure: {error}"))?;
                let mut orchestrator = MultiPageOrchestrator::new(client, &self.artifacts_dir);
                orchestrator
                    .run(request)
                    .map_err(|error| format!("multi-page pipeline failed: {error}"))
            }
            _ => Err(format!("unsupported runtime backend `{runtime_backend}`")),
        }
    }

    fn run_tool_atlassy_create_subpage(
        &self,
        input: AtlassyCreateSubpageInput,
    ) -> Result<CreatePageResponse, String> {
        let runtime_backend = runtime_backend_or_error(&self.runtime_backend)?;

        match runtime_backend {
            RUNTIME_STUB => {
                let mut pages = HashMap::new();
                pages.insert(
                    input.parent_page_id.clone(),
                    StubPage {
                        version: 1,
                        adf: demo_page(),
                    },
                );
                let mut client = StubConfluenceClient::new(pages);
                client
                    .create_page(&input.title, &input.parent_page_id, &input.space_key)
                    .map_err(|error| format!("create subpage failed: {error}"))
            }
            RUNTIME_LIVE => {
                let mut client = LiveConfluenceClient::from_env()
                    .map_err(|error| format!("live runtime startup failure: {error}"))?;
                client
                    .create_page(&input.title, &input.parent_page_id, &input.space_key)
                    .map_err(|error| format!("create subpage failed: {error}"))
            }
            _ => Err(format!("unsupported runtime backend `{runtime_backend}`")),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

impl RpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError { code, message }),
        }
    }
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
struct AtlassyRunInput {
    page_id: String,
    edit_intent: String,
    #[serde(default)]
    scope_selectors: Vec<String>,
    #[serde(default)]
    block_ops: Vec<BlockOp>,
    #[serde(default)]
    target_index: usize,
    #[serde(default)]
    force_verify_fail: bool,
    #[serde(default)]
    bootstrap_empty_page: bool,
    #[serde(default)]
    run_mode: Option<PageRunMode>,
    #[serde(default)]
    request_id: Option<String>,
}

impl AtlassyRunInput {
    fn into_run_request(self, runtime_backend: &str) -> RunRequest {
        let request_id = self
            .request_id
            .unwrap_or_else(|| format!("mcp-run-{}", monotonic_suffix()));

        RunRequest {
            request_id,
            page_id: self.page_id,
            edit_intent_hash: hash_edit_intent(&self.edit_intent),
            flow: FLOW_OPTIMIZED.to_string(),
            pattern: PATTERN_A.to_string(),
            edit_intent: self.edit_intent,
            scope_selectors: self.scope_selectors,
            timestamp: Utc::now().to_rfc3339(),
            provenance: collect_provenance(runtime_backend),
            run_mode: self
                .run_mode
                .as_ref()
                .map(page_run_mode_to_run_mode)
                .unwrap_or(RunMode::NoOp),
            target_index: self.target_index,
            block_ops: self.block_ops,
            force_verify_fail: self.force_verify_fail,
            bootstrap_empty_page: self.bootstrap_empty_page,
        }
    }
}

#[derive(Debug, Deserialize)]
struct AtlassyRunMultiPageInput {
    plan_id: String,
    pages: Vec<PageTarget>,
    #[serde(default)]
    rollback_on_failure: bool,
    #[serde(default)]
    timestamp: Option<String>,
}

impl AtlassyRunMultiPageInput {
    fn into_multi_page_request(self, runtime_backend: &str) -> MultiPageRequest {
        MultiPageRequest {
            plan_id: self.plan_id,
            pages: self.pages,
            rollback_on_failure: self.rollback_on_failure,
            provenance: collect_provenance(runtime_backend),
            timestamp: self.timestamp.unwrap_or_else(|| Utc::now().to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AtlassyCreateSubpageInput {
    parent_page_id: String,
    space_key: String,
    title: String,
}

fn page_run_mode_to_run_mode(mode: &PageRunMode) -> RunMode {
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

fn runtime_backend_or_error(runtime_backend: &str) -> Result<&str, String> {
    if matches!(runtime_backend, RUNTIME_STUB | RUNTIME_LIVE) {
        return Ok(runtime_backend);
    }

    Err(format!(
        "invalid runtime backend `{runtime_backend}`; expected `{RUNTIME_STUB}` or `{RUNTIME_LIVE}`"
    ))
}

fn collect_provenance(runtime_mode: &str) -> ProvenanceStamp {
    let git_commit_sha = std::env::var("ATLASSY_GIT_COMMIT_SHA")
        .ok()
        .filter(|sha| is_valid_git_sha(sha))
        .unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());
    let git_dirty = std::env::var("ATLASSY_GIT_DIRTY")
        .ok()
        .is_some_and(|value| value == "true");

    ProvenanceStamp {
        git_commit_sha,
        git_dirty,
        pipeline_version: PIPELINE_VERSION.to_string(),
        runtime_mode: runtime_mode.to_string(),
    }
}

fn is_valid_git_sha(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn monotonic_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn hash_edit_intent(edit_intent: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    edit_intent.hash(&mut hasher);
    format!("h{:016x}", hasher.finish())
}

fn demo_page() -> Value {
    json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": "Overview"}]
            },
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": "Initial prose body"}]
            }
        ]
    })
}

fn seed_stub_pages(pages: &[PageTarget]) -> HashMap<String, StubPage> {
    let mut seeded = HashMap::new();

    for target in pages {
        if let Some(page_id) = &target.page_id {
            seeded.entry(page_id.clone()).or_insert_with(|| StubPage {
                version: 1,
                adf: demo_page(),
            });
        }
        if let Some(create) = &target.create {
            seeded
                .entry(create.parent_page_id.clone())
                .or_insert_with(|| StubPage {
                    version: 1,
                    adf: demo_page(),
                });
        }
    }

    seeded
}

fn build_tool_descriptor(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

fn atlassy_run_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "page_id": {"type": "string"},
            "edit_intent": {"type": "string"},
            "scope_selectors": {"type": "array", "items": {"type": "string"}},
            "block_ops": {"type": "array", "items": {"type": "object"}},
            "target_index": {"type": "integer", "minimum": 0},
            "force_verify_fail": {"type": "boolean"},
            "bootstrap_empty_page": {"type": "boolean"},
            "run_mode": {
                "type": "object",
                "description": "PageRunMode payload from atlassy-contracts"
            },
            "request_id": {"type": "string"}
        },
        "required": ["page_id", "edit_intent"],
        "additionalProperties": false
    })
}

fn atlassy_run_multi_page_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "plan_id": {"type": "string"},
            "pages": {"type": "array", "items": {"type": "object"}},
            "rollback_on_failure": {"type": "boolean"},
            "timestamp": {"type": "string"}
        },
        "required": ["plan_id", "pages"],
        "additionalProperties": false
    })
}

fn atlassy_create_subpage_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "parent_page_id": {"type": "string"},
            "space_key": {"type": "string"},
            "title": {"type": "string"}
        },
        "required": ["parent_page_id", "space_key", "title"],
        "additionalProperties": false
    })
}

fn tool_success_response<T: Serialize>(value: T) -> Result<Value, String> {
    let structured = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let text = serde_json::to_string(&structured).map_err(|error| error.to_string())?;
    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
        "isError": false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_list_contains_expected_tool_set() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let result = server
            .dispatch("tools/list", json!({}))
            .expect("tools/list should succeed");
        let tools = result["tools"].as_array().expect("tools should be array");
        let names = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .collect::<Vec<_>>();

        assert!(names.contains(&TOOL_ATLASSY_RUN));
        assert!(names.contains(&TOOL_ATLASSY_RUN_MULTI_PAGE));
        assert!(names.contains(&TOOL_ATLASSY_CREATE_SUBPAGE));
    }

    #[test]
    fn atlassy_run_tool_executes_pipeline_with_stub_backend() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let result = server
            .dispatch(
                "tools/call",
                json!({
                    "name": TOOL_ATLASSY_RUN,
                    "arguments": {
                        "page_id": "18841604",
                        "edit_intent": "Update section",
                        "scope_selectors": [],
                        "run_mode": {
                            "mode": "simple_scoped_prose_update",
                            "target_path": "/content/1/content/0/text",
                            "markdown": "Updated prose body"
                        }
                    }
                }),
            )
            .expect("tool call should succeed");

        assert_eq!(result["isError"], json!(false));
        assert_eq!(
            result["structuredContent"]["success"],
            json!(true),
            "run summary should indicate success"
        );
    }

    #[test]
    fn atlassy_create_subpage_tool_uses_stub_backend() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let result = server
            .dispatch(
                "tools/call",
                json!({
                    "name": TOOL_ATLASSY_CREATE_SUBPAGE,
                    "arguments": {
                        "parent_page_id": "18841604",
                        "space_key": "PROJ",
                        "title": "New Child Page"
                    }
                }),
            )
            .expect("create subpage tool call should succeed");

        assert_eq!(result["isError"], json!(false));
        assert!(result["structuredContent"]["page_id"].is_string());
    }

    #[test]
    fn invalid_tool_call_input_returns_error() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let error = server
            .dispatch(
                "tools/call",
                json!({
                    "name": TOOL_ATLASSY_RUN,
                    "arguments": {
                        "edit_intent": "missing page_id"
                    }
                }),
            )
            .expect_err("invalid tool call should return error");

        assert!(
            error.contains("invalid `atlassy_run` input"),
            "expected input validation error, got: {error}"
        );
    }

    #[test]
    fn rpc_wrapper_returns_error_object_for_invalid_tool_input() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let response = server.handle_request(RpcRequest {
            id: json!(1),
            method: "tools/call".to_string(),
            params: json!({
                "name": TOOL_ATLASSY_RUN,
                "arguments": {}
            }),
        });

        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn collect_provenance_returns_contract_compatible_values() {
        let provenance = collect_provenance(RUNTIME_STUB);
        assert_eq!(provenance.runtime_mode, RUNTIME_STUB);
        assert_eq!(provenance.pipeline_version, PIPELINE_VERSION);
        assert!(is_valid_git_sha(&provenance.git_commit_sha));
    }

    #[test]
    fn seed_stub_pages_includes_existing_and_create_parents() {
        let pages = vec![
            PageTarget {
                page_id: Some("page-a".to_string()),
                create: None,
                edit_intent: "update a".to_string(),
                scope_selectors: vec![],
                run_mode: PageRunMode::NoOp,
                block_ops: vec![],
                bootstrap_empty_page: false,
                depends_on: vec![],
            },
            PageTarget {
                page_id: None,
                create: Some(atlassy_contracts::CreatePageTarget {
                    title: "child".to_string(),
                    parent_page_id: "parent-1".to_string(),
                    space_key: "PROJ".to_string(),
                }),
                edit_intent: "create child".to_string(),
                scope_selectors: vec![],
                run_mode: PageRunMode::NoOp,
                block_ops: vec![],
                bootstrap_empty_page: false,
                depends_on: vec![],
            },
        ];

        let seeded = seed_stub_pages(&pages);
        assert!(seeded.contains_key("page-a"));
        assert!(seeded.contains_key("parent-1"));
    }

    #[test]
    fn tool_call_invalid_name_is_rejected() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let error = server
            .dispatch(
                "tools/call",
                json!({
                    "name": "unknown_tool",
                    "arguments": {}
                }),
            )
            .expect_err("unknown tool should return error");
        assert!(error.contains("unknown MCP tool"));
    }

    #[test]
    fn dispatch_initialize_returns_server_metadata() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let result = server
            .dispatch("initialize", json!({}))
            .expect("initialize should succeed");
        assert_eq!(result["serverInfo"]["name"], json!("atlassy-mcp"));
    }

    #[test]
    fn multi_page_tool_round_trip_stub_backend() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let server = McpServer::new_for_tests(RUNTIME_STUB, temp.path().to_path_buf());

        let result = server
            .dispatch(
                "tools/call",
                json!({
                    "name": TOOL_ATLASSY_RUN_MULTI_PAGE,
                    "arguments": {
                        "plan_id": "plan-1",
                        "pages": [
                            {
                                "page_id": "18841604",
                                "edit_intent": "update",
                                "scope_selectors": [],
                                "run_mode": {"mode": "no_op"},
                                "block_ops": [],
                                "bootstrap_empty_page": false,
                                "depends_on": []
                            }
                        ],
                        "rollback_on_failure": false
                    }
                }),
            )
            .expect("multi-page call should succeed");

        assert_eq!(result["isError"], json!(false));
        assert_eq!(result["structuredContent"]["plan_id"], json!("plan-1"));
    }

    #[test]
    fn hash_edit_intent_is_stable() {
        assert_eq!(hash_edit_intent("same"), hash_edit_intent("same"));
    }

    #[test]
    fn runtime_backend_validation_rejects_invalid_value() {
        let error = runtime_backend_or_error("invalid-backend").expect_err("should fail");
        assert!(error.contains("invalid runtime backend"));
    }

    #[test]
    fn demo_page_contains_expected_shape() {
        let page = demo_page();
        assert_eq!(page["type"], json!("doc"));
        assert!(page["content"].is_array());
    }

    #[test]
    fn tool_success_response_contains_structured_content() {
        let response = tool_success_response(json!({"ok": true})).expect("should serialize");
        assert_eq!(response["isError"], json!(false));
        assert_eq!(response["structuredContent"], json!({"ok": true}));
    }

    #[test]
    fn run_multi_page_input_builds_contract_request() {
        let input = AtlassyRunMultiPageInput {
            plan_id: "plan-1".to_string(),
            pages: vec![],
            rollback_on_failure: true,
            timestamp: Some("2026-03-15T00:00:00Z".to_string()),
        };

        let request = input.into_multi_page_request(RUNTIME_STUB);
        assert_eq!(request.plan_id, "plan-1");
        assert!(request.rollback_on_failure);
        assert_eq!(request.timestamp, "2026-03-15T00:00:00Z");
    }

    #[test]
    fn run_input_builds_request_with_defaults() {
        let input = AtlassyRunInput {
            page_id: "18841604".to_string(),
            edit_intent: "update".to_string(),
            scope_selectors: vec![],
            block_ops: vec![],
            target_index: 0,
            force_verify_fail: false,
            bootstrap_empty_page: false,
            run_mode: None,
            request_id: None,
        };

        let request = input.into_run_request(RUNTIME_STUB);
        assert_eq!(request.page_id, "18841604");
        assert!(matches!(request.run_mode, RunMode::NoOp));
    }

    #[test]
    fn rollback_results_type_is_usable_in_mcp_crate() {
        let result = atlassy_contracts::RollbackResult {
            page_id: "page-1".to_string(),
            success: true,
            conflict: false,
            error: None,
        };
        assert!(result.success);
    }
}
