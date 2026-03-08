# Confluence Sandbox Test Plan

## Objective

Run Atlassy end-to-end against real Confluence sandbox pages with explicit safety controls, deterministic artifacts, and clear pass/fail checks.

## Scope

- Covers `run`, `create-subpage`, optional `run-batch`, and optional `run-readiness` in `live` runtime mode.
- Uses v1-safe operations only (scoped prose and table cell text updates, subpage creation, and empty-page bootstrap).
- Includes scoped-selector validation for revised KPI experiments.
- Focuses on sandbox validation, not production release sign-off.

Latest investigation:

- `qa/investigations/2026-03-07-lifecycle-subpage-bootstrap-validation.md`

## Preconditions

- You have a dedicated sandbox Confluence page ID.
- You have sandbox API credentials (email + API token).
- Rust toolchain is installed (`cargo` available).
- Optional but recommended: `jq` for fast artifact checks.
- You know the space key for your sandbox page (visible in the space URL, e.g., `~username` for personal spaces or a short key like `ENG`).
- For KPI experiments (Step 8): 3 sandbox pages with structural variety (prose-only, prose+table, prose+locked blocks) created via KPI Experiment Page Setup section.
- For KPI experiments: page inventory recorded in `qa/manifests/sandbox-page-inventory.md`.
- For KPI experiments: Step 2b per-page scoped fetch spike passed for all experiment pages.

## Safety Rules

- Never paste API tokens in chat, docs, commits, or manifests.
- Keep secrets in environment variables only.
- Use unique `request_id` per run.
- Start with a no-write preflight before any live publish.
- Do not run these steps against production pages.

## Environment Setup

```bash
make qa-setup
```

Notes:

- The setup script prompts for `ATLASSY_CONFLUENCE_BASE_URL`, `ATLASSY_CONFLUENCE_EMAIL`, `ATLASSY_CONFLUENCE_API_TOKEN`, `ARTIFACTS_DIR`, optional `PAGE_ID`, and optional `SPACE_KEY`.
- Values are saved to `qa/.env.local`.
- Do not append `/wiki` to `ATLASSY_CONFLUENCE_BASE_URL`.
- `qa/.env.local` is git-ignored and required by `make qa-check`.
- Re-run `make qa-setup` whenever credentials or target page values change.

## Step 0: Environment Prerequisite Check

After setting up environment variables, validate the environment before proceeding:

```bash
make qa-check
```

This validates:

- Required env vars are set (`ATLASSY_CONFLUENCE_BASE_URL`, `ATLASSY_CONFLUENCE_EMAIL`, `ATLASSY_CONFLUENCE_API_TOKEN`).
- Base URL format (must use `https://`, must not end with `/wiki`).
- API token length sanity.
- Confluence API connectivity (read-only probe against `/wiki/rest/api/user/current`).

All checks must pass before continuing to Step 1. If any fail, re-run `make qa-setup`.

## KPI Experiment Page Setup

For full KPI revalidation across all three edit patterns (A, B, C), create dedicated sandbox pages with structural variety. Single-page smoke testing (Steps 1-7) does not require this section.

Page inventory reference: `qa/manifests/sandbox-page-inventory.md`.

### Create Experiment Subpages

Create three blank child pages under the primary sandbox page:

```bash
# P1: Prose-Rich (Pattern A target)
cargo run -p atlassy-cli -- create-subpage \
  --parent-page-id "$PAGE_ID" \
  --space-key "$SPACE_KEY" \
  --title "KPI Experiment - Prose Rich" \
  --runtime-backend live

# P2: Mixed Prose+Table (Pattern B target)
cargo run -p atlassy-cli -- create-subpage \
  --parent-page-id "$PAGE_ID" \
  --space-key "$SPACE_KEY" \
  --title "KPI Experiment - Mixed Prose Table" \
  --runtime-backend live

# P3: Locked-Adjacent (Pattern C target)
cargo run -p atlassy-cli -- create-subpage \
  --parent-page-id "$PAGE_ID" \
  --space-key "$SPACE_KEY" \
  --title "KPI Experiment - Locked Adjacent" \
  --runtime-backend live
```

Capture the page IDs from each command's JSON output:

```bash
export P1_PAGE_ID="REPLACE_WITH_P1_PAGE_ID"
export P2_PAGE_ID="REPLACE_WITH_P2_PAGE_ID"
export P3_PAGE_ID="REPLACE_WITH_P3_PAGE_ID"
```

### Bootstrap Each Page

Inject a heading + paragraph scaffold so pages are non-empty:

```bash
cargo run -p atlassy-cli -- run \
  --request-id "bootstrap-p1" \
  --page-id "$P1_PAGE_ID" \
  --edit-intent "bootstrap scaffold injection" \
  --mode no-op --bootstrap-empty-page \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"

cargo run -p atlassy-cli -- run \
  --request-id "bootstrap-p2" \
  --page-id "$P2_PAGE_ID" \
  --edit-intent "bootstrap scaffold injection" \
  --mode no-op --bootstrap-empty-page \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"

cargo run -p atlassy-cli -- run \
  --request-id "bootstrap-p3" \
  --page-id "$P3_PAGE_ID" \
  --edit-intent "bootstrap scaffold injection" \
  --mode no-op --bootstrap-empty-page \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"
```

### Add Structural Content via Confluence UI

Open each page in the Confluence editor and replace/extend the bootstrap scaffold:

| Page | Content to Add |
|------|---------------|
| P1 | Heading "Introduction" + 2-3 paragraphs. Heading "Details" + bullet list + blockquote. Heading "Summary" + closing paragraph. |
| P2 | Heading "Overview" + paragraph. Heading "Data" + paragraph + 3x3 table with text cells + paragraph after table. |
| P3 | Heading "Context" + paragraph. Expand macro (`/expand`). Heading "Notes" + paragraph. Image or Jira macro (`/image` or `/jira`). Heading "References" + paragraph. |

### Route Classification Guidance

The route classifier uses a 7-type prose whitelist (`paragraph`, `heading`, `bulletList`, `orderedList`, `listItem`, `blockquote`, `codeBlock`). Table family nodes route to `table_adf`. Everything else routes to `locked_structural` via catch-all.

Blocks that produce `locked_structural` nodes:

- Expand macro (`expand` node).
- Jira/Forge/Connect macros (`extension` or `bodiedExtension` node).
- Attached images (`mediaSingle` > `media` nodes).
- Layout columns (`layoutSection` > `layoutColumn` nodes).
- Info/note/warning panels (`panel` node — but inner paragraphs are still `editable_prose`).

Heading naming rules: avoid substring overlaps on the same page. The scope resolver uses `text.contains()` matching, so `heading:View` would match a heading titled "Overview".

### Record Page Inventory

After content is added, update `qa/manifests/sandbox-page-inventory.md` with page IDs and confirmed structural characteristics.

## Step 1: No-Write Live Preflight

`no-op` can still publish, so force verify failure to block publish intentionally.

```bash
export RUN_ID_PREFLIGHT="live-preflight-001"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_PREFLIGHT" \
  --page-id "$PAGE_ID" \
  --edit-intent "sandbox live preflight" \
  --mode no-op \
  --runtime-backend live \
  --force-verify-fail \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Expected outcome:

- Command exits non-zero (expected).
- Run summary exists at `artifacts/$RUN_ID_PREFLIGHT/summary.json`.
- Failure state is `verify`; publish is not reached.

Quick checks:

```bash
jq '{success,failure_state,error_codes,runtime_mode,full_page_fetch,scope_resolution_failed}' \
  "artifacts/$RUN_ID_PREFLIGHT/summary.json"
```

## Step 2: Discover Safe Target Paths

List candidate text paths from the fetched scoped payload:

```bash
jq -r '.payload.scoped_adf | paths(scalars) as $p | select($p[-1] == "text") | "/" + ($p | map(tostring) | join("/"))' \
  "artifacts/$RUN_ID_PREFLIGHT/fetch/state_output.json"
```

Pick:

- One prose path (not under table ancestry).
- Optional one table cell text path (for table smoke test).

## Step 2b: Validate Scoped Selector Fetch Behavior (No Publish)

Run a no-write check with an explicit scope selector. Keep `--force-verify-fail` so publish is intentionally blocked.

### Single-page scoped fetch (basic smoke)

```bash
export RUN_ID_SCOPED_FETCH="live-scoped-fetch-001"
export SCOPE_SELECTOR="heading:REPLACE_WITH_SECTION_HEADING"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_SCOPED_FETCH" \
  --page-id "$PAGE_ID" \
  --edit-intent "sandbox scoped fetch validation" \
  --scope "$SCOPE_SELECTOR" \
  --mode no-op \
  --runtime-backend live \
  --force-verify-fail \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Expected outcome:

- Command exits non-zero (expected, verify fail forced).
- `scope_resolution_failed: false`.
- `full_page_fetch: false`.

Check:

```bash
jq '{success,failure_state,error_codes,scope_selectors,scope_resolution_failed,full_page_fetch,context_reduction_ratio}' \
  "artifacts/$RUN_ID_SCOPED_FETCH/summary.json"
```

Notes:

- If `context_reduction_ratio` is emitted by your current build, it should be >0 for valid scoped runs.
- If this run falls back to full page fetch, fix selector quality before KPI batch execution.

### KPI experiment: per-page scoped fetch spike

For KPI revalidation, validate scope resolution against all three experiment pages before running the full batch. Run a full-page preflight first, then a scoped fetch for each page.

#### Full-page preflight (all 3 pages)

```bash
for PAGE_LABEL in p1 p2 p3; do
  eval PAGE_VAR="\$$(echo ${PAGE_LABEL}_PAGE_ID | tr '[:lower:]' '[:upper:]')"
  cargo run -p atlassy-cli -- run \
    --request-id "spike-fetch-${PAGE_LABEL}-full" \
    --page-id "$PAGE_VAR" \
    --edit-intent "scoped fetch spike" \
    --mode no-op --force-verify-fail \
    --runtime-backend live \
    --artifacts-dir "$ARTIFACTS_DIR"
done
```

#### Scoped fetch (one heading per page)

```bash
# P1: prose-only page
cargo run -p atlassy-cli -- run \
  --request-id "spike-fetch-p1-scoped" \
  --page-id "$P1_PAGE_ID" \
  --edit-intent "scoped fetch spike" \
  --scope "heading:Introduction" \
  --mode no-op --force-verify-fail \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"

# P2: mixed prose+table page
cargo run -p atlassy-cli -- run \
  --request-id "spike-fetch-p2-scoped" \
  --page-id "$P2_PAGE_ID" \
  --edit-intent "scoped fetch spike" \
  --scope "heading:Data" \
  --mode no-op --force-verify-fail \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"

# P3: locked-adjacent page
cargo run -p atlassy-cli -- run \
  --request-id "spike-fetch-p3-scoped" \
  --page-id "$P3_PAGE_ID" \
  --edit-intent "scoped fetch spike" \
  --scope "heading:Notes" \
  --mode no-op --force-verify-fail \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"
```

#### Per-page pass criteria

For each scoped fetch run, check:

```bash
jq '{scope_selectors,scope_resolution_failed,full_page_fetch,context_reduction_ratio,full_page_adf_bytes,scoped_adf_bytes}' \
  "artifacts/spike-fetch-p1-scoped/summary.json"
```

All three must show:

- `scope_resolution_failed: false`
- `full_page_fetch: false`
- `context_reduction_ratio > 0`
- `scoped_adf_bytes < full_page_adf_bytes`

#### Discover target paths

For each full-page preflight, list candidate text paths:

```bash
jq -r '.payload.scoped_adf | paths(scalars) as $p | select($p[-1] == "text") | "/" + ($p | map(tostring) | join("/"))' \
  "artifacts/spike-fetch-p1-full/fetch/state_output.json"
```

Pick prose and table cell text paths per page. Record these in `qa/manifests/sandbox-page-inventory.md`.

#### Decision gate: substring matching

If any scoped fetch unexpectedly matches multiple headings or the wrong heading due to substring matching (the resolver uses `text.contains()`), either:

- Choose more specific heading names and re-edit the page, or
- Fix `resolve_scope` to use exact matching (small code change in `crates/atlassy-adf/src/lib.rs`).

Do not proceed to the KPI batch until all three pages pass the scoped fetch spike.

#### Auto-discovery validation

Validate that the pipeline can auto-select targets without manual `jq` path discovery.

Run a scoped prose update with `--target-path` omitted. Use `--mode simple-scoped-prose-update` (not `no-op`) so the pipeline enters the edit state where discovery runs. Use `--force-verify-fail` to block publish:

```bash
cargo run -p atlassy-cli -- run \
  --request-id "spike-autodiscover-p1" \
  --page-id "$P1_PAGE_ID" \
  --edit-intent "auto-discovery validation" \
  --scope "heading:Introduction" \
  --mode simple-scoped-prose-update \
  --new-value "auto-discovery validation text" \
  --force-verify-fail \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"
```

Check that auto-discovery resolved a target:

```bash
jq '{discovered_target_path,scope_resolution_failed}' \
  "artifacts/spike-autodiscover-p1/summary.json"
```

Pass criteria:

- `discovered_target_path` is non-null and points to a valid text node within the heading section.
- `scope_resolution_failed: false`.

Repeat for P2 (prose and table cell routes) and P3. For P2, also validate table cell auto-discovery:

```bash
cargo run -p atlassy-cli -- run \
  --request-id "spike-autodiscover-p2-table" \
  --page-id "$P2_PAGE_ID" \
  --edit-intent "auto-discovery table validation" \
  --scope "heading:Data" \
  --mode simple-scoped-table-cell-update \
  --new-value "auto-discovery table validation text" \
  --force-verify-fail \
  --runtime-backend live --artifacts-dir "$ARTIFACTS_DIR"
```

Compare `discovered_target_path` against the manually discovered paths in `qa/manifests/sandbox-page-inventory.md` to verify consistency.

## Step 3: Live Scoped Prose Update

```bash
export RUN_ID_PROSE="live-prose-001"
export PROSE_PATH="REPLACE_WITH_PROSE_TEXT_PATH"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_PROSE" \
  --page-id "$PAGE_ID" \
  --edit-intent "sandbox scoped prose update" \
  --mode simple-scoped-prose-update \
  --target-path "$PROSE_PATH" \
  --new-value "Sandbox prose update $(date -u +%FT%TZ)" \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Success checks:

```bash
jq '{success,publish_result,new_version,retry_count,error_codes,runtime_mode}' \
  "artifacts/$RUN_ID_PROSE/summary.json"
```

Expected:

- `success: true`
- `publish_result: "published"`
- `runtime_mode: "live"`

## Step 4 (Optional): Live Table Cell Text Update

Run this only if your sandbox page contains a table.

```bash
export RUN_ID_TABLE="live-table-001"
export TABLE_PATH="REPLACE_WITH_TABLE_CELL_TEXT_PATH"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_TABLE" \
  --page-id "$PAGE_ID" \
  --edit-intent "sandbox scoped table cell update" \
  --mode simple-scoped-table-cell-update \
  --target-path "$TABLE_PATH" \
  --new-value "Sandbox table update $(date -u +%FT%TZ)" \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

## Step 5: Negative Safety Check (Expected Failure)

Choose one negative check:

Option A (table shape safety, requires `TABLE_PATH` from Step 4):

```bash
export RUN_ID_NEGATIVE="live-negative-table-shape-001"
export INVALID_TABLE_PATH="${TABLE_PATH%/text}"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_NEGATIVE" \
  --page-id "$PAGE_ID" \
  --edit-intent "sandbox negative table shape check" \
  --mode simple-scoped-table-cell-update \
  --target-path "$INVALID_TABLE_PATH" \
  --new-value "Should fail" \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Option B (prose boundary safety, works with Step 3 prose path):

```bash
export RUN_ID_NEGATIVE="live-negative-prose-boundary-001"
export INVALID_PROSE_PATH="${PROSE_PATH%/text}/type"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_NEGATIVE" \
  --page-id "$PAGE_ID" \
  --edit-intent "sandbox negative prose boundary check" \
  --mode simple-scoped-prose-update \
  --target-path "$INVALID_PROSE_PATH" \
  --new-value "Should fail" \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Expected outcome:

- Command exits non-zero (expected).
- `artifacts/$RUN_ID_NEGATIVE/summary.json` includes deterministic safety code(s), typically `ERR_TABLE_SHAPE_CHANGE` (Option A) or `ERR_SCHEMA_INVALID` (Option B).

Check:

```bash
jq '{success,failure_state,error_codes,publish_result}' \
  "artifacts/$RUN_ID_NEGATIVE/summary.json"
```

## Step 6: Live Create Subpage

Create a blank child page under your sandbox page. This tests the `create-subpage` command against live Confluence and produces an empty page for bootstrap testing in Step 7.

```bash
export SPACE_KEY="${SPACE_KEY:-REPLACE_WITH_SPACE_KEY}"
export SUBPAGE_TITLE="Atlassy Bootstrap Test $(date -u +%FT%TZ)"

cargo run -p atlassy-cli -- create-subpage \
  --parent-page-id "$PAGE_ID" \
  --space-key "$SPACE_KEY" \
  --title "$SUBPAGE_TITLE" \
  --runtime-backend live
```

Expected outcome:

- Command exits zero.
- Stdout prints JSON with `page_id` and `page_version`.

Capture the created page ID for Step 7:

```bash
export CREATED_PAGE_ID="REPLACE_WITH_CREATED_PAGE_ID_FROM_OUTPUT"
```

Quick check:

- `page_version` should be `1`.
- Verify the page exists in Confluence UI under your sandbox page.

## Step 7: Bootstrap Empty Page Detection

Uses the empty page created in Step 6 to validate the four-path bootstrap detection matrix.

### Step 7a: Empty page without bootstrap flag (expect ERR_BOOTSTRAP_REQUIRED)

```bash
export RUN_ID_BOOTSTRAP_REQ="live-bootstrap-required-001"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_BOOTSTRAP_REQ" \
  --page-id "$CREATED_PAGE_ID" \
  --edit-intent "bootstrap required detection" \
  --mode no-op \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Expected outcome:

- Command exits non-zero.
- Failure at `fetch` state with `ERR_BOOTSTRAP_REQUIRED`.

Check:

```bash
jq '{success,failure_state,error_codes,empty_page_detected,bootstrap_applied}' \
  "artifacts/$RUN_ID_BOOTSTRAP_REQ/summary.json"
```

Expected: `success: false`, `failure_state: "fetch"`, `error_codes: ["ERR_BOOTSTRAP_REQUIRED"]`, `empty_page_detected: true`, `bootstrap_applied: false`.

### Step 7b: Empty page with bootstrap flag (expect success)

```bash
export RUN_ID_BOOTSTRAP_OK="live-bootstrap-success-001"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_BOOTSTRAP_OK" \
  --page-id "$CREATED_PAGE_ID" \
  --edit-intent "bootstrap scaffold injection" \
  --mode no-op \
  --bootstrap-empty-page \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Expected outcome:

- Command exits zero.
- Pipeline injects scaffold ADF and completes all states.
- The empty page now has content (heading + paragraph) published to Confluence.

Check:

```bash
jq '{success,publish_result,empty_page_detected,bootstrap_applied,full_page_fetch,scope_resolution_failed}' \
  "artifacts/$RUN_ID_BOOTSTRAP_OK/summary.json"
```

Expected: `success: true`, `publish_result: "published"`, `empty_page_detected: true`, `bootstrap_applied: true`, `full_page_fetch: true`, `scope_resolution_failed: true`.

### Step 7c: Non-empty page with bootstrap flag (expect ERR_BOOTSTRAP_INVALID_STATE)

Uses the original sandbox page (which has content).

```bash
export RUN_ID_BOOTSTRAP_INV="live-bootstrap-invalid-001"

cargo run -p atlassy-cli -- run \
  --request-id "$RUN_ID_BOOTSTRAP_INV" \
  --page-id "$PAGE_ID" \
  --edit-intent "bootstrap invalid state detection" \
  --mode no-op \
  --bootstrap-empty-page \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

Expected outcome:

- Command exits non-zero.
- Failure at `fetch` state with `ERR_BOOTSTRAP_INVALID_STATE`.

Check:

```bash
jq '{success,failure_state,error_codes,empty_page_detected,bootstrap_applied}' \
  "artifacts/$RUN_ID_BOOTSTRAP_INV/summary.json"
```

Expected: `success: false`, `failure_state: "fetch"`, `error_codes: ["ERR_BOOTSTRAP_INVALID_STATE"]`, `empty_page_detected: false`, `bootstrap_applied: false`.

## Step 8: KPI Revalidation Batch

Requires: KPI Experiment Page Setup completed, Step 2b per-page scoped fetch spike passed for all three pages.

### Run Matrix

18 runs total: 6 per pattern (3 baseline/optimized pairs). Each pair shares an `edit_intent_hash`. Baseline runs use empty `scope_selectors`; optimized runs use explicit heading selectors.

| Run ID | Page | Pattern | Flow | scope_selectors | Mode |
|--------|------|---------|------|-----------------|------|
| `kpi-a-base-01` | P1 | A | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-a-opt-01` | P1 | A | optimized | `["heading:Introduction"]` | `simple_scoped_prose_update` |
| `kpi-a-base-02` | P1 | A | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-a-opt-02` | P1 | A | optimized | `["heading:Details"]` | `simple_scoped_prose_update` |
| `kpi-a-base-03` | P1 | A | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-a-opt-03` | P1 | A | optimized | `["heading:Summary"]` | `simple_scoped_prose_update` |
| `kpi-b-base-01` | P2 | B | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-b-opt-01` | P2 | B | optimized | `["heading:Data"]` | `simple_scoped_prose_update` |
| `kpi-b-base-02` | P2 | B | baseline | `[]` | `simple_scoped_table_cell_update` |
| `kpi-b-opt-02` | P2 | B | optimized | `["heading:Data"]` | `simple_scoped_table_cell_update` |
| `kpi-b-base-03` | P2 | B | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-b-opt-03` | P2 | B | optimized | `["heading:Overview"]` | `simple_scoped_prose_update` |
| `kpi-c-base-01` | P3 | C | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-c-opt-01` | P3 | C | optimized | `["heading:Context"]` | `simple_scoped_prose_update` |
| `kpi-c-base-02` | P3 | C | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-c-opt-02` | P3 | C | optimized | `["heading:Notes"]` | `simple_scoped_prose_update` |
| `kpi-c-base-03` | P3 | C | baseline | `[]` | `simple_scoped_prose_update` |
| `kpi-c-opt-03` | P3 | C | optimized | `["heading:Context"]` | `simple_scoped_prose_update` |

### Pairing Rules

- Runs are paired by `(page_id, pattern, edit_intent_hash)`.
- Each pair has exactly one baseline and one optimized run.
- Same edit intent text within a pair; `new_value` distinguishes baseline from optimized.
- Alternate baseline/optimized order across pairs to reduce order bias.

### Write Manifest

**With auto-discovery**: Use `qa/manifests/kpi-revalidation-auto-discovery.example.json` as the template. Omit `target_path` from all entries. The pipeline auto-discovers valid text nodes within each heading section at runtime. Optionally set `target_index` to target a specific text node.

**Without auto-discovery** (legacy): Use `qa/manifests/kpi-revalidation-batch.json` with explicit `target_path` values. Replace `REPLACE_WITH_*` placeholders with real page IDs, target paths, and edit intent hashes from the page inventory. Run the `jq` path discovery step in Step 2b first.

Reference template: `qa/manifests/scoped-poc-experiment.example.json`.

### Execute Batch

```bash
cargo run -p atlassy-cli -- run-batch \
  --manifest qa/manifests/kpi-revalidation-batch.json \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

### Spot-Check Intermediate Results

After batch completion, verify a sample of run summaries:

```bash
# Check an optimized run for scoped metrics
jq '{success,publish_result,context_reduction_ratio,scoped_adf_bytes,full_page_adf_bytes,scope_resolution_failed}' \
  "artifacts/kpi-a-opt-01/summary.json"

# Check a baseline run for comparison
jq '{success,publish_result,context_reduction_ratio,scoped_adf_bytes,full_page_adf_bytes,scope_resolution_failed}' \
  "artifacts/kpi-a-base-01/summary.json"
```

Optimized runs should show `context_reduction_ratio > 0` and `scope_resolution_failed: false`. Baseline runs should show `context_reduction_ratio: 0` and `full_page_fetch: true`.

#### Auto-discovery spot-check

If using auto-discovery manifests (no `target_path` fields), verify the pipeline resolved targets correctly:

```bash
# Check discovered target for an optimized prose run
jq '{discovered_target_path,success,publish_result}' \
  "artifacts/kpi-a-opt-01/summary.json"

# Check discovered target for a table cell run
jq '{discovered_target_path,success,publish_result}' \
  "artifacts/kpi-b-opt-02/summary.json"
```

Pass criteria:

- `discovered_target_path` is non-null for all auto-discovery runs.
- Runs succeed and publish (not blocked by scope or schema errors).

### Evaluate Readiness

```bash
cargo run -p atlassy-cli -- run-readiness \
  --verify-replay \
  --artifacts-dir "$ARTIFACTS_DIR" || true
```

Notes:

- `run-readiness` returns non-zero unless recommendation is `go`.
- Decision output is written to `artifacts/batch/decision.packet.json`.

Review the decision packet:

```bash
jq '{recommendation,blocking_condition}' "artifacts/batch/decision.packet.json"
```

### KPI Gate Thresholds

| KPI | Threshold | Direction |
|-----|-----------|-----------|
| `context_reduction_ratio` | Optimized median >= 70% | Higher is better |
| `edit_success_rate` | > 95% for in-scope runs | Higher is better |
| `structural_preservation` | 100% for in-scope runs | Must be perfect |
| `conflict_rate` | < 10%, one scoped retry cap | Lower is better |
| `publish_latency` | Optimized median < 3000 ms, p90 non-regressive vs baseline | Lower is better |

Review per-pattern breakdown:

```bash
jq '.kpi_summary.pattern_rollups[] | {scope, metrics: [.metrics[] | {kpi, baseline: .baseline.median, optimized: .optimized.median, delta_relative}]}' \
  "artifacts/batch/report.json"
```

### Evidence Capture

1) Create evidence bundle directory:

```bash
mkdir -p qa/evidence/$(date -u +%F)-kpi-revalidation/runs
```

2) Copy run artifacts:

```bash
for RUN_DIR in artifacts/kpi-*/; do
  RUN_NAME=$(basename "$RUN_DIR")
  cp -r "$RUN_DIR" "qa/evidence/$(date -u +%F)-kpi-revalidation/runs/$RUN_NAME/"
done
```

3) Copy batch report and decision packet:

```bash
cp artifacts/batch/report.json "qa/evidence/$(date -u +%F)-kpi-revalidation/"
cp artifacts/batch/decision.packet.json "qa/evidence/$(date -u +%F)-kpi-revalidation/"
```

4) Write evidence bundle README following the established template (see `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/README.md` for format). Include:

- Provenance (`git_commit_sha`, `git_dirty`, `pipeline_version`, `runtime_mode`).
- All page IDs used.
- Per-run outcome summary.
- KPI gate results.

5) Write investigation report at `qa/investigations/$(date -u +%F)-kpi-revalidation.md` with:

- Provenance stamp.
- KPI results table (baseline vs optimized, delta, pass/fail per gate).
- Pattern-level breakdown (A, B, C).
- Recommendation: `go`, `iterate`, or `stop` with rationale.
- Follow-up items if any.

6) Update roadmap docs:

- Update `roadmap/04-kpi-and-experiments.md` checkpoint snapshot with new evidence reference.
- If `go`: update `roadmap/06-decisions-and-defaults.md` with measured outcomes.
- If `iterate`: document what needs fixing and next steps.

## Pass/Fail Checklist

### Single-page smoke (Steps 1-7)

- Preflight fails at verify and does not publish.
- Scoped-selector preflight run reports `scope_resolution_failed: false` and `full_page_fetch: false`.
- Prose smoke run publishes successfully in `live` mode.
- Optional table smoke run publishes successfully in `live` mode.
- Negative safety run fails with deterministic safety error code.
- Create-subpage returns JSON with `page_id` and `page_version: 1`.
- Bootstrap on empty page without flag fails with `ERR_BOOTSTRAP_REQUIRED` and `empty_page_detected: true`.
- Bootstrap on empty page with flag succeeds with `bootstrap_applied: true` and publishes scaffold.
- Bootstrap on non-empty page with flag fails with `ERR_BOOTSTRAP_INVALID_STATE` and `empty_page_detected: false`.
- All run summaries include `empty_page_detected` and `bootstrap_applied` fields.
- Artifacts exist for each run under `artifacts/<run_id>/`.

### KPI experiment (Step 8)

- All 3 experiment pages (P1, P2, P3) fetch successfully with expected route classifications.
- Scoped fetch spike on all 3 pages shows `scope_resolution_failed: false` and `context_reduction_ratio > 0`.
- No substring matching collisions observed during scoped fetch spike.
- KPI batch `report.json` contains a populated `kpi` section (not null).
- `context_reduction_ratio`: optimized median >= 70%.
- `edit_success_rate`: > 95% for in-scope runs.
- `structural_preservation`: 100% for in-scope runs (no locked-node mutation).
- `conflict_rate`: < 10% with no run exceeding one scoped retry.
- `publish_latency`: optimized median < 3000 ms, p90 non-regressive vs baseline.
- If using auto-discovery: `discovered_target_path` is non-null and within scope for all auto-discovery runs.
- Evidence bundle committed with clean provenance (`git_dirty: false`).
- Investigation report written with KPI results, pattern breakdown, and recommendation.

## Cleanup

```bash
unset ATLASSY_CONFLUENCE_API_TOKEN
unset SPACE_KEY
unset CREATED_PAGE_ID
```

Optional cleanup:

- Delete the created subpage via Confluence UI (it will remain as a child of your sandbox page).
- Delete local temporary manifest copy in `/tmp`.
- Delete `artifacts/` if you no longer need local run outputs.
