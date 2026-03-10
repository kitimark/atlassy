## Context

The readiness pipeline loads evidence from three files under `artifacts/batch/` (`manifest.normalized.json`, `artifact-index.json`, `report.json`) plus per-run `summary.json` files. `load_readiness_evidence()` in `evidence.rs` assembles these into a `ReadinessEvidence` struct, which `evaluate_readiness_gates()` in `gates.rs` evaluates across 7 gates. The decision packet is then assembled and persisted. `verify_decision_packet_replay()` rebuilds the full chain from the same filesystem artifacts and asserts bitwise equality with the stored packet — any new evidence source must be loaded identically in both passes.

Gate 7 currently checks four lifecycle conditions: three by scanning `RunSummary` fields in `evidence.summaries` for specific error codes (`BootstrapRequired`, `BootstrapInvalidState`) and flags (`bootstrap_applied`, `empty_page_detected`), and one by reading `evidence.manifest.batch.lifecycle_create_subpage_validated`. In live operation, lifecycle runs happen outside the batch as sequential state-machine traversals, so the summary-scanning path finds nothing and Gate 7 fails.

Gate 6 uses a similar operator-declared pattern (`live_smoke` booleans in `BatchManifestMetadata`) but through a different mechanism — booleans baked into the manifest before the batch runs. There is no unified concept of "operator-attested evidence" in the codebase.

## Goals / Non-Goals

**Goals:**

- Introduce a general-purpose attestation mechanism that gates can query for external evidence.
- Unblock Gate 7 in live workflows without distorting the batch manifest model.
- Maintain full backward compatibility — existing manifests, fixtures, and test suites work without modification.
- Preserve `verify_decision_packet_replay()` determinism.
- Design the attestation structure so future attestation types (drift parity, security audits, MCP transport health) can be added without new plumbing.

**Non-Goals:**

- Migrate Gate 6 drift parity booleans from `BatchManifestMetadata` to the attestation layer.
- Remove `BatchManifestMetadata.lifecycle_create_subpage_validated`.
- Add attestation-producing commands to the CLI (the operator creates the file manually or via external tooling).
- Validate attestation claims against machine evidence (attestations are trusted declarations).

## Decisions

### D1: Claims structure — typed struct per attestation kind

**Decision**: Use a `claims` field typed as `serde_json::Value` on the general `Attestation` struct, with gate-specific deserialization into typed claim structs at evaluation time.

**Alternatives considered**:
- **`BTreeMap<String, bool>`**: Simple, fully generic. Rejected because it limits claims to booleans only and makes gate evaluation depend on string-key correctness with no compiler help. A typo in a claim key silently evaluates to `false` (missing key).
- **Rust enum with per-kind variants**: `Claims::Lifecycle(LifecycleClaims)`, `Claims::DriftParity(DriftParityClaims)`, etc. Rejected because adding a new attestation kind requires modifying the enum, adding serde tag handling, and the enum cannot be extended without code changes. This front-loads decisions about attestation kinds we haven't designed yet.
- **Fully generic `serde_json::Value` everywhere**: No typed structs at all, gates query JSON paths directly. Rejected because it pushes all type safety to runtime and makes gate logic fragile.

**Chosen approach**: The `Attestation` struct carries `claims: serde_json::Value`. Gate evaluation code deserializes the value into a typed struct specific to the attestation kind it expects. For Gate 7, this is `LifecycleClaims { bootstrap_required_failure: bool, bootstrap_success: bool, bootstrap_on_non_empty_failure: bool, create_subpage_validated: bool }`. If deserialization fails (wrong shape, missing fields), the attestation is treated as absent — the gate falls back to summary scanning. New attestation kinds add a new claims struct and a query in the relevant gate; no changes to the `Attestation` type itself.

**Rationale**: This balances extensibility (new attestation kinds don't change the container) with type safety (gates parse into typed structs with compiler-enforced field access). The `serde_json::Value` boundary is the extensibility seam; the typed claims struct is the safety seam.

### D2: Attestation file is optional with empty default

**Decision**: `load_readiness_evidence()` attempts to read `artifacts/batch/attestations.json`. If the file does not exist, `attestations` defaults to an empty `Attestations { schema_version: "v1", entries: vec![] }`. If the file exists but is malformed, evidence loading fails with an operator-facing error.

**Rationale**: This follows the existing pattern — `BatchManifestMetadata` fields like `lifecycle_create_subpage_validated` default to `false` when absent via `#[serde(default)]`. Making the file optional means existing workflows, test fixtures, and the current `batch_complete_manifest.json` path continue working without changes. Gates that check attestations simply find no matching entry and fall through to their existing evidence paths.

### D3: Attestation provenance is self-contained

**Decision**: Each `Attestation` entry carries its own `ProvenanceStamp`. `load_readiness_evidence()` validates that each attestation's provenance stamp is well-formed (`validate_provenance_stamp()`) but does not require it to match the batch provenance.

**Alternatives considered**:
- **Require batch provenance match**: Rejected because attestations record evidence from a different session (e.g., lifecycle runs done before the batch). Requiring a match would force the operator to rerun lifecycle tests at the same commit as the batch, defeating the purpose.
- **No provenance on attestations**: Rejected because it removes traceability. Attestations are trusted declarations — provenance at least records when and from what codebase state the declaration was made.

### D4: Attestation metadata includes `attested_by` but not `attested_at`

**Decision**: Each `Attestation` carries an `attested_by: String` field (operator role, e.g. `"qa_owner"`). No `attested_at` timestamp field.

**Rationale**: `attested_by` aligns with the existing readiness role model (`ReadinessOwnerRoles` in `types.rs`) and provides audit traceability for who made the assertion. A separate `attested_at` timestamp adds operator friction without value — the `ProvenanceStamp` already provides codebase-state traceability via `git_commit_sha`, and the file's filesystem metadata captures write time. Adding a manually-entered timestamp invites clock skew and copy-paste errors.

### D5: Gate 7 uses `||` composition for dual-source evaluation

**Decision**: Gate 7 checks attestation claims first. If a `lifecycle_validation` attestation is present and its `LifecycleClaims` deserializes successfully, the four claim booleans are used. Each claim is then `||`-composed with the existing summary-scanning logic. Gate 7 passes if all four conditions are met from either source or a combination of both.

```
has_bootstrap_required_failure = attestation.bootstrap_required_failure
    || summary_scan_bootstrap_required_failure

has_bootstrap_success = attestation.bootstrap_success
    || summary_scan_bootstrap_success

(etc.)
```

**Rationale**: This preserves full backward compatibility. The existing `batch_complete_manifest.json` test fixture passes Gate 7 via summary scanning alone. A live workflow with an attestation file passes via the attestation path. A mixed scenario (some evidence in summaries, some in attestation) also works. No existing behavior changes.

### D6: Attestation container structure

**Decision**: The `attestations.json` file has the following shape:

```json
{
  "schema_version": "v1",
  "entries": [
    {
      "attestation_id": "lifecycle_validation",
      "attested_by": "qa_owner",
      "provenance": {
        "git_commit_sha": "...",
        "git_dirty": false,
        "pipeline_version": "0.1.0"
      },
      "evidence_refs": [
        "qa/evidence/2026-03-07-lifecycle-subpage-bootstrap/README.md"
      ],
      "claims": {
        "bootstrap_required_failure": true,
        "bootstrap_success": true,
        "bootstrap_on_non_empty_failure": true,
        "create_subpage_validated": true
      }
    }
  ]
}
```

Rust types:

```rust
struct Attestations {
    schema_version: String,
    entries: Vec<Attestation>,
}

struct Attestation {
    attestation_id: String,
    attested_by: String,
    provenance: ProvenanceStamp,
    evidence_refs: Vec<String>,
    claims: serde_json::Value,
}
```

Gates query attestations by `attestation_id` and deserialize `claims` into their expected typed struct. Duplicate `attestation_id` entries are not validated — the first match wins (consistent with how gates find the first matching summary in the existing code).

## Risks / Trade-offs

- **[Trusted assertions]** Attestations are not machine-verified. An operator can assert `bootstrap_required_failure: true` without having run the test. **Mitigation**: This is the same trust model Gate 6 already uses for `live_smoke`. Provenance and `evidence_refs` provide audit traceability. The readiness decision packet is a recommendation, not an autonomous deployment trigger.
- **[`serde_json::Value` type boundary]** Claims are untyped at the container level, shifting type checking to gate evaluation time. A malformed claims object is only detected when a gate tries to deserialize it. **Mitigation**: Failed deserialization treats the attestation as absent (falls back to summary scanning), which is safe — it can't cause a false pass. Gate tests cover both valid and malformed claims.
- **[Replay determinism]** The attestation file must be present (or absent) consistently between the initial `run-readiness` and any subsequent `verify_decision_packet_replay()` call. If an operator adds or modifies the file between runs, replay fails. **Mitigation**: This is the same constraint all evidence files have. `verify_decision_packet_replay` is designed to detect exactly this kind of mutation.
- **[Operator authoring friction]** The operator must manually create `attestations.json`. There is no CLI command to generate or template it. **Mitigation**: The file format is small and documented. A future CLI command to scaffold attestation files is a natural follow-up but out of scope for this change.
