## Purpose

Define deterministic runtime backend selection and live Confluence error mapping behavior for PoC execution.

## Requirements

### Requirement: Runtime backend selection is explicit and deterministic
The runtime MUST support backend selection between `stub` and `live` modes using explicit operator configuration, and `live` backend initialization MUST never terminate execution through process panic.

#### Scenario: Stub backend execution
- **WHEN** runtime backend is configured as `stub`
- **THEN** execution uses deterministic stub Confluence behavior and preserves existing fixture reproducibility

#### Scenario: Live backend execution
- **WHEN** runtime backend is configured as `live` with valid runtime configuration
- **THEN** execution uses live Confluence fetch and publish paths with validated runtime configuration
- **AND** startup remains within deterministic error-handling flow

#### Scenario: Live backend initialization failure is deterministic
- **WHEN** runtime backend is configured as `live` and backend initialization fails
- **THEN** execution returns a deterministic mapped runtime error
- **AND** operator-facing output is emitted without panic backtrace termination

### Requirement: Live publish request conforms to Confluence update contract
Live publish MUST construct a Confluence-compatible page update payload for ADF updates, including required version metadata and accepted `atlas_doc_format` body shape.

#### Scenario: Publish request includes required version metadata
- **WHEN** publish is invoked for page version `N`
- **THEN** the request includes `version.number = N + 1`
- **AND** includes required page update metadata fields used by Confluence update endpoints

#### Scenario: Publish request encodes atlas_doc_format body correctly
- **WHEN** a verified `candidate_page_adf` is available
- **THEN** the request includes `body.atlas_doc_format` with representation `atlas_doc_format`
- **AND** serializes value in the Confluence-accepted payload format for ADF updates

#### Scenario: Contract mismatch is deterministically classified
- **WHEN** Confluence rejects publish with payload-contract `400` response
- **THEN** execution records a deterministic runtime backend error code
- **AND** failure state is reported as `publish` for operator triage

### Requirement: Live runtime errors map to deterministic taxonomy
Live backend failures MUST map to deterministic error codes compatible with existing verifier, report, and runbook workflows, including failures that occur before `fetch` begins.

#### Scenario: Live publish conflict follows mapped retry behavior
- **WHEN** live publish returns a version conflict
- **THEN** execution applies at most one scoped retry and emits deterministic retry diagnostics on failure

#### Scenario: Live hard error is classifiable
- **WHEN** live fetch or publish returns a hard failure
- **THEN** the failure is mapped to a deterministic error code used by readiness and triage outputs

#### Scenario: Live startup hard error is classifiable
- **WHEN** live runtime startup fails before state execution
- **THEN** mapped startup failures use deterministic runtime backend taxonomy
- **AND** unmapped startup failures are surfaced as deterministic unmapped runtime hard errors

### Requirement: Live create request conforms to Confluence page creation contract
Live page creation MUST construct a Confluence-compatible page creation payload including required space key, ancestor metadata, and empty ADF body in the accepted `atlas_doc_format` representation.

#### Scenario: Create request includes space and ancestor metadata
- **WHEN** `create_page` is invoked in live mode with a valid space key and parent page ID
- **THEN** the request includes `space.key`, `ancestors` with the parent page ID, and `type` set to `page`

#### Scenario: Create request encodes empty ADF body correctly
- **WHEN** `create_page` constructs a creation payload
- **THEN** the request includes `body.atlas_doc_format` with representation `atlas_doc_format`
- **AND** the ADF value is serialized in the Confluence-accepted payload format

#### Scenario: Create request does not include version metadata
- **WHEN** `create_page` constructs a creation payload
- **THEN** the request does not include a `version` field
- **AND** Confluence auto-assigns version 1

### Requirement: Live create errors map to deterministic taxonomy
Live create failures MUST map to deterministic error codes compatible with existing runtime error taxonomy.

#### Scenario: Parent not found returns deterministic error
- **WHEN** live create targets a parent page ID that does not exist
- **THEN** the error is mapped as a not-found error with the parent page ID

#### Scenario: Permission denied returns deterministic error
- **WHEN** live create is rejected with a 403 response
- **THEN** the error is mapped as a transport error with HTTP status and response body

#### Scenario: Duplicate title returns deterministic error
- **WHEN** live create is rejected because a page with the same title already exists
- **THEN** the error is mapped as a transport error with HTTP status and response body identifying the collision

### Requirement: Runtime mode is included in artifacts
Run, batch, and readiness artifacts MUST include the selected runtime backend mode.

#### Scenario: Runtime mode is auditable in outputs
- **WHEN** artifacts are generated for a run sequence
- **THEN** each decision-grade output records whether execution used `stub` or `live`
