# Confluence Sub-Page Creation Support

## Status

Incubating (deferred from v1)

## Plain Problem Points

- Live execution currently requires an existing `page_id` before Atlassy can operate.
- Teams repeatedly create child pages manually before running sandbox or pilot workflows.
- This setup overhead slows QA loops and increases operator handoff friction.

## Proposed Direction

Introduce a command-first page lifecycle capability:

- Add a dedicated `create-subpage` command for creating a child page under a specified parent page ID.
- Create truly blank pages by default (no automatic seed content).
- Return created page metadata (minimum: new page ID) for immediate follow-up edit runs.
- Keep `run` behavior unchanged in phase 1 (no implicit page creation inside edit execution).

## Why Not Now

- Current stabilization work focused on deterministic live runtime behavior and publish reliability.
- Page creation introduces new permission, hierarchy, and duplicate-title failure modes.
- Safe rollout needs explicit idempotency and duplicate-title policy decisions.

## Risks

- Duplicate titles under one parent can create ambiguous operator targeting.
- Incorrect parent placement can scatter pages across spaces and increase cleanup cost.
- Permission gaps can create partial automation where create fails but edit assumptions remain.
- If creation is silently coupled to edit flows, blast radius increases.

## Signals To Revisit

- Repeated manual child-page setup in QA and pre-production workflows.
- Frequent requests for end-to-end "create then edit" automation.
- Stable live runtime behavior across multiple clean validation cycles.

## Promotion Path

Move this idea to `roadmap/` when all conditions are true:

- `create-subpage` contract is defined for stub and live clients with deterministic error mapping.
- Duplicate-title handling policy is explicit and covered by tests.
- Provenance and audit outputs capture parent page ID, created page ID, and request context.
- QA runbook includes safe usage and cleanup guidance for created pages.
