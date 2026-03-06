# Atlassy OpenSpec

This folder contains executable specification assets for Atlassy.

## Structure

- `specs/`: active behavior specs that define current expected system behavior.
- `changes/`: change proposals and execution records.
- `changes/archive/`: completed change sets kept for traceability.

## Relationship to `roadmap/`

- `roadmap/` defines strategic planning, defaults, KPI goals, and readiness policy.
- `openspec/` defines implementation-trackable behavior contracts and phase-level execution changes.
- When roadmap strategy changes, update or add OpenSpec specs and change artifacts before implementation.

## Provenance and Evidence

- OpenSpec change outcomes should reference reproducible commands.
- Runtime outputs in `artifacts/` are temporary and non-versioned.
- Decision-grade outcomes should include commit provenance (`git_commit_sha`, `git_dirty`, `pipeline_version`).
