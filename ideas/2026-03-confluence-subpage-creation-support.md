# Confluence Sub-Page Creation Support

## Status

Promoted to roadmap (v1 release enablement)

## Roadmap Linkage (current source of truth)

- `roadmap/03-phased-roadmap.md`
- `roadmap/06-decisions-and-defaults.md`
- `roadmap/07-execution-readiness.md`
- `roadmap/08-poc-scope.md`
- `roadmap/11-live-runtime-execution-plan.md`
- `roadmap/12-page-lifecycle-expansion-plan.md`

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

## Historical Why Not Now (pre-promotion)

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

## Promotion Outcome

This idea has been promoted into v1 release-enablement scope.

- `create-subpage` is a command-first capability in v1 planning.
- New sub-pages must be created truly blank by default.
- Standard edit runs must not create pages implicitly.
- Release readiness requires lifecycle matrix evidence and deterministic failure handling.
