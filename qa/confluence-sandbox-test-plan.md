# Confluence Sandbox Test Plan

## Objective

Run Atlassy end-to-end against real Confluence sandbox pages with explicit safety controls, deterministic artifacts, and clear pass/fail checks.

## Scope

- Covers `run`, optional `run-batch`, and optional `run-readiness` in `live` runtime mode.
- Uses v1-safe operations only (scoped prose and table cell text updates).
- Focuses on sandbox validation, not production release sign-off.

## Preconditions

- You have a dedicated sandbox Confluence page ID.
- You have sandbox API credentials (email + API token).
- Rust toolchain is installed (`cargo` available).
- Optional but recommended: `jq` for fast artifact checks.

## Safety Rules

- Never paste API tokens in chat, docs, commits, or manifests.
- Keep secrets in environment variables only.
- Use unique `request_id` per run.
- Start with a no-write preflight before any live publish.
- Do not run these steps against production pages.

## Environment Setup

```bash
export ATLASSY_CONFLUENCE_BASE_URL="https://YOURDOMAIN.atlassian.net"
export ATLASSY_CONFLUENCE_EMAIL="you@example.com"
read -rsp "Confluence API token: " ATLASSY_CONFLUENCE_API_TOKEN; echo
export ATLASSY_CONFLUENCE_API_TOKEN

export ARTIFACTS_DIR="."
export PAGE_ID="REPLACE_WITH_SANDBOX_PAGE_ID"
```

Notes:

- Do not append `/wiki` to `ATLASSY_CONFLUENCE_BASE_URL`.
- Runtime reads credentials from env vars only.

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

## Step 6 (Optional): Batch and Readiness Replay

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
- Artifacts exist for each run under `artifacts/<run_id>/`.

## Cleanup

```bash
unset ATLASSY_CONFLUENCE_API_TOKEN
```

Optional cleanup:

- Delete local temporary manifest copy in `/tmp`.
- Delete `artifacts/` if you no longer need local run outputs.
