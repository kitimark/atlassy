## 1. Runtime Bootstrap Hardening

- [ ] 1.1 Reproduce and codify the live startup panic path in a regression test harness.
- [ ] 1.2 Update live runtime startup flow so backend initialization failures return deterministic mapped errors instead of process panic.
- [ ] 1.3 Verify operator-facing failure output for startup failures remains deterministic and compatible with existing run/report handling.

## 2. Live Publish Contract Alignment

- [ ] 2.1 Implement explicit live publish payload contract assembly for Confluence page updates (version metadata and `atlas_doc_format` fields).
- [ ] 2.2 Add tests that assert payload contract details, including version increment and ADF value encoding expectations.
- [ ] 2.3 Ensure Confluence payload-contract rejections (`400`) map to deterministic runtime backend errors with `publish` failure state.

## 3. Regression Validation

- [ ] 3.1 Run and update affected crate tests (`atlassy-confluence`, `atlassy-cli`) to cover startup and publish failure/success paths.
- [ ] 3.2 Confirm existing live retry and hard-error taxonomy behavior remains intact after changes.
- [ ] 3.3 Execute a local live preflight failure-path check to verify no runtime panic occurs.

## 4. QA Evidence Refresh

- [ ] 4.1 Run sandbox smoke sequence from `qa/confluence-sandbox-test-plan.md` (preflight, prose publish, negative safety).
- [ ] 4.2 Capture post-fix evidence bundle under `qa/evidence/` and add follow-up investigation summary in `qa/investigations/`.
- [ ] 4.3 Update QA index pointers to include the latest post-fix investigation/evidence artifacts.
