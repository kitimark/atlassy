## Purpose

Define a unified operation type system for patch operations and Phase 6 block-op preparation.

## Requirements

### Requirement: Unified Operation enum replaces three patch types
The system SHALL provide a single `Operation` enum in `atlassy-contracts` that replaces `PatchCandidate`, `PatchOperation`, and `PatchOp`. The enum MUST have four variants: `Operation::Replace { path: String, value: Value }`, `Operation::Insert { parent_path: String, index: usize, block: Value }`, `Operation::Remove { target_path: String }`, and `Operation::UpdateAttrs { target_path: String, attrs: Value }`.

#### Scenario: Operation::Replace carries path and value
- **WHEN** a replace operation is constructed
- **THEN** `Operation::Replace` SHALL contain a `path` field (JSON pointer) and a `value` field (serde_json::Value)

#### Scenario: Operation::Insert carries parent_path, index, and block
- **WHEN** an insert operation is constructed
- **THEN** `Operation::Insert` SHALL contain `parent_path` (JSON pointer to parent array), `index` (usize position), and `block` (complete ADF node as Value)

#### Scenario: Operation::Remove carries target_path
- **WHEN** a remove operation is constructed
- **THEN** `Operation::Remove` SHALL contain `target_path` (JSON pointer to the block to remove)

#### Scenario: Operation::UpdateAttrs carries target_path and attrs
- **WHEN** an update-attrs operation is constructed
- **THEN** `Operation::UpdateAttrs` SHALL contain `target_path` (JSON pointer to the node) and `attrs` (serde_json::Value)

#### Scenario: Operation enum is the sole operation type in patch output
- **WHEN** the patch stage produces output
- **THEN** `PatchOutput.patch_ops` MUST be `Vec<Operation>` containing Replace, Insert, Remove, and/or UpdateAttrs variants

#### Scenario: UpdateAttrs serialization
- **WHEN** `Operation::UpdateAttrs` is serialized
- **THEN** it MUST produce `{"op": "update_attrs", "target_path": "...", "attrs": {...}}`

### Requirement: Operation serializes identically to previous PatchOp
The `Operation` enum MUST use `#[serde(tag = "op", rename_all = "snake_case")]` so that `Operation::Replace { path, value }` serializes as `{"op": "replace", "path": "...", "value": ...}`, byte-identical to the previous `PatchOp { op: "replace", path, value }`.

#### Scenario: Serialized artifact format preserved
- **WHEN** an `Operation::Replace` is serialized to JSON
- **THEN** the output MUST be structurally identical to a `PatchOp` with `op: "replace"` - the `op` field appears as a string `"replace"`, followed by `path` and `value`

#### Scenario: Existing artifacts deserialize into Operation
- **WHEN** a previously serialized `PatchOp` JSON value with `op: "replace"` is deserialized
- **THEN** it MUST successfully deserialize as `Operation::Replace`

### Requirement: PatchCandidate, PatchOperation, and PatchOp are removed
The types `PatchCandidate` (from `atlassy-adf`), `PatchOperation` (from `atlassy-adf`), and `PatchOp` (from `atlassy-contracts`) MUST be removed from the codebase. No code SHALL reference these types after Phase 5.5.

#### Scenario: Removed types do not compile
- **WHEN** any source file references `PatchCandidate`, `PatchOperation`, or `PatchOp`
- **THEN** the build MUST fail with an unresolved type error

### Requirement: BlockOp enum supports structural composition intents
The `BlockOp` enum MUST include higher-level variants that expand into multiple `Operation` commands: `InsertSection`, `RemoveSection`, `InsertTable`, `InsertList`, `InsertRow { table_path, index, cells: Vec<String> }`, `RemoveRow { table_path, index }`, `InsertColumn { table_path, index }`, `RemoveColumn { table_path, index }`, and `UpdateAttrs { target_path, attrs: Value }`.

#### Scenario: BlockOp::Insert carries required insert data
- **WHEN** a caller constructs `BlockOp::Insert`
- **THEN** it MUST provide `parent_path`, `index`, and `block`
- **AND** the compiler MUST enforce all fields are present

#### Scenario: BlockOp::Remove carries target path only
- **WHEN** a caller constructs `BlockOp::Remove`
- **THEN** it MUST provide only `target_path`
- **AND** no `value` or `index` field exists

#### Scenario: BlockOp::InsertSection carries section data
- **WHEN** a caller constructs `BlockOp::InsertSection`
- **THEN** it MUST provide `parent_path: String`, `index: usize`, `heading_level: u8`, `heading_text: String`, `body_blocks: Vec<Value>`

#### Scenario: BlockOp::RemoveSection carries heading path
- **WHEN** a caller constructs `BlockOp::RemoveSection`
- **THEN** it MUST provide `heading_path: String`

#### Scenario: BlockOp::InsertTable carries table dimensions
- **WHEN** a caller constructs `BlockOp::InsertTable`
- **THEN** it MUST provide `parent_path: String`, `index: usize`, `rows: usize`, `cols: usize`, `header_row: bool`

#### Scenario: BlockOp::InsertList carries list data
- **WHEN** a caller constructs `BlockOp::InsertList`
- **THEN** it MUST provide `parent_path: String`, `index: usize`, `ordered: bool`, `items: Vec<String>`

#### Scenario: BlockOp::InsertRow carries row insertion data
- **WHEN** a caller constructs `BlockOp::InsertRow`
- **THEN** it MUST provide `table_path: String`, `index: usize`, and `cells: Vec<String>`

#### Scenario: BlockOp::InsertColumn carries column insertion data
- **WHEN** a caller constructs `BlockOp::InsertColumn`
- **THEN** it MUST provide `table_path: String` and `index: usize`

#### Scenario: BlockOp::UpdateAttrs carries attr update data
- **WHEN** a caller constructs `BlockOp::UpdateAttrs`
- **THEN** it MUST provide `target_path: String` and `attrs: Value`

#### Scenario: All BlockOp variants serialize and deserialize
- **WHEN** any `BlockOp` variant is serialized to JSON and deserialized
- **THEN** the round-trip MUST produce an identical value

#### Scenario: BlockOpKind is removed
- **WHEN** any source file references `BlockOpKind`
- **THEN** the build MUST fail with an unresolved type error
