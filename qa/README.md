# Atlassy QA

This folder contains repeatable QA playbooks for live sandbox validation.

## Contents

- `qa/confluence-sandbox-test-plan.md`: detailed, step-by-step Confluence sandbox testing flow.
- `qa/manifests/live-sandbox-smoke.example.json`: example batch manifest for live smoke validation.
- `qa/manifests/scoped-poc-experiment.example.json`: example paired baseline/optimized manifest for revised scoped KPI evaluation.
- `qa/manifests/sandbox-page-inventory.md`: page ID inventory and structural characteristics for KPI experiment pages.
- `qa/manifests/kpi-revalidation-batch.json`: 18-run paired baseline/optimized manifest for KPI revalidation (patterns A, B, C) with explicit `target_path` fields.
- `qa/manifests/kpi-revalidation-auto-discovery.example.json`: same 18-run structure but with `target_path` omitted (requires `roadmap/14-target-path-auto-discovery.md` implementation).
- `qa/manifests/kpi-revalidation-v3-auto-discovery.json`: live v3 revalidation manifest using fresh page IDs and auto-discovery (`kpi-v3-*` run IDs).
- `qa/scripts/setup-confluence-env.sh`: interactive setup for live sandbox environment variables (run via `make qa-setup`).
- `qa/scripts/check-env.sh`: non-interactive environment and connectivity validation (run `make qa-check` before QA execution).
- `qa/.env.local`: local credentials file written by setup script (git-ignored, never commit).
- `qa/investigations/`: timestamped investigation reports with evidence and provenance.

## Investigations

- Naming convention: `YYYY-MM-DD-<topic>.md`.
- Latest: `qa/investigations/2026-03-08-kpi-revalidation-v3.md`.
- Prior: `qa/investigations/2026-03-08-kpi-revalidation-v2.md`.
- Prior: `qa/investigations/2026-03-08-kpi-revalidation.md`.
- Prior: `qa/investigations/2026-03-08-scoped-extract-prose-scope-miss.md`.
- Prior: `qa/investigations/2026-03-08-auto-discovery-minimal-page.md`.
- Prior: `qa/investigations/2026-03-07-lifecycle-subpage-bootstrap-validation.md`.
- Prior clean validation: `qa/investigations/2026-03-07-live-confluence-clean-validation.md`.
- Prior fix validation: `qa/investigations/2026-03-07-live-confluence-fix-validation.md`.
- Prior baseline: `qa/investigations/2026-03-07-live-confluence-failure.md`.
- Include provenance (`git_commit_sha`, `git_dirty`, runtime mode) and artifact paths for every major claim.

## Evidence

- Commit handoff-ready evidence bundles under `qa/evidence/YYYY-MM-DD-<topic>/`.
- Latest bundle: `qa/evidence/2026-03-08-kpi-revalidation-v3/`.
- Prior bundle: `qa/evidence/2026-03-08-kpi-revalidation-v2/`.
- Prior bundle: `qa/evidence/2026-03-08-kpi-revalidation/`.
- Prior bundle: `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/`.
- Prior clean validation bundle: `qa/evidence/2026-03-07-live-confluence-clean-validation/`.
- Prior fix validation bundle: `qa/evidence/2026-03-07-live-confluence-fix-validation/`.
- Prior baseline bundle: `qa/evidence/2026-03-07-live-confluence-failure/`.
- Keep root `artifacts/` as local scratch; use committed `qa/evidence/` paths in investigation docs.

## Safety Defaults

- Use live runtime only against a dedicated sandbox page or sandbox space.
- Keep Confluence credentials out of chat/docs/commits; manage them through `make qa-setup`, which writes local `qa/.env.local` (git-ignored).
- Never commit secrets, page tokens, or tenant-specific credentials.
- `no-op` mode still reaches publish unless `--force-verify-fail` is set.
- Treat `artifacts/` as temporary runtime output, even though it is git-ignored.
- For revised KPI runs, validate selector quality first (`scope_resolution_failed=false`, `full_page_fetch=false`) before large batches.
