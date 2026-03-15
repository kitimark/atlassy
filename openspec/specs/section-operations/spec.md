## Purpose

Define section-level structural insert/remove operations that expand into ordered patch operations.

## Requirements

### Requirement: InsertSection creates a heading and body blocks as a unit
The system SHALL support `BlockOp::InsertSection` which expands into multiple `Operation::Insert` commands - one for the heading and one for each body block - all targeting consecutive positions in the parent array.

#### Scenario: Insert section with heading and two paragraphs
- **WHEN** `BlockOp::InsertSection { parent_path: "/content", index: 2, heading_level: 2, heading_text: "FAQ", body_blocks: [para1, para2] }` is processed
- **THEN** `adf_block_ops` MUST produce 3 Operation::Insert commands: heading at index 2, para1 at index 3, para2 at index 4
- **AND** after reverse-order sorting and application, the result contains [heading, para1, para2] at positions 2-4

#### Scenario: Insert section with empty body
- **WHEN** `BlockOp::InsertSection` has empty `body_blocks`
- **THEN** it MUST produce 1 Operation::Insert for the heading only

#### Scenario: Insert section heading level validation
- **WHEN** `BlockOp::InsertSection` has `heading_level` outside 1-6
- **THEN** `adf_block_ops` MUST fail with an appropriate error

#### Scenario: Insert section respects scope
- **WHEN** `BlockOp::InsertSection` has `parent_path` outside `allowed_scope_paths`
- **THEN** `adf_block_ops` MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`

### Requirement: RemoveSection deletes a heading and all its body blocks
The system SHALL support `BlockOp::RemoveSection` which uses section boundary detection to find all blocks in the section and produces `Operation::Remove` commands for each, in reverse document order.

#### Scenario: Remove section with body blocks
- **GIVEN** doc.content has H2("Details") at index 2, followed by para at index 3 and para at index 4, with H2("Summary") at index 5
- **WHEN** `BlockOp::RemoveSection { heading_path: "/content/2" }` is processed
- **THEN** `adf_block_ops` MUST produce 3 Operation::Remove commands for indices 4, 3, 2 (reverse order)

#### Scenario: Remove section at end of document
- **GIVEN** doc.content has H2("Last") at index 3, followed by para at index 4 (end of content)
- **WHEN** `BlockOp::RemoveSection { heading_path: "/content/3" }` is processed
- **THEN** it MUST produce 2 Operation::Remove commands for indices 4, 3

#### Scenario: Remove section with no body
- **GIVEN** doc.content has H2("Empty") at index 1, H2("Next") at index 2
- **WHEN** `BlockOp::RemoveSection { heading_path: "/content/1" }` is processed
- **THEN** it MUST produce 1 Operation::Remove for index 1

#### Scenario: Remove section target is not a heading
- **WHEN** `BlockOp::RemoveSection { heading_path: "/content/1" }` targets a paragraph
- **THEN** `adf_block_ops` MUST fail with an appropriate error

#### Scenario: Remove section respects scope
- **WHEN** `BlockOp::RemoveSection` has `heading_path` outside `allowed_scope_paths`
- **THEN** `adf_block_ops` MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`
