# 2026-03-07 Live Confluence Failure Investigation

## Summary

This investigation records why live Confluence sandbox execution failed and what evidence was captured.

- Goal: run QA smoke in live sandbox mode without committing implementation changes.
- Outcome: identified two distinct live-mode failure causes.
- Source code status: local diagnostic code changes were reverted and not committed.

## Provenance

- Baseline commit under test: `4a21d6d630ba990fb58c1a86e99b7d3bdf87a1a7`.
- Runtime mode: `live`.
- Sandbox page id: `131207`.
- Evidence bundle root: `qa/evidence/2026-03-07-live-confluence-failure/`.
- Test-time summaries report `git_dirty: true` (see run summaries below).

## Run Timeline

| Run ID | Purpose | Result | Key Evidence |
| --- | --- | --- | --- |
| `live-preflight-20260307T084059Z` | First live preflight on as-is implementation | Unexpected runtime panic before artifacts | CLI stderr panic: `Cannot drop a runtime in a context where blocking is not allowed` |
| `live-preflight-backtrace` | Re-run with `RUST_BACKTRACE=1` | Panic reproduced with stack trace | Backtrace points to blocking client init in live client |
| `live-qa-preflight-20260307T0845Z` | No-write preflight (`--force-verify-fail`) | Expected failure at verify | `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-preflight-20260307T0845Z/summary.json` |
| `live-qa-prose-20260307T0845Z` | Scoped prose publish smoke | Unexpected publish failure | `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-prose-20260307T0845Z/summary.json` |
| `live-qa-negative-20260307T0850Z` | Negative prose boundary check | Expected schema failure | `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-negative-20260307T0850Z/summary.json` |

## Detailed Findings

### Finding 1: Runtime bootstrap panic in live mode

Observed on initial as-is run:

- Panic message: `Cannot drop a runtime in a context where blocking is not allowed`.
- Backtrace highlights:
  - `reqwest::blocking::client::Client::new`
  - `atlassy_confluence::LiveConfluenceClient::new` in `crates/atlassy-confluence/src/lib.rs`
  - `atlassy_cli::run_single_request` and async `main` in `crates/atlassy-cli/src/main.rs`

Assessment:

- Live mode initializes a blocking HTTP client from an async Tokio context.
- This is a runtime lifecycle mismatch and can terminate before pipeline execution completes.

### Finding 2: Publish contract failure against Confluence

Observed on scoped prose run after bypassing the runtime panic:

- Summary shows `failure_state: "publish"` with `ERR_RUNTIME_BACKEND`.
- Confluence error snippet from CLI output:
  - `http_status=400`
  - `Must supply an incremented version when updating Content. No version`

Evidence that earlier states were healthy for this run:

- Fetch returned `page_version: 1` in `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-prose-20260307T0845Z/fetch/state_output.json`.
- Patch generated candidate ADF in `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-prose-20260307T0845Z/patch/state_output.json`.
- Verify passed in `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-prose-20260307T0845Z/verify/state_output.json`.

Assessment:

- The failure is isolated to live publish request/contract handling, not scope resolution, patch generation, or verification.

## Other Observations

- Preflight behaved as expected with forced verify failure:
  - `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-preflight-20260307T0845Z/summary.json` reports verify failure and no publish result.
- Negative prose boundary test behaved as expected:
  - `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-negative-20260307T0850Z/summary.json` reports `ERR_SCHEMA_INVALID`.

## Deferred Fix Plan (No Code Changes In This Investigation)

1. Align runtime model for live execution:
   - either run blocking client in synchronous CLI flow,
   - or migrate live client to async reqwest and avoid blocking client lifecycle under Tokio async main.
2. Validate Confluence publish payload contract with explicit sandbox check:
   - capture outbound payload shape,
   - compare with Confluence API expectation for `PUT /wiki/rest/api/content/{id}`.
3. Add a regression smoke path for live mode:
   - no-write preflight,
   - one scoped prose publish,
   - one deterministic negative safety run.

## Appendix: Non-Committed Diagnostic Experiment

The following local-only diagnostic edits were temporarily applied to test hypotheses and were not committed:

- Temporary change A: made CLI entrypoint synchronous to avoid async + blocking runtime conflict.
- Temporary change B: adjusted live publish payload shape for `atlas_doc_format` update handling.

Observed behavior after temporary diagnostics:

- `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-prose-20260307T0850Z/summary.json` reports successful publish (`publish_result: "published"`, `new_version: 2`).
- `qa/evidence/2026-03-07-live-confluence-failure/runs/live-qa-negative-20260307T0850Z/summary.json` still reports expected deterministic schema failure.

Reproducibility note:

- Diagnostic edits were reverted with `git restore`.
- Repository source code remains at baseline implementation for follow-up fix work.
