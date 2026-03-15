## Context

The workspace has 5 crates with internal path dependencies declared as `{ path = "...", version = "0.1.0" }`. The `version` field was set once at creation and never updated — it reads `0.1.0` while the workspace version has moved to `0.1.2`. Cargo ignores `version` when resolving via `path`, so this field has never had any effect.

The project has no publish intent: no `license`, `description`, or other crates.io-required metadata exists on any crate.

## Goals / Non-Goals

**Goals:**
- Remove dead `version` metadata from internal path dependencies
- Eliminate version drift between workspace version and path dep declarations
- Establish a clear convention: local path deps use `path` only

**Non-Goals:**
- Centralizing internal deps via `[workspace.dependencies]` (considered, deferred — Option B from exploration)
- Adding `publish = false` to crates (separate concern)
- Changing any external dependency declarations

## Decisions

**Decision: Remove `version` rather than fix it**
- Alternative: Update all `version = "0.1.0"` to `version = "0.1.2"` to match workspace
- Rationale: Fixing creates ongoing maintenance — every version bump would need updates in 7 places. Since there's no publish intent, the field serves no purpose. Removing is permanent; fixing is recurring.

**Decision: Keep path declarations local (Option A)**
- Alternative: Move internal deps to `[workspace.dependencies]` for full consistency (Option B)
- Rationale: Option B is a valid improvement but a separate concern. This change is scoped to removing dead config. Option B can be pursued independently later.

## Risks / Trade-offs

- [If publish intent emerges] `version` will need to be re-added to path deps → Low risk; would require adding `license`, `description`, etc. anyway, so adding `version` back is trivial by comparison.
- [Zero risk to build] Cargo never used the `version` field for local path resolution → No functional change.
