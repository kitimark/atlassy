# Atlassy

Atlassy is a Confluence content control toolkit for AI and MCP-driven editing -- insert, edit, and delete ADF blocks across pages and sub-pages.

## Installation

Resolve the latest release tag first:

```bash
VERSION="$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/kitimark/atlassy/releases/latest)"
VERSION="${VERSION##*/}"
```

macOS (Apple Silicon):

```bash
curl -fsSL "https://github.com/kitimark/atlassy/releases/download/${VERSION}/atlassy-cli-${VERSION}-aarch64-apple-darwin.tar.gz" | tar -xz
```

macOS (Intel):

```bash
curl -fsSL "https://github.com/kitimark/atlassy/releases/download/${VERSION}/atlassy-cli-${VERSION}-x86_64-apple-darwin.tar.gz" | tar -xz
```

Linux (x86_64):

```bash
curl -fsSL "https://github.com/kitimark/atlassy/releases/download/${VERSION}/atlassy-cli-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" | tar -xz
```

Linux (ARM64):

```bash
curl -fsSL "https://github.com/kitimark/atlassy/releases/download/${VERSION}/atlassy-cli-${VERSION}-aarch64-unknown-linux-gnu.tar.gz" | tar -xz
```

## Editing Model

### Foundation (current -- text replacement)

- ADF is canonical for fetch, diff, patch, verify, and publish.
- Markdown is a transient assist format for prose-only editing.
- Tables are edited via ADF-native operations (cell text only).
- Unsupported structural blocks remain locked.

### Structural (planned -- insert/edit/delete)

- Insert new ADF blocks (paragraphs, headings, tables, lists) at specified positions within scope.
- Delete existing ADF blocks within scope.
- Compose structures: insert full sections (heading + body), create new tables, create new lists.
- Multi-page content control: create sub-pages with content, coordinate edits across page hierarchies.
- Advanced operations: table topology changes (row/column add/remove), structural block attribute editing, MCP server integration.

## Implementation Stack

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

- Achieve `operation_success_rate` >95% for all insert/edit/delete operations.
- Maintain `schema_validity_rate` at 100% for all structural operations (insert/delete).
- Achieve `operation_precision` at 100% -- only declared target blocks are affected.
- Maintain `structural_integrity` at 100% -- non-target structures preserved.
- Keep `conflict_rate` below 10% with one scoped retry cap.
- Maintain fast `publish_latency` (median <3000ms).

## Non-goals

- Multi-space orchestration (cross-space page operations).
- Fully autonomous conflict resolution without human review.
- Block type conversion (paragraph to heading, etc.).
- Inline node editing (mentions, status, emoji, date).

## Approach

Use a minimal-change pipeline:

`fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`

## Planned Capabilities

### Pipeline States

`fetch -> classify -> extract_prose -> md_assist_edit -> adf_table_edit -> merge_candidates -> patch -> verify -> publish`

### Operation Types

- `replace` (Foundation): text-value replacement at leaf paths within existing blocks.
- `insert` (Structural): add new ADF blocks at specified positions within scope.
- `remove` (Structural): delete existing ADF blocks within scope.

### Structural Operations (Phases 6-9)

- Section composition: insert/delete full sections (heading + body blocks).
- Table creation: insert new tables with specified dimensions.
- List creation: insert new lists with specified items.
- Multi-page orchestration: coordinated edits across page hierarchies with rollback.

### Lifecycle Commands

- `create-subpage` (child page creation under an explicit parent)
- `--bootstrap-empty-page` (explicit first-edit scaffold injection for empty pages)

### Advanced Operations (Phase 9)

- Table topology changes (row/column add/remove).
- Structural block attribute editing (media metadata, macro parameters).
- MCP server integration (expose pipeline as MCP tools for AI agents).

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

- **Foundation (Phases 0-5): complete.** All 5 implementation phases delivered. Live Confluence runtime operational. 159 tests pass. All 7 readiness gates pass. 3 releases shipped (v0.1.0 through v0.1.2).
- **Structural (Phases 5.5-9): planning active.** Roadmap redesigned for structural operations. Decisions D-017 through D-021 define the architectural approach. Phase 5.5 (Structural Refactor) is the next implementation target: type consolidation and pipeline preparation with zero behavior change, followed by Phase 6 (Block Operation Foundation).
- Foundation KPI framework superseded by Structural KPI framework (D-019). The Pattern B context reduction issue is expected to be addressable through Phase 6 structural operations.

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

### Primary (Structural)

- `operation_success_rate`
- `schema_validity_rate`
- `operation_precision`
- `structural_integrity`
- `conflict_rate`
- `publish_latency`

### Diagnostic (from Foundation)

- `context_reduction_ratio`
- `scoped_section_tokens`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
