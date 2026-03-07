# 2026-03-07 Lifecycle Subpage and Bootstrap Validation

## Summary

This investigation validates live sandbox behavior after implementing the `lifecycle-subpage-and-bootstrap` change (34 tasks across 7 groups). It is the first validation that covers `create-subpage`, empty-page bootstrap detection, and the new `empty_page_detected`/`bootstrap_applied` summary fields.

All 7 executed test plan steps pass. The four-path bootstrap detection matrix is fully validated against live Confluence.

## Provenance

- `git_commit_sha`: `1d24e2df5099a8827b83b36a519e172bbabffd4c`
- `git_dirty`: `false`
- Runtime mode: `live`
- Evidence bundle: `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/`
- Test plan: `qa/confluence-sandbox-test-plan.md`

## Run Timeline

| Run ID | Step | Purpose | Result | Evidence |
| --- | --- | --- | --- | --- |
| `live-preflight-001` | 1 | No-write preflight (`no-op` + `--force-verify-fail`) | Expected verify failure | `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/runs/live-preflight-001/summary.json` |
| `live-prose-001` | 3 | Scoped prose publish smoke | Success (`published`, `new_version: 5`) | `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/runs/live-prose-001/summary.json` |
| `live-negative-prose-boundary-001` | 5 | Negative prose boundary check | Expected `ERR_SCHEMA_INVALID` | `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/runs/live-negative-prose-boundary-001/summary.json` |
| (create-subpage stdout) | 6 | Create subpage under sandbox page | `page_id: "98309"`, `page_version: 1` | stdout captured in evidence README |
| `live-bootstrap-required-001` | 7a | Empty page without `--bootstrap-empty-page` | `ERR_BOOTSTRAP_REQUIRED` | `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/runs/live-bootstrap-required-001/summary.json` |
| `live-bootstrap-success-001` | 7b | Empty page with `--bootstrap-empty-page` | Success (`published`, `bootstrap_applied: true`) | `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/runs/live-bootstrap-success-001/summary.json` |
| `live-bootstrap-invalid-001` | 7c | Non-empty page with `--bootstrap-empty-page` | `ERR_BOOTSTRAP_INVALID_STATE` | `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/runs/live-bootstrap-invalid-001/summary.json` |

## Validation Checks

### Step 1: Preflight

- `failure_state`: `verify`
- `error_codes`: `["ERR_SCHEMA_INVALID"]`
- No publish result recorded.
- `empty_page_detected: false`, `bootstrap_applied: false` — lifecycle fields present.

### Step 3: Prose publish

- `success: true`
- `publish_result`: `published`
- `runtime_mode`: `live`
- `new_version`: `5`
- `empty_page_detected: false`, `bootstrap_applied: false`

### Step 5: Negative safety

- `success: false`
- `failure_state`: `md_assist_edit`
- `error_codes`: `["ERR_SCHEMA_INVALID"]`
- No publish reached.

### Step 6: Create subpage

- Exit code: `0`
- Output: `{"page_id": "98309", "page_version": 1}`
- Page verified as child of sandbox page `131207`.

### Step 7a: Bootstrap required detection

- `success: false`
- `failure_state`: `fetch`
- `error_codes`: `["ERR_BOOTSTRAP_REQUIRED"]`
- `empty_page_detected: true`, `bootstrap_applied: false`

### Step 7b: Bootstrap scaffold injection

- `success: true`
- `publish_result`: `published`
- `new_version`: `2`
- `empty_page_detected: true`, `bootstrap_applied: true`
- Scaffold ADF (heading + paragraph) injected and published to live Confluence.

### Step 7c: Bootstrap invalid state

- `success: false`
- `failure_state`: `fetch`
- `error_codes`: `["ERR_BOOTSTRAP_INVALID_STATE"]`
- `empty_page_detected: false`, `bootstrap_applied: false`

## Bootstrap Detection Matrix

| Page state | `--bootstrap-empty-page` flag | Expected | Actual | Pass |
| --- | --- | --- | --- | --- |
| Empty | absent | `ERR_BOOTSTRAP_REQUIRED` | `ERR_BOOTSTRAP_REQUIRED` | Yes |
| Empty | present | Success, scaffold published | Success, `bootstrap_applied: true` | Yes |
| Non-empty | present | `ERR_BOOTSTRAP_INVALID_STATE` | `ERR_BOOTSTRAP_INVALID_STATE` | Yes |
| Non-empty | absent | Normal pipeline flow | Normal flow (Steps 1, 3, 5 confirm) | Yes |

## Skipped Steps

- **Step 4** (table cell update): Sandbox page has no table — skipped.
- **Step 8** (batch/readiness replay): Optional — skipped.

## Outcome

All mandatory test plan steps pass. The `lifecycle-subpage-and-bootstrap` implementation is validated against live Confluence with decision-grade provenance (`git_dirty: false`). This evidence supersedes the prior bundles at commit `a80edac` which lacked the lifecycle summary fields.
