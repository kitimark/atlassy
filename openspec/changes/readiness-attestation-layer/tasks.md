## 1. Attestation types

- [ ] 1.1 Add `Attestations` struct to `crates/atlassy-cli/src/types.rs` with `schema_version: String` and `entries: Vec<Attestation>`, deriving `Debug`, `Clone`, `Deserialize`, `Serialize`, `PartialEq`. Implement `Default` returning `schema_version: "v1"` and empty `entries`.
- [ ] 1.2 Add `Attestation` struct to `types.rs` with fields `attestation_id: String`, `attested_by: String`, `provenance: ProvenanceStamp`, `evidence_refs: Vec<String>`, `claims: serde_json::Value`. Derive `Debug`, `Clone`, `Deserialize`, `Serialize`, `PartialEq`.
- [ ] 1.3 Add `LifecycleClaims` struct to `types.rs` with four `bool` fields: `bootstrap_required_failure`, `bootstrap_success`, `bootstrap_on_non_empty_failure`, `create_subpage_validated`. Derive `Debug`, `Clone`, `Deserialize`, `Serialize`, `PartialEq`.
- [ ] 1.4 Add `attestations: Attestations` field to `ReadinessEvidence` struct in `types.rs`.
- [ ] 1.5 Add `serde_json` to `[dependencies]` in `crates/atlassy-cli/Cargo.toml` if not already present (needed for `serde_json::Value` on `Attestation.claims`).

## 2. Evidence loading

- [ ] 2.1 In `crates/atlassy-cli/src/readiness/evidence.rs`, after loading manifest/artifact-index/report, add attestation loading: attempt to read `artifacts/batch/attestations.json`. If file does not exist, default to `Attestations::default()`. If file exists but is malformed, return operator-facing error.
- [ ] 2.2 When attestation file is present and loaded, append `"artifacts/batch/attestations.json"` to `source_artifacts` before sorting. When absent, do not add it.
- [ ] 2.3 Set the `attestations` field on the returned `ReadinessEvidence` struct.
- [ ] 2.4 Add provenance validation for each attestation entry: call `validate_provenance_stamp(&entry.provenance)` during loading. Fail with operator-facing error if any attestation has malformed provenance.

## 3. Gate 7 dual-source evaluation

- [ ] 3.1 In `crates/atlassy-cli/src/readiness/gates.rs`, add a helper to extract `LifecycleClaims` from attestations: find the first entry where `attestation_id == "lifecycle_validation"`, attempt `serde_json::from_value::<LifecycleClaims>(entry.claims.clone())`. Return `None` if not found or deserialization fails.
- [ ] 3.2 Revise Gate 7 evaluation to use `||` composition: each of the four lifecycle conditions checks the attestation claims (if present) OR the existing summary-scanning logic.
- [ ] 3.3 Update Gate 7 `evidence_refs` to include `"artifacts/batch/attestations.json"` when attestation evidence is present and contributes to the gate outcome.

## 4. Test fixture

- [ ] 4.1 Create `crates/atlassy-cli/tests/fixtures/attestations_lifecycle_complete.json` with a valid `lifecycle_validation` attestation entry (all four claims `true`, valid provenance, evidence refs pointing to `qa/evidence/` paths).

## 5. Tests

- [ ] 5.1 Add test in `crates/atlassy-cli/tests/readiness.rs`: Gate 7 passes via attestation alone. Use `batch_coverage_failure_manifest.json` (no lifecycle runs in manifest), place `attestations_lifecycle_complete.json` as `artifacts/batch/attestations.json` in the temp dir. Assert Gate 7 passes.
- [ ] 5.2 Add test: Gate 7 fails with malformed attestation claims. Place an attestation file with `lifecycle_validation` entry but invalid claims shape (e.g., missing fields). Assert Gate 7 fails (falls back to summary scanning, finds nothing).
- [ ] 5.3 Add test: existing `batch_complete_manifest.json` test continues passing without any attestation file (backward compatibility).
- [ ] 5.4 Add test: replay verification works with attestation file present. Run batch + attestation, generate readiness, verify `verify_decision_packet_replay()` succeeds.
- [ ] 5.5 Run `cargo test --workspace` and confirm all existing tests pass with no regressions.

## 6. Verification

- [ ] 6.1 Run `cargo clippy --workspace -- -D warnings` and resolve any warnings.
- [ ] 6.2 Run `cargo fmt --all --check` and confirm formatting is clean.
