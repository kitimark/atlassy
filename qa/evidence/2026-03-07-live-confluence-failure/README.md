# 2026-03-07 Live Confluence Failure Evidence Bundle

This bundle contains the committed run artifacts referenced by `qa/investigations/2026-03-07-live-confluence-failure.md`.

## Provenance

- `git_commit_sha`: `4a21d6d630ba990fb58c1a86e99b7d3bdf87a1a7`
- `git_dirty` at test time: `true`
- `runtime_mode`: `live`
- `pipeline_version` (from summaries): `v1`
- Sandbox page id in captured runs: `131207`

## Included Runs

- `runs/live-qa-preflight-20260307T0845Z/`: forced verify-fail preflight (expected no publish).
- `runs/live-qa-prose-20260307T0845Z/`: scoped prose publish failure (`ERR_RUNTIME_BACKEND`, http 400 message).
- `runs/live-qa-negative-20260307T0850Z/`: expected negative boundary failure (`ERR_SCHEMA_INVALID`).
- `runs/live-qa-prose-20260307T0850Z/`: appendix diagnostic run showing successful publish after local-only experiment.

## Notes

- This folder is intentionally committed under `qa/evidence/` for teammate handoff and reproducible investigation context.
- Root `artifacts/` remains a local scratch area and is intentionally git-ignored.
- The related local diagnostic source edits were reverted and not committed.
