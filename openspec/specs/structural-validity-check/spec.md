## Purpose

Define post-mutation structural validity requirements for insert/remove operation batches.

## Requirements

### Requirement: Post-mutation ADF passes structural validity checks
After applying insert or remove operations, the resulting ADF document MUST pass structural validity checks before publish. The check function MUST live in `atlassy-adf` as domain logic.

#### Scenario: Valid ADF after insert
- **WHEN** an insert operation produces ADF with valid doc.content containing known block types
- **THEN** structural validity check MUST pass

#### Scenario: Empty doc.content after remove
- **WHEN** remove operations result in `doc.content` being an empty array
- **THEN** structural validity check MUST fail with `ERR_POST_MUTATION_SCHEMA_INVALID`

#### Scenario: Block with missing type field
- **WHEN** the post-mutation ADF contains a block node without a `type` field
- **THEN** structural validity check MUST fail with `ERR_POST_MUTATION_SCHEMA_INVALID`

#### Scenario: Heading missing attrs.level
- **WHEN** the post-mutation ADF contains a heading node without `attrs.level` or with level outside 1-6
- **THEN** structural validity check MUST fail with `ERR_POST_MUTATION_SCHEMA_INVALID`

#### Scenario: Unknown block type in doc.content
- **WHEN** the post-mutation ADF contains a block with an unrecognized type in `doc.content`
- **THEN** structural validity check MUST pass (unknown types are allowed - they may be Confluence extensions)

### Requirement: Structural validity check is only required for runs with structural operations
The check SHALL only execute when the operation set contains at least one Insert or Remove operation. Replace-only runs MUST skip this check (backward compatible).

#### Scenario: Replace-only run skips structural validity
- **WHEN** all operations in the batch are `Operation::Replace`
- **THEN** `check_structural_validity` MUST NOT be called

#### Scenario: Run with insert triggers structural validity
- **WHEN** the operation batch contains at least one `Operation::Insert`
- **THEN** `check_structural_validity` MUST be called on the candidate ADF
