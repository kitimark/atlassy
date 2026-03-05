# Structural Block Editing Support

## Status

Incubating (deferred from v1)

## Plain Problem Points

- v1 locks structural Confluence-native blocks to protect fidelity.
- Teams still need updates in media, macros/extensions, and layout blocks.
- Manual edits for these blocks can slow delivery and increase handoff cost.

## Proposed Direction

Add staged support for structural block editing in ADF-native mode:

- Phase S1: metadata-safe updates for selected media and panel attributes.
- Phase S2: constrained macro and extension parameter edits with strict schema checks.
- Phase S3: layout-aware transformations with section and column invariants.

## Why Not Now

- v1 is focused on stable prose and table-cell updates first.
- Structural nodes have wide schema variation and higher breakage risk.
- Verification and fixture coverage are not yet sufficient for safe rollout.

## Risks

- Extension parameter edits can break runtime behavior or rendering.
- Layout changes can cause broad visual regressions beyond target scope.
- Media updates can create stale references or missing assets.

## Signals To Revisit

- Frequent user demand for macro, media, or layout edits.
- Repeated manual fix-ups after otherwise successful automated updates.
- Stable verifier performance on current v1 flows with low false positives.

## Promotion Path

Move this idea to `roadmap/` when all conditions are true:

- Structural block support matrix is defined by node type and allowed operations.
- Validation rules are specified for each supported node family.
- Golden fixtures cover representative macro/media/layout variants.
- KPI guardrails show no regression in fidelity pass rate or publish success.
