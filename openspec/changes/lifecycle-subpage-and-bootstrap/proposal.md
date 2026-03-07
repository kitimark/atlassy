## Why

v1 release readiness requires page lifecycle support — sub-page creation and empty-page first-edit bootstrapping — before the final `go | iterate | stop` sign-off. These capabilities are release-gating per `roadmap/12-page-lifecycle-expansion-plan.md`, but zero implementation exists today: no CLI commands, no client trait methods, no error constants, no pipeline detection logic. This change closes that gap so end-to-end lifecycle flows can be validated in both stub and live sandbox before the v1 decision.

## What Changes

- Add `create-subpage` as a standalone CLI command that creates a truly blank child page under an explicit parent, without involving the pipeline orchestrator. Extends the `ConfluenceClient` trait with a `create_page` method, implemented for both stub and live backends.
- Add `--bootstrap-empty-page` flag to the `run` command. After fetch, detect whether the page is effectively empty. Enforce a deterministic behavior matrix: empty page without flag hard-fails (`ERR_BOOTSTRAP_REQUIRED`), empty page with flag injects a minimal prose scaffold then continues the pipeline, non-empty page with flag hard-fails (`ERR_BOOTSTRAP_INVALID_STATE`), non-empty page without flag follows the unchanged v1 flow.
- Add empty-page detection as a new function in `atlassy-adf` since no content-emptiness check exists today.
- Add Gate 7 (Lifecycle Enablement Validation) to the readiness evaluation. The roadmap defines this gate but the code currently evaluates only gates 1–6.
- Add new error constants and telemetry markers for lifecycle operations to the contracts crate.

## Capabilities

### New Capabilities

- `confluence-subpage-creation`: Defines the `create_page` client contract, blank-page creation default, parent-page targeting, space-key requirement, deterministic error paths for not-found and duplicate-title cases, and provenance in creation outputs.
- `empty-page-bootstrap-editing`: Defines empty-page detection criteria, the `--bootstrap-empty-page` flag contract, the four-path behavior matrix (empty±flag, non-empty±flag), minimal prose scaffold shape, and bootstrap telemetry markers in run summaries.

### Modified Capabilities

- `readiness-gate-checklist`: Add Gate 7 (Lifecycle Enablement Validation) requiring deterministic evidence for blank subpage creation, bootstrap-required failure, bootstrap success, and bootstrap-on-non-empty failure.
- `live-confluence-runtime-selection`: Extend the client contract to include `create_page` with live HTTP implementation (POST to content collection endpoint) and deterministic error mapping for 404/400/403 responses.

## Impact

- Affected code: `crates/atlassy-confluence` (trait extension, stub + live implementations), `crates/atlassy-adf` (empty-page detection function), `crates/atlassy-pipeline` (bootstrap detection and scaffold injection in `run_internal`, `RunRequest` field addition), `crates/atlassy-contracts` (error constants, response types, summary fields), `crates/atlassy-cli` (new `CreateSubpage` command, `--bootstrap-empty-page` flag, Gate 7 evaluation).
- Affected outputs: run summaries gain bootstrap-related markers, readiness checklist gains Gate 7, decision packets reflect lifecycle gate outcomes.
- Runtime impact: introduces a new write operation (`create_page`) and a new pre-classify detection step in the pipeline, while preserving all existing v1 safety constraints and route policies.
