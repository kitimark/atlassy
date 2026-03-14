# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/kitimark/atlassy/compare/v0.1.0...v0.1.1) - 2026-03-14

### Fixed

- *(cli)* clarify help text for readiness checks

## [0.1.0](https://github.com/kitimark/atlassy/releases/tag/v0.1.0) - 2026-03-14

### Added

- *(readiness)* support lifecycle attestations for gate 7
- *(cli)* embed git provenance at build time
- auto-discover scoped edit targets
- implement lifecycle subpage creation and bootstrap
- harden live runtime execution evidence
- *(cli)* implement phase5 readiness workflow
- *(cli)* complete phase4 PoC reporting and gating
- implement phase3 table cell route behavior
- implement phase2 prose assist route behavior
- implement phase1 Rust pipeline skeleton

### Fixed

- *(pipeline)* remove dead simple-scoped-update mode
- *(cli)* generate runtime UTC timestamps
- stabilize live runtime startup and publish contract

### Other

- *(cli)* format provenance imports for CI
- *(cli)* replace diagnostic string taxonomy with typed enums
- *(cli)* modularize into facade modules
- make hard error codes compiler-checked
- extract inline test modules from production files
- align KPI telemetry with revised framework
