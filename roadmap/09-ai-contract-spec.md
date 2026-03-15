# AI-Facing Pipeline Contract

## Contract Metadata

- `contract_version`: `2.0.0`
- `pipeline_version`: `v1`
- `canonical_representation`: `adf`
- `routes`: `editable_prose | table_adf | locked_structural`
- `path_format`: `json_pointer` (RFC 6901)
- `operation_kinds`: `replace | insert | remove`
- `state_order`: `fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`

Contract version `1.0.0` (Foundation) is fully backward compatible. Structural additions (2.0.0) are additive; existing `replace`-only payloads remain valid.

## Global Rules

- Every state payload includes: `request_id`, `page_id`, `state`, `timestamp`.
- Keys use `snake_case`; enums use fixed values only.
- Arrays are always present (use `[]` instead of `null`).
- `changed_paths` must be unique and lexicographically sorted.
- Any mutation outside `allowed_scope_paths` is a hard error.

## Shared Types

- `node_ref`: `{ "path": "/...", "node_type": "...", "route": "..." }`
- `error`: `{ "code": "ERR_*", "message": "...", "recovery": "..." }`
- `diagnostics`: `{ "warnings": [], "errors": [], "metrics": {} }`

## Error Taxonomy

### Foundation Error Codes

- `ERR_SCOPE_MISS`: requested target cannot be resolved.
- `ERR_ROUTE_VIOLATION`: node edited through an invalid route.
- `ERR_SCHEMA_INVALID`: candidate ADF fails schema validation.
- `ERR_OUT_OF_SCOPE_MUTATION`: change detected outside allowed scope.
- `ERR_LOCKED_NODE_MUTATION`: locked node fingerprint changed.
- `ERR_TABLE_SHAPE_CHANGE`: table row/column topology changed in v1.
- `ERR_CONFLICT_RETRY_EXHAUSTED`: publish conflict after one scoped retry.
- `ERR_BOOTSTRAP_REQUIRED`: page is effectively empty and `--bootstrap-empty-page` was not provided.
- `ERR_BOOTSTRAP_INVALID_STATE`: `--bootstrap-empty-page` was provided but page is not empty.

### Structural Error Codes

- `ERR_INSERT_POSITION_INVALID`: insert target position does not exist, is out of bounds, or would violate ADF schema constraints.
- `ERR_REMOVE_ANCHOR_MISSING`: removal target block does not exist at the specified path.
- `ERR_POST_MUTATION_SCHEMA_INVALID`: resulting ADF after insert/delete operations fails schema validation.
- `ERR_MULTI_PAGE_PARTIAL_FAILURE`: one or more pages in a multi-page operation failed; rollback initiated (Phase 8).

## State Contracts

### `fetch`

- Inputs: `page_id`, `edit_intent`, `scope_selectors`.
- Outputs: `scoped_adf`, `page_version`, `allowed_scope_paths`, `node_path_index`.
- Postconditions: no full-page fetch unless `scope_resolution_failed=true`.

### `classify`

- Inputs: `scoped_adf`.
- Outputs: `node_manifest[]` with route labels and lock reasons.
- Postconditions: every node has exactly one route.

### `extract_prose`

- Inputs: `node_manifest`, `scoped_adf`.
- Outputs: `markdown_blocks[]`, `md_to_adf_map[]`.
- Postconditions: only `editable_prose` is converted.

### `md_assist_edit`

- Inputs: `markdown_blocks[]`, `edit_intent`.
- Outputs: `edited_markdown_blocks[]`, `prose_changed_paths[]`.
- Constraints: no new top-level block types; no boundary expansion outside mapped prose blocks.

### `adf_table_edit`

- Inputs: `table_nodes[]`, `edit_intent`.
- Outputs: `table_candidate_nodes[]`, `table_changed_paths[]`.
- Allowed ops (v1): `cell_text_update`.
- Forbidden ops (v1): `row_add`, `row_remove`, `col_add`, `col_remove`, `merge_cells`, `split_cells`, `table_attr_change`.
- Violation: return `ERR_TABLE_SHAPE_CHANGE`.

### `merge_candidates`

- Inputs: prose and table candidate sets.
- Outputs: `merged_candidate_nodes[]`, `changed_paths[]`.
- Postconditions: path uniqueness is enforced; cross-route conflicts fail fast.

### `patch`

- Inputs: `scoped_adf`, `merged_candidate_nodes[]`.
- Outputs: `patch_ops[]`, `candidate_page_adf`.
- Constraints: path-targeted ops only; whole-body rewrite is disallowed.
- Structural additions:
  - Each `patch_op` includes `operation_kind`: `replace | insert | remove`.
  - `insert` ops include `parent_path`, `index`, and `value` (the ADF node to insert).
  - `remove` ops include `target_path` (the ADF node to delete).
  - Operations are sorted by descending document position and applied in reverse order (D-020).
  - Outputs add: `insert_count`, `remove_count`.

### `verify`

- Inputs: `original_scoped_adf`, `candidate_page_adf`, `allowed_scope_paths`.
- Outputs: `verify_result` (`pass|fail`), `diagnostics`.
- Required checks:
  - ADF schema validity
  - locked-node fingerprint unchanged
  - no out-of-scope mutation
  - route-policy compliance
- Structural additions:
  - Post-mutation ADF schema validation (required for any run containing insert or remove ops).
  - Operation manifest cross-check: each changed path must correspond to a declared operation with matching kind.
  - Intentional structural changes (declared in operation manifest) are distinguished from accidental mutations.

### `publish`

- Inputs: `candidate_page_adf`, `page_version`.
- Outputs: `publish_result`, `new_version?`, `diagnostics`.
- Conflict policy: one scoped rebase retry; then return `ERR_CONFLICT_RETRY_EXHAUSTED`.

## Orchestrator Contract

- Halt on first hard error.
- Persist replay artifacts per state: `state_input.json`, `state_output.json`, `diagnostics.json`.
- Emit final summary: `success`, `applied_paths`, `blocked_paths`, `error_codes`, `token_metrics`, `empty_page_detected`, `bootstrap_applied`.
- Structural additions: summary includes `insert_count`, `remove_count`, `replace_count`, `schema_validation_result`, and `operation_manifest[]` (list of declared operations with kinds and paths).

## Minimal Envelope Example

```json
{
  "request_id": "req_2026_03_001",
  "page_id": "18841604",
  "state": "adf_table_edit",
  "timestamp": "2026-03-06T10:00:00Z",
  "allowed_scope_paths": [
    "/content/0/table/2/tableRow/1/tableCell/0"
  ],
  "edit_intent": "Update one table cell text",
  "allowed_ops": [
    "cell_text_update"
  ]
}
```
