## Why

Provenance fields `git_commit_sha` and `git_dirty` are currently resolved at runtime by shelling out to `git`. This means provenance reflects whichever repository the CLI happens to run in, not the commit that built the binary. A binary built from commit `abc123` on a clean tree will report a different SHA and dirty state if executed from a different checkout. Build-time embedding fixes this so provenance is always tied to the compiled binary.

## What Changes

- Add a `build.rs` to `atlassy-cli` that captures `git_commit_sha` and `git_dirty` at compile time via `cargo:rustc-env`.
- Replace runtime git subprocess calls in `collect_provenance()` with compile-time constants read via `env!()`.
- Remove `resolve_git_commit_sha()` and `resolve_git_dirty()` functions.
- Rename the `runtime-provenance-stamping` spec to `build-provenance-stamping` to reflect the new sourcing model.
- Add a smoke test asserting the embedded SHA is well-formed.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `runtime-provenance-stamping`: Provenance values (`git_commit_sha`, `git_dirty`) change from runtime-resolved to compile-time-embedded. Rename spec to `build-provenance-stamping`.

## Impact

- Affected code: `crates/atlassy-cli` (new `build.rs`, updated `provenance.rs`).
- Affected tests: `provenance.rs` no longer requires git at test runtime; existing validation and mismatch tests unchanged.
- No API or output format changes — `ProvenanceStamp` struct and JSON shape remain identical.
