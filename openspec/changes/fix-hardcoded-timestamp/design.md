## Context

The `RunRequest.timestamp` field is a `String` used throughout the pipeline for run identification, ordering, and telemetry. Currently, `execute_run_command()` in `commands/run.rs:113` assigns a hardcoded `"2026-03-06T10:00:00Z"` and `default_manifest_timestamp()` in `types.rs:563` returns `"1970-01-01T00:00:00Z"`. No time/date crate exists in the workspace dependency tree. The timestamp field is a plain `String` in all three structs that carry it (`RunRequest`, `RunSummary`, `ManifestRunEntry`).

## Goals / Non-Goals

**Goals:**

- Generate real RFC 3339 UTC timestamps at CLI invocation time
- Add `chrono` as a workspace dependency available to `atlassy-cli`
- Keep test code deterministic — no changes to hardcoded timestamps in test files

**Non-Goals:**

- Changing the `timestamp` field type from `String` to a typed datetime (would be a larger refactor across all crates)
- Adding timestamp validation or parsing anywhere in the pipeline (out of scope)
- Modifying how `add_duration_suffix()` in `atlassy-pipeline/src/util.rs` handles timestamps

## Decisions

### Decision 1: Use `chrono` over `std::time::SystemTime`

**Choice:** Add `chrono` with features `clock` and `serde`.

**Alternatives considered:**

- **`std::time::SystemTime`**: No new dependency, but formatting to RFC 3339 requires manual code (~15 lines of `strftime`-style formatting via the `time` crate or raw arithmetic). Error-prone and not idiomatic.
- **`time` crate**: Viable alternative, but `chrono` is more widely used in the Rust ecosystem (2x downloads), has a simpler RFC 3339 API (`Utc::now().to_rfc3339()`), and the `serde` feature provides free `DateTime` serialization if the field type is ever upgraded.

**Rationale:** `chrono` gives a one-liner (`Utc::now().to_rfc3339()`) that produces ISO 8601 / RFC 3339 output matching the existing hardcoded format. The `serde` feature is included to support a future migration from `String` to `DateTime<Utc>` without adding another dependency change.

### Decision 2: Workspace-level dependency, `atlassy-cli` only

**Choice:** Declare `chrono` in `[workspace.dependencies]` and consume it only in `atlassy-cli/Cargo.toml`.

**Rationale:** Only the CLI layer generates timestamps — the pipeline and contracts crates pass them through as `String`. Workspace-level declaration follows the existing pattern (all deps are workspace-managed) and makes `chrono` available to other crates later without a second `Cargo.toml` edit.

### Decision 3: Keep `default_manifest_timestamp()` as real UTC time

**Choice:** Replace the `"1970-01-01T00:00:00Z"` serde default with `Utc::now().to_rfc3339()`.

**Rationale:** When a manifest entry omits the `timestamp` field, the current epoch placeholder produces misleading telemetry. Using the parse-time UTC timestamp ensures manifest runs without explicit timestamps still get meaningful ordering. This is a serde `#[serde(default = "...")]` function, so it executes at deserialization time — the timestamp will reflect when the manifest was loaded, which is close enough to invocation time for batch runs.

## Risks / Trade-offs

- **New dependency** → Increases compile time slightly. Mitigated by: `chrono` is a well-maintained, widely-used crate with no transitive `unsafe` in default features. The `clock` feature pulls in `libc`/`winapi` for system clock access, which is standard.
- **Non-deterministic manifests** → `default_manifest_timestamp()` now produces different values on each parse. Mitigated by: manifest entries that care about timestamp accuracy should specify it explicitly. The default is a fallback for entries that omit it.
- **Format drift** → `to_rfc3339()` produces subsecond precision (e.g., `2026-03-11T10:00:00.123456789+00:00`) while the hardcoded values use second precision with `Z` suffix. Mitigated by: both are valid RFC 3339; all downstream consumers treat the field as an opaque string. No code parses or compares timestamps structurally.
