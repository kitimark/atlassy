## MODIFIED Requirements

### Requirement: Unknown failure classes are handled safely
If diagnostics include a failure class not covered by a specific runbook section, the system MUST still produce a safe output. With a closed `ErrorClass` enum, unmapped failure classes are prevented at compile time — the runbook builder MUST use exhaustive `match` on `ErrorClass` so that adding a new variant without handling it is a compiler error. The `matches!()` known-class guard and `unknown:{class}` fallback pattern MUST be removed. Error classes without specific runbook sections (`Io`, `ProvenanceIncomplete`, `RuntimeUnmappedHard`) MUST be handled as explicit no-op match arms, not via wildcard.

#### Scenario: Compiler enforces exhaustive class handling
- **WHEN** a new variant is added to the `ErrorClass` enum
- **THEN** the runbook builder MUST fail to compile until the new variant is handled in the match expression

#### Scenario: Classes without runbook sections are explicit
- **WHEN** an error class has no specific runbook triage steps (e.g., `Io`, `ProvenanceIncomplete`, `RuntimeUnmappedHard`)
- **THEN** the match arm MUST exist as an explicit no-op, not be covered by a wildcard or omitted

#### Scenario: No runtime fallback for unknown classes
- **WHEN** the runbook builder processes diagnostics
- **THEN** no `unknown:{class}` fallback runbook sections SHALL be emitted, because all classes are handled at compile time
