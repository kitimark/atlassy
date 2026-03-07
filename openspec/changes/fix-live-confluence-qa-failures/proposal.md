## Why

Live sandbox QA uncovered two blocking failures in `live` runtime mode: process-level runtime panic before deterministic error handling, and Confluence publish rejection (`400 Must supply an incremented version... No version`) despite successful upstream pipeline states. These failures prevent reliable live validation and keep readiness evidence from representing real Confluence behavior.

## What Changes

- Harden live runtime startup so backend initialization failures never crash the process and always map into deterministic runtime error taxonomy.
- Define and enforce the Confluence publish request contract for ADF updates so live scoped prose publish succeeds when upstream states pass verification.
- Expand live QA acceptance coverage to require: no runtime panic, deterministic failure mapping, and successful publish path in sandbox smoke.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `live-confluence-runtime-selection`: strengthen requirements for runtime bootstrap behavior and Confluence publish contract compliance in `live` mode.

## Impact

- Affected code: `crates/atlassy-cli` runtime entry and live path invocation, `crates/atlassy-confluence` live client publish path.
- Affected tests: live runtime regression coverage and deterministic error-mapping expectations.
- Operational impact: enables reproducible live sandbox QA results and unblocks follow-up readiness evaluation with real backend behavior.
