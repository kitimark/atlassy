## 1. Contract Foundations

- [ ] 1.1 Add `ERR_BOOTSTRAP_REQUIRED` and `ERR_BOOTSTRAP_INVALID_STATE` error constants to `atlassy-contracts` following the existing `ERR_SCREAMING_SNAKE_CASE` pattern
- [ ] 1.2 Add `CreatePageResponse { page_id: String, page_version: u64 }` to `atlassy-contracts` or `atlassy-confluence` alongside existing `FetchPageResponse` and `PublishPageResponse`
- [ ] 1.3 Add `empty_page_detected: bool` and `bootstrap_applied: bool` fields to `RunSummary` in `atlassy-contracts`, defaulting both to `false`
- [ ] 1.4 Add `bootstrap_empty_page: bool` field to `RunRequest` in `atlassy-pipeline`, defaulting to `false`

## 2. Confluence Client create_page Contract

- [ ] 2.1 Add `create_page(&mut self, title: &str, parent_page_id: &str, space_key: &str) -> Result<CreatePageResponse, ConfluenceError>` to the `ConfluenceClient` trait
- [ ] 2.2 Implement `create_page` on `StubConfluenceClient`: insert new page into in-memory `pages` HashMap with `version: 1` and empty ADF doc, return synthetic `page_id` derived from title, reject duplicate titles
- [ ] 2.3 Add `build_create_payload(title, parent_page_id, space_key)` to `LiveConfluenceClient` following the `build_publish_payload` pattern: include `type: "page"`, `ancestors`, `space.key`, `body.atlas_doc_format` with empty ADF serialized as string, no `version` or `id` fields
- [ ] 2.4 Add `content_collection_endpoint()` helper to `LiveConfluenceClient` returning `{base_url}/wiki/rest/api/content`
- [ ] 2.5 Implement `create_page` on `LiveConfluenceClient`: POST to collection endpoint, map 404 to `NotFound`, map non-success to `Transport` via `parse_http_error`, deserialize response for `page_id` and `page_version`
- [ ] 2.6 Add unit tests for `build_create_payload`: verify space key, ancestors, atlas_doc_format encoding, absence of version field

## 3. CLI create-subpage Command

- [ ] 3.1 Add `CreateSubpage { parent_page_id, space_key, title, runtime_backend }` variant to `Commands` enum in `atlassy-cli`
- [ ] 3.2 Implement `CreateSubpage` handler in `main()` match arm: instantiate stub or live client based on backend, call `create_page`, print JSON result with `page_id` and `page_version`
- [ ] 3.3 Handle live startup errors for `CreateSubpage` using the existing `map_live_startup_error` pattern
- [ ] 3.4 Verify existing `Run`, `RunBatch`, and `RunReadiness` commands are unchanged by the new variant

## 4. Empty-Page Detection

- [ ] 4.1 Add `pub fn is_page_effectively_empty(adf: &Value) -> bool` to `atlassy-adf`: return true when top-level `content` is absent, empty, or contains only paragraphs whose content is absent, empty, or contains only empty-string text nodes
- [ ] 4.2 Add `pub fn bootstrap_scaffold() -> Value` to `atlassy-adf`: return the minimal ADF doc with one level-2 heading (empty text) and one paragraph (empty text)
- [ ] 4.3 Add unit tests for `is_page_effectively_empty`: empty content array, single empty paragraph, paragraph with empty text, paragraph with `localId` but no text, non-empty paragraph, heading with text, table node, panel node
- [ ] 4.4 Add unit test for `bootstrap_scaffold`: verify all content nodes are `editable_prose` route types (heading, paragraph), no `table_adf` or `locked_structural` nodes

## 5. Bootstrap Pipeline Integration

- [ ] 5.1 Add `--bootstrap-empty-page` flag to `Commands::Run` in CLI, map to `RunRequest.bootstrap_empty_page`
- [ ] 5.2 Add optional `bootstrap_empty_page` field to `ManifestRunEntry` for batch manifest support, map to `RunRequest` in `execute_manifest_runs`
- [ ] 5.3 Implement bootstrap detection block in `Orchestrator::run_internal` between fetch and classify: call `is_page_effectively_empty` on `fetch.payload.scoped_adf`, evaluate the four-path matrix, set `summary.empty_page_detected` and `summary.bootstrap_applied`
- [ ] 5.4 On empty page without flag: return `PipelineError::Hard` with `ERR_BOOTSTRAP_REQUIRED` at `PipelineState::Fetch`
- [ ] 5.5 On non-empty page with flag: return `PipelineError::Hard` with `ERR_BOOTSTRAP_INVALID_STATE` at `PipelineState::Fetch`
- [ ] 5.6 On empty page with flag: replace `fetch.payload.scoped_adf` with `bootstrap_scaffold()`, rebuild `node_path_index` and `allowed_scope_paths` from the scaffolded ADF
- [ ] 5.7 Add integration tests: empty page + no flag (expect `ERR_BOOTSTRAP_REQUIRED`), empty page + flag (expect pipeline completes), non-empty page + flag (expect `ERR_BOOTSTRAP_INVALID_STATE`), non-empty page + no flag (expect unchanged behavior)

## 6. Gate 7 Readiness Evaluation

- [ ] 6.1 Add `gate_7_lifecycle_enablement_validation` to `evaluate_readiness_gates` in `atlassy-cli`: check batch summaries for at least one run each with bootstrap-required failure, bootstrap success, and bootstrap-on-non-empty failure evidence
- [ ] 6.2 Add lifecycle metadata marker to batch manifest for create-subpage evidence (e.g., `lifecycle_create_subpage_validated: bool` in `BatchManifestMetadata`)
- [ ] 6.3 Include Gate 7 result in `gates` vec with mandatory=true, `qa_owner` role, and lifecycle evidence references
- [ ] 6.4 Update existing fixture manifests to include lifecycle fields (defaulting to false) so existing batch tests continue to pass with Gate 7 evaluated but non-blocking on legacy fixtures
- [ ] 6.5 Add fixture-backed test: batch with complete lifecycle evidence passes Gate 7, batch without lifecycle evidence fails Gate 7 and produces `iterate` recommendation

## 7. Validation and Quality

- [ ] 7.1 Run `cargo fmt --all` and fix any formatting issues
- [ ] 7.2 Run `cargo clippy --workspace --all-targets -- -D warnings` and resolve all warnings
- [ ] 7.3 Run `cargo test --workspace` and verify all existing and new tests pass
- [ ] 7.4 Verify `RunSummary` telemetry validation (`validate_run_summary_telemetry`) handles new `empty_page_detected` and `bootstrap_applied` fields without breaking completeness checks
