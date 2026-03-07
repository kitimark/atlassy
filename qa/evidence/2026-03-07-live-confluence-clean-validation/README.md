# 2026-03-07 Live Confluence Clean Validation Evidence Bundle

This bundle captures a clean-working-tree live sandbox validation run after archiving `fix-live-confluence-qa-failures`.

## Provenance

- `git_commit_sha`: `a80edacc0b6e921c15dcd535d56390b25d075953`
- `git_dirty`: `false` (all included run summaries)
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Sandbox page id: `131207`

## Included Runs

- `runs/live-clean-preflight-20260307T095210Z/`
  - No-write preflight (`no-op` + `--force-verify-fail`).
  - Expected deterministic verify failure with no publish.

- `runs/live-clean-prose-20260307T095210Z/`
  - Scoped prose publish smoke run.
  - Expected success with `publish_result: "published"` and incremented version.

- `runs/live-clean-negative-20260307T095210Z/`
  - Negative prose boundary validation.
  - Expected deterministic schema failure (`ERR_SCHEMA_INVALID`).

## Notes

- This bundle is committed for handoff-ready validation evidence.
- Root `artifacts/` remains local scratch output.
