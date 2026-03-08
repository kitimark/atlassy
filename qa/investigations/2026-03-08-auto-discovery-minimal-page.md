# 2026-03-08 Auto-Discovery on Minimal Page Structure

## Summary

Investigated auto-discovery behavior on sandbox page 131207 which has minimal content: a single info panel containing one paragraph with text, plus an empty paragraph. No headings exist on the page.

Key findings:

- Auto-discovery with full page scope correctly identifies the panel's inner paragraph text as a valid prose target.
- Live prose update via auto-discovery succeeds — publishes without modifying the panel structure (`locked_node_mutation: false`).
- Scoped fetch testing (heading selector) is not possible on this page since no headings exist.
- This confirms the test plan's route classification guidance: "inner paragraphs [of panels] are still `editable_prose`."

## Provenance

- `git_commit_sha`: `e05f1935e7df1c152b689b77ec053b4c80c731eb`
- `git_dirty`: `false`
- Runtime mode: `live`

## Page Structure

```
doc
├── panel (info)
│   └── paragraph
│       └── text: "Sandbox prose update 2026-03-07T20:48:41Z"
└── paragraph (empty)
```

Node path index: 5 nodes. Only 1 text node at `/content/0/content/0/content/0`.

## Run Timeline

| Run ID | Purpose | Result |
| --- | --- | --- |
| `live-preflight-001` | No-write preflight (`no-op` + `--force-verify-fail`) | Expected verify failure. `full_page_fetch: true`, `scope_resolution_failed: true` (no selectors). |
| `investigate-autodiscover-fullpage` | Auto-discovery with `--force-verify-fail` | `discovered_target_path: "/content/0/content/0/content/0/text"`. Failed at verify (forced). Discovery itself succeeded. |
| `investigate-live-prose-panel` | Live prose update via auto-discovery | `success: true`, `publish_result: "published"`, `new_version: 7`. `locked_node_mutation: false`. |

## Analysis

### Why auto-discovery finds the panel's inner text

The `discover_target_path()` Prose route filter:

1. Excludes paths with table ancestry (`table`, `tableRow`, `tableCell`) — panel text has no table ancestors, passes.
2. Includes only paths with `EDITABLE_PROSE_TYPES` ancestry (`paragraph`, `heading`, etc.) — panel text has `paragraph` ancestor at `/content/0/content/0`, passes.

The filter does not check for panel/locked_structural ancestry. This is correct: the text inside a panel's paragraph IS editable. Changing the text value preserves the panel structure.

### Scoped fetch limitation

The page has no headings, so `--scope "heading:..."` selectors cannot resolve. This is not a bug — the page simply lacks the structure needed for scoped testing. Scoped fetch validation will be performed on experiment pages (P1, P2, P3) which have dedicated heading sections.

## Impact on Test Plan Execution

| Step | Status | Notes |
| --- | --- | --- |
| Step 1 (preflight) | PASS | Verify failure at correct state. |
| Step 2b (scoped fetch) | SKIP | No headings on page. Will test on experiment pages. |
| Step 2b (auto-discovery) | PASS | Discovery resolved correct target. |
| Step 3 (live prose) | PASS | Published via auto-discovery. |
| Step 4 (table cell) | SKIP | No tables on page. |
| Step 5 (negative safety) | PROCEED | Use discovered path for boundary check. |

## Conclusion

Auto-discovery handles minimal page structures correctly. The panel's inner paragraph text is a valid editable target. No code changes needed.
