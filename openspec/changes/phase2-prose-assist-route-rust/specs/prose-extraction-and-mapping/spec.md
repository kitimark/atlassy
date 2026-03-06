## ADDED Requirements

### Requirement: Extract prose from editable-prose route only
The `extract_prose` state SHALL convert only nodes classified as `editable_prose` into `markdown_blocks` and MUST NOT convert `table_adf` or `locked_structural` nodes.

#### Scenario: Mixed-route scope extraction
- **WHEN** `node_manifest` contains `editable_prose`, `table_adf`, and `locked_structural` nodes
- **THEN** `markdown_blocks` contains entries only for `editable_prose` nodes
- **AND** no markdown block is emitted for table or locked-structural nodes

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
