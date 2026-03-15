## MODIFIED Requirements

### Requirement: Operation enum supports attribute updates
The `Operation` enum MUST include an `UpdateAttrs { target_path: String, attrs: Value }` variant for modifying node attributes.

#### Scenario: UpdateAttrs serialization
- **WHEN** `Operation::UpdateAttrs` is serialized
- **THEN** it MUST produce `{"op": "update_attrs", "target_path": "...", "attrs": {...}}`

### Requirement: BlockOp enum supports table topology and attribute operations
The `BlockOp` enum MUST include: `InsertRow { table_path, index, cells: Vec<String> }`, `RemoveRow { table_path, index }`, `InsertColumn { table_path, index }`, `RemoveColumn { table_path, index }`, `UpdateAttrs { target_path, attrs: Value }`.

#### Scenario: All new BlockOp variants serialize and deserialize
- **WHEN** any new `BlockOp` variant is serialized to JSON and deserialized
- **THEN** the round-trip MUST produce an identical value
