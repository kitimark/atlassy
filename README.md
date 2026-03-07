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

Large Confluence pages increase token usage and make updates fragile.

- Context windows overflow when full pages are injected into prompts.
- Large payload updates can fail or time out.
- Full-body updates increase version drift and conflict retries.
- Formatting fidelity can regress (tables, macros, media) after repeated edits.

## Goals

- Reduce `tokens_per_successful_update` by 40-60%.
- Reduce full-page retrievals by 60-80%.
- Maintain or improve publish success and formatting fidelity.

## Non-goals (v1)

- Full multi-space orchestration on day one.
- Fully autonomous conflict resolution without human review.

## Approach

Use a minimal-change pipeline:

`fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`

## Planned Capabilities

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

- Implementation checkpoint: phases 1-5 are implemented for stub and fixture-backed execution.
- Current readiness decision: `iterate` (KPI target misses for token reduction and full-page retrieval reduction).
- Real Confluence runtime is not enabled yet (`LiveConfluenceClient` remains pending).

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

- `tokens_per_successful_update`
- `full_page_retrieval_rate`
- `retry_conflict_token_waste`
- `formatting_fidelity_pass_rate`
- `publish_latency`

## Contributing

Use a design-first flow.

- Add or update roadmap docs before implementation changes.
- Record major tradeoffs and defaults in decision notes.
- Keep proposals testable with measurable success criteria.
