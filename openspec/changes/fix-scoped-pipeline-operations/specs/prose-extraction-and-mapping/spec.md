## MODIFIED Requirements

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
