# Atlassy QA

This folder contains repeatable QA playbooks for live sandbox validation.

## Contents

- `qa/confluence-sandbox-test-plan.md`: detailed, step-by-step Confluence sandbox testing flow.
- `qa/manifests/live-sandbox-smoke.example.json`: example batch manifest for live smoke validation.
- `qa/scripts/setup-confluence-env.sh`: interactive setup for live sandbox environment variables.

## Safety Defaults

- Use live runtime only against a dedicated sandbox page or sandbox space.
- Keep Confluence credentials in shell environment variables only.
- Never commit secrets, page tokens, or tenant-specific credentials.
- `no-op` mode still reaches publish unless `--force-verify-fail` is set.
- Treat `artifacts/` as temporary runtime output, even though it is git-ignored.
