# ADF + Markdown Dual Representation

## Status

Incubating (not scheduled)

## Plain Problem Points

- ADF is reliable for Confluence fidelity but hard for humans to read and edit directly.
- Markdown is easy to read and edit but cannot represent all Confluence features.
- Round-trip conversion between Markdown and ADF can cause drift in complex blocks.
- Large full-body payloads increase token usage and retry cost during AI-assisted editing.

## Proposed Direction

Use a block-level dual representation:

- Markdown-editable blocks for prose and simple structures.
- Locked ADF blocks for high-risk Confluence-native features.
- Minimal patch updates that preserve locked ADF blocks unless explicitly targeted.

## Why Not Now

- Current project phase is focused on foundation and roadmap definition.
- This idea needs schema and conversion validation before implementation.
- It introduces additional complexity in parser, patch planner, and fidelity checks.

## Risks

- Incorrect block boundary detection may edit protected content.
- Over-locking can reduce usability for writers.
- Under-locking can regress formatting fidelity in production pages.
- Confluence API behavior changes can affect conversion stability.

## Signals To Revisit

- Repeated fidelity regressions from Markdown-only updates.
- High token costs from ADF-heavy prompts.
- Frequent manual rework for tables, media, or macro-rich pages.
- Clear owner and capacity available for a dedicated PoC.

## Promotion Path

Move this idea to `roadmap/` when all conditions are true:

- A v1 scope and support matrix are approved.
- KPI targets are defined and accepted.
- Test corpus for round-trip fidelity is prepared.
- Phase 0 implementation capacity is committed.
