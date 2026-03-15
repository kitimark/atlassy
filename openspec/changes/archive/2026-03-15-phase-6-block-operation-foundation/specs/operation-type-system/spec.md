## MODIFIED Requirements

### Requirement: Unified Operation enum replaces three patch types
The system SHALL provide a single `Operation` enum in `atlassy-contracts` that replaces `PatchCandidate`, `PatchOperation`, and `PatchOp`. The enum MUST have three variants: `Operation::Replace { path: String, value: Value }`, `Operation::Insert { parent_path: String, index: usize, block: Value }`, and `Operation::Remove { target_path: String }`.

#### Scenario: Operation::Replace carries path and value
- **WHEN** a replace operation is constructed
- **THEN** `Operation::Replace` SHALL contain a `path` field (JSON pointer) and a `value` field (serde_json::Value)

#### Scenario: Operation::Insert carries parent_path, index, and block
- **WHEN** an insert operation is constructed
- **THEN** `Operation::Insert` SHALL contain `parent_path` (JSON pointer to parent array), `index` (usize position), and `block` (complete ADF node as Value)

#### Scenario: Operation::Remove carries target_path
- **WHEN** a remove operation is constructed
- **THEN** `Operation::Remove` SHALL contain `target_path` (JSON pointer to the block to remove)

#### Scenario: Operation enum is the sole operation type in patch output
- **WHEN** the patch stage produces output
- **THEN** `PatchOutput.patch_ops` MUST be `Vec<Operation>` containing Replace, Insert, and/or Remove variants

### Requirement: BlockOp is an enum with per-variant data
The `BlockOp` type MUST be an enum with `Insert { parent_path: String, index: usize, block: Value }` and `Remove { target_path: String }` variants. The `BlockOpKind` enum MUST be removed.

#### Scenario: BlockOp::Insert carries required insert data
- **WHEN** a caller constructs `BlockOp::Insert`
- **THEN** it MUST provide `parent_path`, `index`, and `block`
- **AND** the compiler MUST enforce all fields are present

#### Scenario: BlockOp::Remove carries target path only
- **WHEN** a caller constructs `BlockOp::Remove`
- **THEN** it MUST provide only `target_path`
- **AND** no `value` or `index` field exists

#### Scenario: BlockOpKind is removed
- **WHEN** any source file references `BlockOpKind`
- **THEN** the build MUST fail with an unresolved type error
