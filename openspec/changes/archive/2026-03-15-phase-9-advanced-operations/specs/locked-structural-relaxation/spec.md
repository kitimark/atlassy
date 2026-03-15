## ADDED Requirements

### Requirement: Locked boundary check is op-type-aware
The `check_locked_boundary` function MUST evaluate operations based on their type and the nature of the locked node, rather than applying a blanket block.

#### Scenario: Replace on locked node remains blocked
- **WHEN** `Operation::Replace` targets a path inside a locked structural node
- **THEN** the check MUST return a route violation error (unchanged behavior)

#### Scenario: UpdateAttrs on attr-editable locked node is allowed
- **WHEN** `Operation::UpdateAttrs` targets a panel, expand, or mediaSingle node
- **THEN** the check MUST allow the operation (no error)

#### Scenario: UpdateAttrs on non-attr-editable locked node is blocked
- **WHEN** `Operation::UpdateAttrs` targets an extension or layoutSection
- **THEN** the check MUST return a route violation error

#### Scenario: Insert child inside locked container is allowed
- **WHEN** `Operation::Insert` has parent_path inside a locked container (e.g., panel content)
- **THEN** the check MUST allow the operation (child insertion OK)

#### Scenario: Remove child inside locked container is allowed
- **WHEN** `Operation::Remove` targets a block inside a locked container
- **THEN** the check MUST allow the operation (child removal OK)

#### Scenario: Remove the locked container itself is blocked
- **WHEN** `Operation::Remove` targets the locked container node directly
- **THEN** the check MUST return a route violation error

### Requirement: is_attr_editable_type controls which locked nodes accept UpdateAttrs
The type policy module MUST provide `is_attr_editable_type(node_type)` returning true for `panel`, `expand`, and `mediaSingle` only.

#### Scenario: Panel is attr-editable
- **WHEN** `is_attr_editable_type("panel")` is called
- **THEN** it MUST return `true`

#### Scenario: Extension is not attr-editable
- **WHEN** `is_attr_editable_type("extension")` is called
- **THEN** it MUST return `false`
