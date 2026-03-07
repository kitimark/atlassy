# Atlassy QA

This folder contains repeatable QA playbooks for live sandbox validation.

## Contents

- `qa/confluence-sandbox-test-plan.md`: detailed, step-by-step Confluence sandbox testing flow.
- `qa/manifests/live-sandbox-smoke.example.json`: example batch manifest for live smoke validation.
- `qa/manifests/scoped-poc-experiment.example.json`: example paired baseline/optimized manifest for revised scoped KPI evaluation.
- `qa/manifests/sandbox-page-inventory.md`: page ID inventory and structural characteristics for KPI experiment pages.
- `qa/manifests/kpi-revalidation-batch.json`: 18-run paired baseline/optimized manifest for KPI revalidation (patterns A, B, C).
- `qa/scripts/setup-confluence-env.sh`: interactive setup for live sandbox environment variables.
- `qa/investigations/`: timestamped investigation reports with evidence and provenance.

## Investigations

- Naming convention: `YYYY-MM-DD-<topic>.md`.
- Latest: `qa/investigations/2026-03-08-kpi-revalidation.md`.
- Prior: `qa/investigations/2026-03-07-lifecycle-subpage-bootstrap-validation.md`.
- Prior clean validation: `qa/investigations/2026-03-07-live-confluence-clean-validation.md`.
- Prior fix validation: `qa/investigations/2026-03-07-live-confluence-fix-validation.md`.
- Prior baseline: `qa/investigations/2026-03-07-live-confluence-failure.md`.
- Include provenance (`git_commit_sha`, `git_dirty`, runtime mode) and artifact paths for every major claim.

## Evidence

- Commit handoff-ready evidence bundles under `qa/evidence/YYYY-MM-DD-<topic>/`.
- Latest bundle: `qa/evidence/2026-03-08-kpi-revalidation/`.
- Prior bundle: `qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/`.
- Prior clean validation bundle: `qa/evidence/2026-03-07-live-confluence-clean-validation/`.
- Prior fix validation bundle: `qa/evidence/2026-03-07-live-confluence-fix-validation/`.
- Prior baseline bundle: `qa/evidence/2026-03-07-live-confluence-failure/`.
- Keep root `artifacts/` as local scratch; use committed `qa/evidence/` paths in investigation docs.

## Safety Defaults

- Use live runtime only against a dedicated sandbox page or sandbox space.
- Keep Confluence credentials in shell environment variables only.
- Never commit secrets, page tokens, or tenant-specific credentials.
- `no-op` mode still reaches publish unless `--force-verify-fail` is set.
- Treat `artifacts/` as temporary runtime output, even though it is git-ignored.
- For revised KPI runs, validate selector quality first (`scope_resolution_failed=false`, `full_page_fetch=false`) before large batches.
