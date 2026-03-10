## Why

The v1 readiness pipeline assumes all gate evidence flows through the batch manifest and its run summaries. Gates 1–5 fit this model — they evaluate machine-produced pipeline outputs. Gate 7 (Lifecycle Enablement Validation) does not: three of its four checks scan `RunSummary` fields for bootstrap error codes, but lifecycle validation runs are inherently sequential state-machine traversals (create page → bootstrap-required fail → bootstrap success → bootstrap-invalid-state fail) that were executed outside the batch. The batch model treats runs as independent and paired; lifecycle runs are neither. Gate 6 already accepts operator-declared input (`live_smoke` booleans in `BatchManifestMetadata`), but this pattern is implicit and inconsistent — Gate 6 reads manifest booleans while Gate 7 scans summaries for evidence that isn't there. This change introduces a general-purpose attestation layer so gates that require external evidence have a consistent, provenance-stamped, extensible source — starting with Gate 7 lifecycle evidence while leaving Gate 6 untouched.

## What Changes

- Introduce an `Attestations` container and `Attestation` entry type in the CLI types module. Each attestation carries an identifier, provenance stamp, evidence references, and structured claims. The claims representation (generic map vs typed struct per attestation kind) is deferred to design.
- Add `attestations` as a new field on `ReadinessEvidence`, loaded from an optional `artifacts/batch/attestations.json` file by `load_readiness_evidence()`. When the file is absent, attestations default to empty — no breaking change to existing workflows.
- Revise Gate 7 evaluation in `evaluate_readiness_gates()` to accept lifecycle evidence from either the attestation layer (new path) or batch run summaries (existing path). Gate 7 passes if either source provides complete lifecycle matrix coverage.
- Attestation provenance is self-contained — it records the git SHA and pipeline version of the lifecycle validation session, not the batch session. `load_readiness_evidence()` does not require attestation provenance to match batch provenance.
- Attestation loading is deterministic for `verify_decision_packet_replay()` — the file is read from a fixed path and produces identical results on rebuild.
- Gate 6 and `BatchManifestMetadata.live_smoke` are unchanged. Migration of drift parity to the attestation layer is a future optional cleanup, not part of this change.
- `BatchManifestMetadata.lifecycle_create_subpage_validated` remains but becomes redundant when attestation evidence is present. No removal in this change.

## Capabilities

### New Capabilities
- `readiness-attestation-evidence`: Defines the attestation file format, entry structure, evidence loading behavior, provenance policy, and backward-compatible defaulting when the attestation file is absent.

### Modified Capabilities
- `readiness-gate-checklist`: Gate 7 evaluation accepts lifecycle evidence from the attestation layer as an alternative to batch run summary scanning. Gate 7 pass criteria, failure messaging, and blocking behavior are unchanged — only the evidence source is extended.
- `decision-packet-governance`: Decision packet replay verification must account for attestation evidence as a deterministic input. Attestation file path is included in `source_artifacts` when present.

## Impact

- **Code**: `crates/atlassy-cli/src/types.rs` (new attestation types), `crates/atlassy-cli/src/readiness/evidence.rs` (attestation loading), `crates/atlassy-cli/src/readiness/gates.rs` (Gate 7 dual-source evaluation). Test files: `tests/readiness.rs` (new test for attestation-based Gate 7 pass), test fixtures (new `attestations.json` fixture).
- **Wire format**: New `attestations.json` file under `artifacts/batch/`. Existing files (`manifest.normalized.json`, `report.json`, `artifact-index.json`) are unchanged.
- **Dependencies**: No crate dependency changes. No changes to `atlassy-contracts`, `atlassy-pipeline`, `atlassy-adf`, or `atlassy-confluence`.
- **Backward compatibility**: Fully backward compatible. Missing attestation file defaults to empty attestations. Existing batch manifests with lifecycle runs in the manifest continue to work via the existing summary-scanning path.
