## ADDED Requirements

### Requirement: Insert operation adds a new ADF block at a specified position
The system SHALL support `Operation::Insert { parent_path, index, block }` which inserts the given ADF block node as a child of the parent array at the specified index position. The block type MUST be one of the `EDITABLE_PROSE_TYPES` (paragraph, heading, bulletList, orderedList, listItem, blockquote, codeBlock).

#### Scenario: Insert a paragraph after an existing heading
- **WHEN** an `Operation::Insert` targets parent_path `/content` with index `2` and block `{"type": "paragraph", "content": [{"type": "text", "text": "New content"}]}`
- **THEN** the block SHALL be inserted at position 2 in the `/content` array
- **AND** all subsequent siblings shift to higher indices
- **AND** the resulting ADF is valid

#### Scenario: Insert at index 0 (beginning of parent array)
- **WHEN** an `Operation::Insert` targets index `0`
- **THEN** the block SHALL be inserted at the start of the parent array

#### Scenario: Insert at index equal to parent array length (append)
- **WHEN** an `Operation::Insert` targets index equal to the parent array length
- **THEN** the block SHALL be appended at the end of the parent array

#### Scenario: Insert at out-of-bounds index
- **WHEN** an `Operation::Insert` targets index greater than the parent array length
- **THEN** the operation MUST fail with `ERR_INSERT_POSITION_INVALID`

#### Scenario: Insert non-editable-prose block type
- **WHEN** an `Operation::Insert` specifies a block with type not in `EDITABLE_PROSE_TYPES`
- **THEN** the operation MUST fail with `ERR_INSERT_POSITION_INVALID`

#### Scenario: Insert at parent path that is not an array
- **WHEN** an `Operation::Insert` targets a parent_path that does not resolve to a JSON array
- **THEN** the operation MUST fail with `ERR_INSERT_POSITION_INVALID`

### Requirement: Remove operation deletes an existing ADF block
The system SHALL support `Operation::Remove { target_path }` which removes the ADF block at the specified path. The target block type MUST be one of the `EDITABLE_PROSE_TYPES`.

#### Scenario: Remove a paragraph within scope
- **WHEN** an `Operation::Remove` targets `/content/2` which is a paragraph
- **THEN** the block at that position SHALL be removed
- **AND** all subsequent siblings shift to lower indices

#### Scenario: Remove target that does not exist
- **WHEN** an `Operation::Remove` targets a path that does not resolve
- **THEN** the operation MUST fail with `ERR_REMOVE_ANCHOR_MISSING`

#### Scenario: Remove a scope anchor heading
- **WHEN** an `Operation::Remove` targets a heading that is used as a scope selector
- **THEN** the operation MUST fail with `ERR_REMOVE_ANCHOR_MISSING`

#### Scenario: Remove non-editable-prose block type
- **WHEN** an `Operation::Remove` targets a block with type not in `EDITABLE_PROSE_TYPES`
- **THEN** the operation MUST fail with `ERR_REMOVE_ANCHOR_MISSING`

### Requirement: Insert and Remove operations respect scope boundaries
All Insert and Remove operations MUST have their paths within `allowed_scope_paths`. Insert `parent_path` MUST be within scope. Remove `target_path` MUST be within scope.

#### Scenario: Insert parent path outside scope
- **WHEN** an `Operation::Insert` has a parent_path outside `allowed_scope_paths`
- **THEN** the operation MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`

#### Scenario: Remove target path outside scope
- **WHEN** an `Operation::Remove` has a target_path outside `allowed_scope_paths`
- **THEN** the operation MUST fail with `ERR_OUT_OF_SCOPE_MUTATION`

### Requirement: Multi-operation batch produces correct results
A single pipeline run MUST support multiple operations (insert + delete + replace) in the same batch, producing correct results via reverse-order processing.

#### Scenario: Insert and replace in same run
- **WHEN** a run contains both `Operation::Insert` and `Operation::Replace` targeting the same parent
- **THEN** both operations apply correctly with replaces executing before structural ops

#### Scenario: Insert and remove in same run
- **WHEN** a run contains both `Operation::Insert` and `Operation::Remove` targeting the same parent
- **THEN** both operations apply correctly with removes and inserts in reverse index order

#### Scenario: Conflicting remove and nested operation
- **WHEN** a run contains `Operation::Remove` at path `/content/2` and another operation targeting `/content/2/content/0`
- **THEN** the batch MUST be rejected as a conflict before application
