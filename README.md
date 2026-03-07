# Atlassy

Atlassy is a token-efficient Confluence workflow toolkit for AI and MCP-driven editing.

## Editing Model (v1 default)

- ADF is canonical for fetch, diff, patch, verify, and publish.
- Markdown is a transient assist format for prose-only editing.
- Tables are edited via ADF-native operations (cell text only in v1).
- Unsupported structural blocks remain locked until dedicated support is added.

## Implementation Stack (v1 default)

- Runtime language: Rust (stable toolchain).
- Architecture: CLI-first workspace with shared core libraries for pipeline states and ADF operations.
- Confluence integration: async HTTP client with typed request/response contracts.
- Verification and diagnostics: contract-validated state envelopes, structured tracing logs, and replay artifacts.
- Testing: live Confluence research in sandbox plus fixture-backed stub simulation for CI and regression.

## Problem

AI-assisted Confluence updates are expensive and fragile when they rely on full-page context and broad rewrites.

- Full-page payloads inflate context windows and token spend.
- Large write payloads increase publish failures and timeout risk.
- Full-body rewrites amplify version drift and conflict retries.
- Repeated broad edits increase fidelity risk for tables, macros, and media.

## Goals

- Reach `context_reduction_ratio` of 70-90% for in-scope optimized runs.
- Keep scoped payload size observable via `scoped_section_tokens` (median/p90 by pattern).
- Achieve `edit_success_rate` >95% and `structural_preservation` at 100% for in-scope runs.
- Keep `conflict_rate` below 10% with one scoped retry cap and maintain fast `publish_latency`.

## Non-goals (v1)

- Full multi-space orchestration on day one.
- Fully autonomous conflict resolution without human review.

## Approach

Use a minimal-change pipeline:

`fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`

## Planned Capabilities

Pipeline states:

- `fetch`
- `classify`
- `extract_prose`
- `md_assist_edit`
- `adf_table_edit`
- `merge_candidates`
- `patch`
- `verify`
- `publish`
- `cache`

Lifecycle commands:

- `create-subpage` (blank child page creation under an explicit parent)
- `--bootstrap-empty-page` (explicit first-edit scaffold injection for empty pages)

## Roadmap

Detailed planning docs are under `roadmap/`.

Start here: `roadmap/README.md`.

## Ideas

Incubating concepts that are not scheduled yet are stored under `ideas/`.

Start here: `ideas/README.md`.

## OpenSpec

Executable specs and change archives are under `openspec/`.

Start here: `openspec/README.md`.

## Current Status

- Implementation checkpoint: phases 1-5 are implemented. Live Confluence runtime (`LiveConfluenceClient`) is operational and validated in sandbox.
- Lifecycle features (`create-subpage`, `--bootstrap-empty-page`) are implemented and validated with committed live evidence.
- Current readiness decision: `iterate` (legacy KPI framing from stub-backed evaluation is superseded; revised KPI revalidation with scoped-selector manifests is pending).

## Project Structure

- `crates/`: Rust workspace implementation (`atlassy-cli`, `atlassy-pipeline`, `atlassy-adf`, `atlassy-confluence`, `atlassy-contracts`).
- `roadmap/`: strategy, defaults, KPI protocol, risks, and readiness policy.
- `openspec/specs/`: active behavior specs for implemented capabilities.
- `openspec/changes/archive/`: completed phase execution records.
- `qa/`: sandbox QA playbooks and example manifests for live validation.
- `ideas/`: incubating or deferred scope beyond v1.
- `artifacts/`: temporary CLI outputs (`run`, `run-batch`, `run-readiness`), intentionally non-versioned.

## Reproducibility Metadata

`artifacts/` is temporary and may be deleted at any time. Any result referenced in docs or decisions must include provenance:

- `git_commit_sha` (full 40-character SHA)
- `git_dirty` (whether the working tree had local changes)
- `pipeline_version`
- command set used to regenerate outputs

Suggested capture sequence before running experiments:

```bash
git rev-parse HEAD
git status --porcelain
cargo run -p atlassy-cli -- run ...
cargo run -p atlassy-cli -- run-batch ...
cargo run -p atlassy-cli -- run-readiness --verify-replay ...
```

## Success Metrics

- `context_reduction_ratio`
- `scoped_section_tokens`
- `edit_success_rate`
- `structural_preservation`
- `conflict_rate`
- `publish_latency`

## Contributing

Use a design-first flow.

- Add or update roadmap docs before implementation changes.
- Record major tradeoffs and defaults in decision notes.
- Keep proposals testable with measurable success criteria.
