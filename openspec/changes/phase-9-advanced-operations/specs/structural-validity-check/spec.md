## MODIFIED Requirements

### Requirement: Structural validity checks table column consistency
After table topology operations, `check_structural_validity` MUST verify that all rows in a table have the same number of cells.

#### Scenario: Consistent column count passes
- **WHEN** all rows in every table have the same number of cells
- **THEN** structural validity MUST pass

#### Scenario: Inconsistent column count after column operation
- **WHEN** a table has rows with different cell counts
- **THEN** structural validity MUST fail with `ERR_POST_MUTATION_SCHEMA_INVALID`

#### Scenario: Tables with no rows
- **WHEN** a table has an empty content array (no rows)
- **THEN** structural validity MUST fail with `ERR_POST_MUTATION_SCHEMA_INVALID`
