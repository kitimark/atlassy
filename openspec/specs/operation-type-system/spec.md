## Purpose

Define a unified operation type system for patch operations and Phase 6 block-op preparation.

## Requirements

### Requirement: Unified Operation enum replaces three patch types
The system SHALL provide a single `Operation` enum in `atlassy-contracts` that replaces `PatchCandidate`, `PatchOperation`, and `PatchOp`. In Phase 5.5, the enum MUST have exactly one variant: `Operation::Replace { path: String, value: Value }`.

#### Scenario: Operation::Replace carries path and value
- **WHEN** a replace operation is constructed
- **THEN** `Operation::Replace` SHALL contain a `path` field (JSON pointer) and a `value` field (serde_json::Value)
- **AND** no other fields or `op` string field exists

#### Scenario: Operation enum is the sole operation type in patch output
- **WHEN** the patch stage produces output
- **THEN** `PatchOutput.patch_ops` MUST be `Vec<Operation>` and MUST NOT reference `PatchOp`, `PatchCandidate`, or `PatchOperation`

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

### Requirement: BlockOp and BlockOpKind types exist for Phase 6 preparation
The system SHALL define `BlockOp { kind: BlockOpKind, path: String, value: Option<Value> }` and `BlockOpKind` enum with `Insert` and `Remove` variants in `atlassy-contracts`. These types are unused in Phase 5.5.

#### Scenario: BlockOp types compile and serialize
- **WHEN** a `BlockOp` with `kind: BlockOpKind::Insert` is constructed and serialized
- **THEN** it MUST produce valid JSON without runtime errors
