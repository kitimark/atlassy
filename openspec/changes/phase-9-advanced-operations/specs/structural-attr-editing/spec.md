## ADDED Requirements

### Requirement: UpdateAttrs modifies node attributes without touching content
The system SHALL support `Operation::UpdateAttrs { target_path, attrs }` which merges the provided attrs into the target node's existing `attrs` object. The node's `content` MUST NOT be modified.

#### Scenario: Update panel panelType
- **WHEN** `Operation::UpdateAttrs { target_path: "/content/2", attrs: {"panelType": "warning"} }` is applied to a panel node
- **THEN** the panel's `attrs.panelType` MUST be `"warning"`
- **AND** the panel's `content` MUST be unchanged

#### Scenario: Update media alt text
- **WHEN** `Operation::UpdateAttrs` sets `attrs: {"alt": "Updated description"}` on a mediaSingle node
- **THEN** `attrs.alt` MUST be `"Updated description"`
- **AND** all other attrs MUST be unchanged (merge, not replace)

#### Scenario: Target path does not resolve
- **WHEN** `Operation::UpdateAttrs` targets a non-existent path
- **THEN** it MUST fail with `ERR_ATTR_UPDATE_BLOCKED`

#### Scenario: Target node has no attrs field
- **WHEN** `Operation::UpdateAttrs` targets a node without an existing `attrs` object
- **THEN** it MUST create the `attrs` object with the provided values

### Requirement: BlockOp::UpdateAttrs translates to Operation::UpdateAttrs
The `BlockOp::UpdateAttrs { target_path, attrs }` SHALL translate 1:1 to `Operation::UpdateAttrs` with scope validation.

#### Scenario: UpdateAttrs on attr-editable node
- **WHEN** `BlockOp::UpdateAttrs` targets a panel
- **THEN** `adf_block_ops` MUST produce `Operation::UpdateAttrs` after validating scope

#### Scenario: UpdateAttrs on non-attr-editable node
- **WHEN** `BlockOp::UpdateAttrs` targets a paragraph (not attr-editable)
- **THEN** `adf_block_ops` MUST fail with `ERR_ATTR_UPDATE_BLOCKED`

#### Scenario: UpdateAttrs outside scope
- **WHEN** `BlockOp::UpdateAttrs` targets a path outside `allowed_scope_paths`
- **THEN** it MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`

### Requirement: Only allowed attrs can be modified per node type
Verify MUST check that the attr keys being modified are in the allowed set for the target node type. Unknown or dangerous keys MUST be rejected.

#### Scenario: Panel allows panelType attr
- **WHEN** verify checks `UpdateAttrs` on a panel with `attrs: {"panelType": "note"}`
- **THEN** legality check MUST pass

#### Scenario: Panel rejects unknown attr key
- **WHEN** verify checks `UpdateAttrs` on a panel with `attrs: {"dangerousKey": true}`
- **THEN** verify MUST fail with `ERR_ATTR_SCHEMA_VIOLATION`
