# 2026-03-07 Lifecycle Subpage and Bootstrap Evidence Bundle

This bundle captures live sandbox validation after the `lifecycle-subpage-and-bootstrap` implementation (create-subpage command, empty-page bootstrap detection, and lifecycle summary fields).

## Provenance

- `git_commit_sha`: `1d24e2df5099a8827b83b36a519e172bbabffd4c`
- `git_dirty`: `false` (all included run summaries)
- `runtime_mode`: `live`
- `pipeline_version`: `v1`
- Sandbox page id: `131207`
- Created subpage id: `98309`

## Included Runs

- `runs/live-preflight-001/`
  - No-write preflight (`no-op` + `--force-verify-fail`).
  - Expected deterministic verify failure with no publish.
  - Confirms `empty_page_detected` and `bootstrap_applied` fields present.

- `runs/live-prose-001/`
  - Scoped prose publish smoke run.
  - `success: true`, `publish_result: "published"`, `new_version: 5`.
  - `empty_page_detected: false`, `bootstrap_applied: false`.

- `runs/live-negative-prose-boundary-001/`
  - Negative prose boundary validation (target path ending in `/type`).
  - Expected deterministic schema failure (`ERR_SCHEMA_INVALID`) at `md_assist_edit`.

- `runs/live-bootstrap-required-001/`
  - Empty page (id `98309`) without `--bootstrap-empty-page` flag.
  - Expected `ERR_BOOTSTRAP_REQUIRED` at `fetch`.
  - `empty_page_detected: true`, `bootstrap_applied: false`.

- `runs/live-bootstrap-success-001/`
  - Empty page (id `98309`) with `--bootstrap-empty-page` flag.
  - `success: true`, `publish_result: "published"`, `new_version: 2`.
  - `empty_page_detected: true`, `bootstrap_applied: true`.

- `runs/live-bootstrap-invalid-001/`
  - Non-empty page (id `131207`) with `--bootstrap-empty-page` flag.
  - Expected `ERR_BOOTSTRAP_INVALID_STATE` at `fetch`.
  - `empty_page_detected: false`, `bootstrap_applied: false`.

## Create-Subpage Output

```json
{
  "page_id": "98309",
  "page_version": 1
}
```

## Notes

- This is the first evidence bundle containing `empty_page_detected` and `bootstrap_applied` summary fields.
- Prior bundles (pre-lifecycle at commit `a80edac`) lack these fields.
- Step 4 (table cell update) skipped — sandbox page has no table.
- Step 8 (batch/readiness replay) skipped — optional.
