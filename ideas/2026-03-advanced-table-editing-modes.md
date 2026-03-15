# Advanced Table Editing Modes

## Status

Promoted to roadmap Phase 9 (Advanced Operations). See `roadmap/03-phased-roadmap.md`.

## Plain Problem Points

- v1 supports table cell text edits only, which avoids high-risk structural mutations.
- Some workflows require row or column operations that cannot be handled by cell-text-only updates.
- Full table restructuring introduces higher fidelity and conflict risk.

## Proposed Direction

Stage table capability growth behind explicit modes:

- Mode 1 (v1 default): cell text updates only.
- Mode 2 (future): row and column add/remove operations with strict bounds checks.
- Mode 3 (future): full restructuring (merge/split cells, table attrs/layout updates).

## Why Not Now

- Mode 2 and Mode 3 require stronger table-shape diffing and verification.
- Conflict handling for structural table edits is more complex than text edits.
- Additional fixture coverage is required across table variants and nested content.

## Risks

- Row and column operations can break references and section assumptions.
- Merge/split operations can corrupt cell mapping if path identity is unstable.
- Larger table patches increase retry cost and conflict probability.

## Signals To Revisit

- Frequent user demand for row and column edits.
- Repeated manual work after automated cell text updates.
- Verified stability of table path mapping and shape checks in v1 production usage.

## Promotion Path

Move this idea to `roadmap/` when all conditions are true:

- v1 table-cell path targeting is stable across representative datasets.
- Table-shape diff and validation rules are specified and reviewed.
- Conflict policy for structural table edits is defined and tested.
- KPI guardrails show no regression risk for fidelity and publish success.
