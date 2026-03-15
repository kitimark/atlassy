## MODIFIED Requirements

### Requirement: Deterministic v1 state execution order
The pipeline orchestrator SHALL execute states in this exact order: `fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> adf_block_ops -> merge_candidates -> patch -> verify -> publish`. The orchestrator MUST wire `adf_block_ops` output to `merge_candidates` input. The `patch` state MUST receive operations from `merge_candidates` output and no longer receive `md_assist_edit` or `adf_table_edit` outputs directly.

#### Scenario: All states succeed in order
- **WHEN** a run starts with valid input and no state returns a hard error
- **THEN** the orchestrator executes each state exactly once in the defined order and marks the run successful after `publish`

#### Scenario: AdfBlockOps output flows to MergeCandidates
- **WHEN** the orchestrator completes `adf_block_ops`
- **THEN** its output operations MUST be passed to `merge_candidates` as an input parameter

#### Scenario: Patch receives operations from merge only
- **WHEN** the orchestrator calls the patch state
- **THEN** it MUST pass `FetchOutput` and `MergeCandidatesOutput` (containing `Vec<Operation>`)
- **AND** it MUST NOT pass `MdAssistEditOutput` or `AdfTableEditOutput`

#### Scenario: Patch output is propagated to verify and publish
- **WHEN** `patch` produces updated `candidate_page_adf`
- **THEN** `verify` evaluates the updated candidate payload
- **AND** `publish` receives the same verified payload
