## ADDED Requirements

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
