## Purpose

Define route-gated prose extraction behavior and deterministic markdown-to-ADF mapping for Phase 2 prose assist flows.

## Requirements

### Requirement: Extract prose from editable-prose route only
The `extract_prose` state SHALL convert only nodes classified as `editable_prose` whose paths are within `allowed_scope_paths` into `markdown_blocks` and MUST NOT convert `table_adf` or `locked_structural` nodes. Nodes classified as `editable_prose` whose paths fall outside `allowed_scope_paths` SHALL be silently skipped.

#### Scenario: Mixed-route scope extraction
- **WHEN** `node_manifest` contains `editable_prose`, `table_adf`, and `locked_structural` nodes
- **THEN** `markdown_blocks` contains entries only for `editable_prose` nodes within `allowed_scope_paths`
- **AND** no markdown block is emitted for table or locked-structural nodes

#### Scenario: Out-of-scope editable prose nodes are skipped
- **WHEN** `node_manifest` contains `editable_prose` nodes at paths both inside and outside `allowed_scope_paths`
- **THEN** `markdown_blocks` contains entries only for in-scope `editable_prose` nodes
- **AND** out-of-scope `editable_prose` nodes do not cause an error
- **AND** `md_to_adf_map` contains no entries for out-of-scope paths

#### Scenario: Full-page scope processes all editable prose nodes
- **WHEN** `allowed_scope_paths` contains `"/"` (full-page scope)
- **THEN** all `editable_prose` nodes are processed regardless of their path
- **AND** behavior is identical to the pre-change full-page pipeline path

### Requirement: Stable markdown-to-ADF mapping
The `extract_prose` state SHALL emit `md_to_adf_map` entries that bind each markdown block to exactly one canonical JSON Pointer path and MUST preserve deterministic mapping identity within a run.

#### Scenario: Deterministic mapping output
- **WHEN** `extract_prose` runs on the same scoped ADF and node classification inputs
- **THEN** the emitted `md_to_adf_map` set is identical across repeated executions in that run context

### Requirement: Mapping integrity between blocks and paths
Every emitted markdown block MUST have a corresponding mapping entry, and every mapping entry path MUST resolve to an in-scope `editable_prose` node.

#### Scenario: Block-map integrity validation
- **WHEN** `extract_prose` produces `markdown_blocks` and `md_to_adf_map`
- **THEN** each `md_block_id` appears exactly once in the map
- **AND** each mapped `adf_path` is inside `allowed_scope_paths`
