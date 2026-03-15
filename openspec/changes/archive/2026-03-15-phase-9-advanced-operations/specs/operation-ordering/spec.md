## MODIFIED Requirements

### Requirement: Ordering handles UpdateAttrs operations
The `sort_operations` function MUST handle `Operation::UpdateAttrs` operations. UpdateAttrs operations MUST be treated as leaf operations (like Replace) — they modify attrs in place without shifting indices.

#### Scenario: UpdateAttrs sorted with Replace
- **WHEN** `sort_operations` receives Replace and UpdateAttrs operations
- **THEN** both MUST appear before structural ops (Insert/Remove)

#### Scenario: UpdateAttrs does not conflict with Insert/Remove
- **WHEN** UpdateAttrs targets a node and Insert/Remove targets a sibling
- **THEN** no conflict MUST be detected (UpdateAttrs doesn't shift indices)

### Requirement: Table-internal operations follow existing ordering rules
Row and column operations that compose into `Operation::Insert` / `Operation::Remove` within tables MUST follow the existing reverse-document-order sorting. Deeper paths (inside tables) process before shallower paths (doc.content level).

#### Scenario: Table-internal ops before doc-level ops
- **WHEN** operations include both doc-level Insert and table-internal Insert (column add)
- **THEN** table-internal ops (deeper paths) MUST be sorted before doc-level ops
