## MODIFIED Requirements

### Requirement: BlockOp enum supports structural composition intents
The `BlockOp` enum MUST include higher-level variants that expand into multiple `Operation` commands: `InsertSection`, `RemoveSection`, `InsertTable`, `InsertList`.

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

#### Scenario: All BlockOp variants serialize and deserialize
- **WHEN** any `BlockOp` variant is serialized to JSON and deserialized
- **THEN** the round-trip MUST produce an identical value
