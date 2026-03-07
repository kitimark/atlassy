# Route Classification Drift

## Status

Promoted to roadmap (Phase 5 hardening)

## Roadmap Linkage (current source of truth)

- `roadmap/03-phased-roadmap.md` (Phase 5 scope: fix `rule` node route classification drift)
- `roadmap/06-decisions-and-defaults.md` (Default Route Matrix: `rule` removed from `editable_prose`)

## Plain Problem Points

- The Default Route Matrix in `roadmap/06-decisions-and-defaults.md` lists `rule` (horizontal rule) as `editable_prose`.
- The code in `route_for_node()` (`crates/atlassy-pipeline/src/lib.rs`, lines 1354-1370) classifies `rule` as `locked_structural` via the catch-all `_ =>` arm. The prose whitelist contains only: `paragraph`, `heading`, `bulletList`, `orderedList`, `listItem`, `blockquote`, `codeBlock`.
- Spec and code disagree on the classification of `rule` nodes.

## Proposed Direction

Either:

1. **Fix the code**: add `"rule"` to the match arm in `route_for_node()` so it classifies as `editable_prose`, matching the spec. This is a one-line change.
2. **Fix the spec**: remove `rule` from the Default Route Matrix in `06-decisions-and-defaults.md` if the intent is to keep horizontal rules locked.

The correct choice depends on whether horizontal rules should be editable in v1. They are simple structural elements with no complex attributes, so option 1 (making them editable) is likely the intended behavior.

## Historical Why Not Now (pre-promotion)

- Impact is very low. Horizontal rules are rarely edit targets in typical Confluence workflows.
- No KPI experiment or test scenario currently targets `rule` nodes.
- The discrepancy was discovered during KPI revalidation exploration, not from a user-facing failure.

## Risks

- If left unfixed, a user attempting to edit content near a horizontal rule may see unexpected locked-node behavior.
- Spec/code drift, even in low-impact areas, erodes confidence in the documentation as a source of truth.

## Signals To Revisit

- Any change to the route classification logic or default route matrix.
- Content classification issues surface during KPI experiments.
- A user or test scenario explicitly targets `rule` nodes.

## Promotion Outcome

This idea has been promoted into v1 Phase 5 hardening scope.

- Resolution: align spec to code. `rule` removed from `editable_prose` in the Default Route Matrix; stays `locked_structural` via the catch-all arm in `route_for_node()`.
- Rationale: horizontal rules are rarely edit targets, and keeping them locked is the safer default. Revisit post-PoC if editing use cases emerge.
- No code change needed — spec updated to match existing code behavior.

## Code References

- `route_for_node()`: `crates/atlassy-pipeline/src/lib.rs`, lines 1354-1370.
- Default Route Matrix: `roadmap/06-decisions-and-defaults.md`, lines 97-99.
