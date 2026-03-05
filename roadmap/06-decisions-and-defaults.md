# Decisions and Defaults

## Decision Log (v1)

### D-001: Canonical representation

- Decision: ADF is canonical across fetch, patch, verify, and publish.
- Rationale: preserves Confluence-native fidelity and full feature coverage.

### D-002: Markdown usage policy

- Decision: Markdown is a transient assist format for `editable_prose` only.
- Rationale: improves human readability while avoiding structural feature drift.

### D-003: Table editing scope

- Decision: tables are editable in v1 via ADF-native path, cell text only.
- Rationale: balances usability gains with low structural risk.
- Deferred: row/column operations and full restructuring are tracked in `ideas/2026-03-advanced-table-editing-modes.md`.

### D-004: Locked structural policy

- Decision: unsupported structural blocks remain locked in v1.
- Default set: media, macros/extensions, layouts, mentions, status, panels, embeds, and non-whitelisted nodes.
- Rationale: avoid lossy or unsafe edits until dedicated support exists.

### D-005: Patch strategy

- Decision: path-targeted ADF patch operations only.
- Rationale: minimize mutation scope and reduce conflict amplification.

### D-006: Verification gates

- Decision: publish is blocked unless all checks pass.
- Required checks: ADF schema validity, locked-node fingerprint preservation, no out-of-scope mutation, route-policy compliance.

### D-007: Conflict policy

- Decision: one scoped rebase retry on version conflict, then fail fast.
- Rationale: limits retry token waste and prevents hidden repeated mutation.

## Default Route Matrix

- `editable_prose`: paragraph, heading, bulletList, orderedList, listItem, blockquote, simple codeBlock, rule.
- `table_adf`: table node family, cell text edits only.
- `locked_structural`: all other nodes by default.

## Change Control

- Any expansion of `table_adf` beyond cell text requires a new decision entry and updated verifier rules.
- Any expansion of Markdown conversion scope requires explicit fidelity test evidence.
