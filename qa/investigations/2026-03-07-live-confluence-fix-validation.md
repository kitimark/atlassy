# 2026-03-07 Live Confluence Fix Validation

## Summary

This follow-up investigation validates remediation work for the failures documented in `qa/investigations/2026-03-07-live-confluence-failure.md`.

- Startup path no longer panics under live runtime initialization conditions.
- Live scoped prose publish succeeds in sandbox with `publish_result: "published"`.
- Negative prose boundary checks still fail deterministically with `ERR_SCHEMA_INVALID`.

## Provenance

- Baseline change under validation: `fix-live-confluence-qa-failures`
- `git_commit_sha` in summaries: `b924eaf369b5084beaa7f2cc1991812faf893b4e`
- `git_dirty` at test time: `true`
- Runtime mode: `live`
- Evidence bundle: `qa/evidence/2026-03-07-live-confluence-fix-validation/`

## Run Timeline

| Run ID | Purpose | Result | Key Evidence |
| --- | --- | --- | --- |
| `live-preflight-local-failure-check` | Local failure-path regression (unreachable base URL) | Expected hard failure at `fetch` with deterministic runtime code; no panic | `qa/evidence/2026-03-07-live-confluence-fix-validation/runs/live-preflight-local-failure-check/summary.json` |
| `live-postfix-preflight-20260307T093543Z` | Sandbox no-write preflight (`--force-verify-fail`) | Expected verify failure, publish not reached | `qa/evidence/2026-03-07-live-confluence-fix-validation/runs/live-postfix-preflight-20260307T093543Z/summary.json` |
| `live-postfix-prose-20260307T093543Z` | Sandbox scoped prose publish smoke | Success (`publish_result: "published"`, `new_version: 3`) | `qa/evidence/2026-03-07-live-confluence-fix-validation/runs/live-postfix-prose-20260307T093543Z/summary.json` |
| `live-postfix-negative-20260307T093543Z` | Sandbox negative prose boundary validation | Expected schema failure (`ERR_SCHEMA_INVALID`) | `qa/evidence/2026-03-07-live-confluence-fix-validation/runs/live-postfix-negative-20260307T093543Z/summary.json` |

## Key Checks

### Startup panic regression check

- Local failure-path run reports `failure_state: "fetch"` with `ERR_RUNTIME_BACKEND`.
- Operator-facing failure remains deterministic (`pipeline hard error`) without Tokio runtime-drop panic output.

### Publish contract validation

- Sandbox prose run reached `publish` and succeeded.
- Summary confirms non-empty publish token metrics and version increment to `3`.

### Safety invariants retained

- Negative prose boundary check still fails at `md_assist_edit` with `ERR_SCHEMA_INVALID`.
- Confirms hardening did not weaken route/boundary safety behavior.

## Outcome

The previously observed live-runtime blockers are remediated for sandbox QA workflows:

1. Runtime startup no longer terminates with async/blocking panic behavior.
2. Live publish contract path is valid for scoped prose updates.

Remaining follow-up for sign-off:

- Re-run validation from a clean commit state (`git_dirty: false`) to finalize decision-grade provenance.
