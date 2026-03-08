# 2026-03-08 Heading Discovery Defect Reproduction

## Provenance

- `git_commit_sha`: `ac57271f04722db2a8b43841ce0bcf574ca9524f`
- `git_dirty`: `false`
- `pipeline_version`: `v1`
- `runtime_mode`: `live`
- Reproduction page: `66125` (child of `131207`)
- Artifacts: `artifacts/repro-baseline-01/`, `artifacts/repro-optimized-01/`

## Objective

Reproduce the auto-discovery heading text defect from the KPI revalidation v2 investigation in a minimal, isolated 2-run sequence against live Confluence.

## Setup

1. Created fresh subpage `66125` ("Bug Repro - Heading Discovery") under sandbox root `131207`.
2. Bootstrapped with `--bootstrap-empty-page` (version 2).
3. Seeded via Confluence REST API with ADF: heading "Introduction" + 2 paragraphs (version 3).
4. Preflight confirmed structure:

```
/content/0  heading   "Introduction"
/content/1  paragraph "This is the body paragraph under the introduction heading."
/content/2  paragraph "Second paragraph with additional content for the section."
```

## Reproduction

### Run 1: Baseline (no scope, auto-discovery)

```bash
cargo run -p atlassy-cli -- run \
  --request-id "repro-baseline-01" \
  --page-id 66125 \
  --mode simple-scoped-prose-update \
  --new-value "Baseline overwrites this node" \
  --runtime-backend live
```

| Field | Value |
|-------|-------|
| `discovered_target_path` | `/content/0/content/0/text` **(heading text)** |
| `applied_paths` | `["/content/0/content/0/text"]` |
| `publish_result` | `published` |
| `new_version` | `4` |

**Result**: Auto-discovery selected the heading text node at index 0. The heading "Introduction" was overwritten to "Baseline overwrites this node".

### Run 2: Optimized (scoped, heading:Introduction)

```bash
cargo run -p atlassy-cli -- run \
  --request-id "repro-optimized-01" \
  --page-id 66125 \
  --scope "heading:Introduction" \
  --mode simple-scoped-prose-update \
  --new-value "Optimized scoped update" \
  --runtime-backend live
```

| Field | Value |
|-------|-------|
| `scope_selectors` | `["heading:Introduction"]` |
| `scope_resolution_failed` | `true` |
| `full_page_fetch` | `true` |
| `context_reduction_ratio` | `0.0` |
| `publish_result` | `published` |
| `new_version` | `5` |

**Result**: Heading "Introduction" no longer exists on the page (overwritten by Run 1). Scope resolution fell back to full page. Context reduction is zero.

### Page State Transition

| State | Heading Text | Paragraph 1 | Paragraph 2 |
|-------|-------------|-------------|-------------|
| Before Run 1 | "Introduction" | "This is the body paragraph..." | "Second paragraph..." |
| After Run 1 | **"Baseline overwrites this node"** | (unchanged) | (unchanged) |
| After Run 2 | **"Optimized scoped update"** | (unchanged) | (unchanged) |

## Root Cause Confirmed

`discover_target_path()` in `crates/atlassy-adf/src/lib.rs:273-308` treats heading text nodes as valid prose targets because `EDITABLE_PROSE_TYPES` (line 46-54) includes `"heading"`. Candidates are string-sorted, so `/content/0/content/0` (heading child) sorts before `/content/1/content/0` (paragraph child) when the heading is at a low content index.

The defect is index-dependent: it reliably triggers for headings at `/content/0` through `/content/9` (single-digit indices). For headings at `/content/10`+, string sort may place a paragraph child first (e.g., `/content/10/...` sorts before `/content/8/...`). This was confirmed by a Phase 1 preflight on P1 with `heading:Summary` (at `/content/8`), where discovery returned `/content/10/content/0/text` (a paragraph), not the heading.

## Comparison with v2 Batch

This reproduction exactly matches the 3 failed optimized runs from the KPI revalidation v2 batch:

| v2 Batch Run | Reproduction Run | `scope_resolution_failed` | `context_reduction_ratio` |
|-------------|-----------------|--------------------------|--------------------------|
| `kpi-v2-a-opt-01` (heading:Introduction) | `repro-optimized-01` (heading:Introduction) | `true` | `0.0` |
| `kpi-v2-b-opt-01` (heading:Overview) | (same pattern) | `true` | `0.0` |
| `kpi-v2-c-opt-01` (heading:Context) | (same pattern) | `true` | `0.0` |

All three v2 failures shared the same signature: first heading at `/content/0`, baseline auto-discovered heading text, overwrote it, subsequent optimized run's heading selector failed.

## Additional Finding: String Sort Sensitivity

The bug's manifestation depends on string-sorted content indices:

- **Triggers** when heading is at `/content/0`-`/content/9` (heading child path sorts before paragraph child paths).
- **Does not trigger** when heading is at `/content/10`+ and paragraphs are at lower indices within the section, because string comparison of multi-digit indices is non-numeric (`"10" < "8"` as strings).

This means the fix in `discover_target_path` should not rely on sort order but should explicitly filter heading ancestors from prose route candidates.
