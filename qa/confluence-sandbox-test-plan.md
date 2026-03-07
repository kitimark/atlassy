# Confluence Sandbox Test Plan

## Objective

Run Atlassy end-to-end against real Confluence sandbox pages with explicit safety controls, deterministic artifacts, and clear pass/fail checks.

## Scope

- Covers `run`, `create-subpage`, optional `run-batch`, and optional `run-readiness` in `live` runtime mode.
- Uses v1-safe operations only (scoped prose and table cell text updates, subpage creation, and empty-page bootstrap).
- Focuses on sandbox validation, not production release sign-off.

Latest investigation:

- `qa/investigations/2026-03-07-lifecycle-subpage-bootstrap-validation.md`

## Preconditions

- You have a dedicated sandbox Confluence page ID.
- You have sandbox API credentials (email + API token).
- Rust toolchain is installed (`cargo` available).
- Optional but recommended: `jq` for fast artifact checks.
- You know the space key for your sandbox page (visible in the space URL, e.g., `~username` for personal spaces or a short key like `ENG`).

## Safety Rules

- Never paste API tokens in chat, docs, commits, or manifests.
- Keep secrets in environment variables only.
- Use unique `request_id` per run.
- Start with a no-write preflight before any live publish.
- Do not run these steps against production pages.

## Environment Setup

```bash
source qa/scripts/setup-confluence-env.sh
```

Notes:

- The setup script prompts for `ATLASSY_CONFLUENCE_BASE_URL`, `ATLASSY_CONFLUENCE_EMAIL`, `ATLASSY_CONFLUENCE_API_TOKEN`, `ARTIFACTS_DIR`, optional `PAGE_ID`, and optional `SPACE_KEY`.
- You must `source` the script (do not execute it) so exports persist in the current shell.
- Do not append `/wiki` to `ATLASSY_CONFLUENCE_BASE_URL`.
- Runtime reads credentials from env vars only.

Manual fallback (if you do not use the script):

```bash
export ATLASSY_CONFLUENCE_BASE_URL="https://YOURDOMAIN.atlassian.net"
export ATLASSY_CONFLUENCE_EMAIL="you@example.com"
read -rs "ATLASSY_CONFLUENCE_API_TOKEN?Confluence API token: "; print; export ATLASSY_CONFLUENCE_API_TOKEN
export ARTIFACTS_DIR="."
export PAGE_ID="REPLACE_WITH_SANDBOX_PAGE_ID"
export SPACE_KEY="REPLACE_WITH_SPACE_KEY"
```

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

## Step 8 (Optional): Batch and Readiness Replay

1) Copy and edit manifest template:

```bash
cp qa/manifests/live-sandbox-smoke.example.json /tmp/live-sandbox-smoke.json
```

2) Run live batch:

```bash
cargo run -p atlassy-cli -- run-batch \
  --manifest /tmp/live-sandbox-smoke.json \
  --runtime-backend live \
  --artifacts-dir "$ARTIFACTS_DIR"
```

3) Run readiness replay verification:

```bash
cargo run -p atlassy-cli -- run-readiness \
  --verify-replay \
  --artifacts-dir "$ARTIFACTS_DIR" || true
```

Notes:

- `run-readiness` returns non-zero unless recommendation is `go`.
- Decision output is written to `artifacts/batch/decision.packet.json`.

Check:

```bash
jq '{recommendation,blocking_condition}' "artifacts/batch/decision.packet.json"
```

## Pass/Fail Checklist

- Preflight fails at verify and does not publish.
- Prose smoke run publishes successfully in `live` mode.
- Optional table smoke run publishes successfully in `live` mode.
- Negative safety run fails with deterministic safety error code.
- Create-subpage returns JSON with `page_id` and `page_version: 1`.
- Bootstrap on empty page without flag fails with `ERR_BOOTSTRAP_REQUIRED` and `empty_page_detected: true`.
- Bootstrap on empty page with flag succeeds with `bootstrap_applied: true` and publishes scaffold.
- Bootstrap on non-empty page with flag fails with `ERR_BOOTSTRAP_INVALID_STATE` and `empty_page_detected: false`.
- All run summaries include `empty_page_detected` and `bootstrap_applied` fields.
- Artifacts exist for each run under `artifacts/<run_id>/`.

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
