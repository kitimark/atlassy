# 2026-03-07 Live Confluence Clean Validation

## Summary

This investigation validates live sandbox behavior from a clean commit state after the runtime and publish-contract fixes.

- No-write preflight fails at `verify` as expected.
- Scoped prose publish succeeds in `live` mode.
- Negative prose boundary validation fails deterministically with `ERR_SCHEMA_INVALID`.
- All included summaries report `git_dirty: false`.

## Provenance

- `git_commit_sha`: `a80edacc0b6e921c15dcd535d56390b25d075953`
- `git_dirty`: `false`
- Runtime mode: `live`
- Evidence bundle: `qa/evidence/2026-03-07-live-confluence-clean-validation/`

## Run Timeline

| Run ID | Purpose | Result | Evidence |
| --- | --- | --- | --- |
| `live-clean-preflight-20260307T095210Z` | No-write preflight (`no-op` + `--force-verify-fail`) | Expected verify failure | `qa/evidence/2026-03-07-live-confluence-clean-validation/runs/live-clean-preflight-20260307T095210Z/summary.json` |
| `live-clean-prose-20260307T095210Z` | Scoped prose publish smoke | Success (`publish_result: "published"`, `new_version: 4`) | `qa/evidence/2026-03-07-live-confluence-clean-validation/runs/live-clean-prose-20260307T095210Z/summary.json` |
| `live-clean-negative-20260307T095210Z` | Negative prose boundary check | Expected deterministic schema failure | `qa/evidence/2026-03-07-live-confluence-clean-validation/runs/live-clean-negative-20260307T095210Z/summary.json` |

## Validation Checks

### Preflight check

- `failure_state` is `verify`.
- `error_codes` contains `ERR_SCHEMA_INVALID`.
- No publish result is recorded.

### Publish check

- `success` is `true`.
- `publish_result` is `published`.
- `runtime_mode` is `live`.

### Negative safety check

- `success` is `false`.
- `failure_state` is `md_assist_edit`.
- `error_codes` contains `ERR_SCHEMA_INVALID`.

## Outcome

Live sandbox smoke behavior is consistent and decision-grade provenance is clean (`git_dirty: false`) for this validation set.
