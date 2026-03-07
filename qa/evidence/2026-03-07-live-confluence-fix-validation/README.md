# 2026-03-07 Live Confluence Fix Validation Evidence Bundle

This bundle captures post-fix validation runs for change `fix-live-confluence-qa-failures`.

## Provenance

- `git_commit_sha` in captured summaries: `b924eaf369b5084beaa7f2cc1991812faf893b4e`
- `git_dirty` at run time: `true`
- `runtime_mode`: `live`
- `pipeline_version`: `v1`

## Included Runs

- `runs/live-preflight-local-failure-check/`
  - Local failure-path check using unreachable runtime base URL.
  - Expected deterministic hard error at `fetch` with `ERR_RUNTIME_BACKEND`.
  - Confirms no Tokio runtime drop panic in operator output.

- `runs/live-postfix-preflight-20260307T093543Z/`
  - Sandbox no-write preflight (`--force-verify-fail`).
  - Expected verify failure with no publish.

- `runs/live-postfix-prose-20260307T093543Z/`
  - Sandbox scoped prose publish smoke.
  - Expected success with `publish_result: "published"`.

- `runs/live-postfix-negative-20260307T093543Z/`
  - Sandbox negative prose boundary validation.
  - Expected deterministic schema failure (`ERR_SCHEMA_INVALID`).

## Notes

- These artifacts are committed for reproducible team handoff and follow-up readiness discussions.
- Root `artifacts/` remains local scratch output for ad hoc runs.
