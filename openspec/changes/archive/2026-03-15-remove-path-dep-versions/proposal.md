## Why

Internal path dependencies in `Cargo.toml` include `version = "0.1.0"`, but this version is stale (workspace is at `0.1.2`) and serves no purpose — Cargo ignores `version` when `path` is present. The dead config is misleading and creates maintenance drift.

## What Changes

- Remove `version = "0.1.0"` from all 7 internal path dependency declarations across 3 crate `Cargo.toml` files (`atlassy-adf`, `atlassy-pipeline`, `atlassy-cli`).

## Capabilities

### New Capabilities

- `path-dep-hygiene`: Convention for how internal workspace crate dependencies are declared in Cargo.toml

### Modified Capabilities

None.

## Impact

- **Files**: `crates/atlassy-adf/Cargo.toml`, `crates/atlassy-pipeline/Cargo.toml`, `crates/atlassy-cli/Cargo.toml`
- **Dependencies**: No functional dependency changes — only removing unused metadata
- **Build**: No effect on compilation or resolution (Cargo already ignores these values locally)
- **Publishing**: Project has no publish intent (missing required crates.io metadata); if that changes, `version` can be re-added at that time
