## Purpose

Define merge-stage behavior for constructing and validating a unified operation list across prose, table, and block routes.

## Requirements

### Requirement: Merge collects operations from all sources
The merge state MUST build a unified `Vec<Operation>` from three sources: prose change candidates (as `Operation::Replace`), table change candidates (as `Operation::Replace`), and block operations (as `Operation::Insert`/`Operation::Remove` from `AdfBlockOps` output).

#### Scenario: Prose and table candidates become Replace operations
- **WHEN** merge receives prose and table change candidates
- **THEN** it MUST construct `Operation::Replace` for each candidate using the candidate's path and value
- **AND** the resulting operations MUST be included in the output

#### Scenario: Block operations are included in merged output
- **WHEN** merge receives block operations from AdfBlockOps
- **THEN** those `Operation::Insert` and `Operation::Remove` instances MUST be included in the merged output alongside Replace operations

#### Scenario: Empty block operations
- **WHEN** AdfBlockOps produces an empty operation list (no block_ops in request)
- **THEN** merge MUST produce only Replace operations from prose/table candidates (backward compatible)

### Requirement: Cross-route conflict detection operates on operations
The merge state MUST detect conflicts between operations from different sources by extracting paths and checking for collisions or overlaps.

#### Scenario: Block operation path overlaps prose path
- **WHEN** a block operation targets a path that overlaps with a prose Replace path
- **THEN** merge MUST reject the batch with `ERR_ROUTE_VIOLATION`

#### Scenario: Block operation path overlaps locked structural boundary
- **WHEN** a block operation targets a path that overlaps with a locked structural node
- **THEN** merge MUST reject the batch with `ERR_ROUTE_VIOLATION`

#### Scenario: No conflicts between independent operations
- **WHEN** block operations, prose operations, and table operations target non-overlapping paths
- **THEN** merge MUST produce the combined operation list without error

### Requirement: Merge output is Vec<Operation>
The `MergeCandidatesOutput` MUST contain `operations: Vec<Operation>` instead of `changed_paths: Vec<String>`.

#### Scenario: Output contains all operation types
- **WHEN** a run has prose edits, table edits, and block operations
- **THEN** `MergeCandidatesOutput.operations` MUST contain Replace, Insert, and Remove operations
