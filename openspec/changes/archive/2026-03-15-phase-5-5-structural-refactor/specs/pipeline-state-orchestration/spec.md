## MODIFIED Requirements

### Requirement: Deterministic v1 state execution order
The pipeline orchestrator SHALL execute states in this exact order: `fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> adf_block_ops -> merge_candidates -> patch -> verify -> publish`, and MUST ensure the `patch` state output candidate payload is the payload evaluated by `verify` and attempted by `publish`.

#### Scenario: All states succeed in order
- **WHEN** a run starts with valid input and no state returns a hard error
- **THEN** the orchestrator executes each state exactly once in the defined order (including `adf_block_ops`) and marks the run successful after `publish`

#### Scenario: State order mismatch is prevented
- **WHEN** a state transition is attempted out of the defined sequence
- **THEN** the orchestrator MUST fail the run with a deterministic transition error and MUST NOT execute downstream states

#### Scenario: Patch output is propagated to verify and publish
- **WHEN** `patch` produces updated `candidate_page_adf`
- **THEN** `verify` evaluates the updated candidate payload
- **AND** `publish` receives the same verified payload

#### Scenario: AdfBlockOps executes between AdfTableEdit and MergeCandidates
- **WHEN** the orchestrator reaches the `adf_block_ops` step
- **THEN** it MUST execute after `adf_table_edit` and before `merge_candidates`
- **AND** in Phase 5.5 it MUST produce no changes to the pipeline data flow
