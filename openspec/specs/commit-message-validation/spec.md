## Purpose

Define shared conventional commit validation requirements for local hooks and CI.

## Requirements

### Requirement: Shared validation script validates conventional commits
A shared script at `scripts/validate-commit-msg.sh` SHALL validate that a commit message follows the conventional commits format: `<type>(<scope>): <description>`, where scope is optional.

#### Scenario: Valid message with scope
- **WHEN** the script receives `"feat(cli): add new command"`
- **THEN** the script exits with code 0

#### Scenario: Valid message without scope
- **WHEN** the script receives `"test: extract inline test modules"`
- **THEN** the script exits with code 0

#### Scenario: Invalid message rejected
- **WHEN** the script receives `"yolo push"`
- **THEN** the script exits with code 1 and prints an error describing the expected format

#### Scenario: Merge commit allowed through
- **WHEN** the script receives a message starting with `"Merge "`
- **THEN** the script exits with code 0

### Requirement: Validation enforces allowed types
The script SHALL accept only these types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `build`, `ci`, `perf`, `style`.

#### Scenario: Unknown type rejected
- **WHEN** the script receives `"yeet(cli): fast deploy"`
- **THEN** the script exits with code 1

### Requirement: Validation enforces subject line length
The script SHALL reject commit messages with a subject line longer than 72 characters.

#### Scenario: Long subject rejected
- **WHEN** the script receives a message with a subject line of 73 or more characters
- **THEN** the script exits with code 1 and reports the actual length

#### Scenario: Subject at limit accepted
- **WHEN** the script receives a message with a subject line of exactly 72 characters
- **THEN** the script exits with code 0

### Requirement: CI validates commit message on push
On push to `main`, the CI workflow SHALL validate the HEAD commit message using the shared validation script.

#### Scenario: Non-conforming push commit fails CI
- **WHEN** a commit with message `"quick fix"` is pushed to `main`
- **THEN** the commit validation step fails and the workflow reports failure

### Requirement: CI validates PR title on pull request
On pull request events, the CI workflow SHALL validate the PR title using the shared validation script.

#### Scenario: Non-conforming PR title fails CI
- **WHEN** a PR with title `"Update stuff"` targets `main`
- **THEN** the commit validation step fails and the workflow reports failure

#### Scenario: Conforming PR title passes CI
- **WHEN** a PR with title `"feat(cli): add seed-page command"` targets `main`
- **THEN** the commit validation step succeeds

### Requirement: Git hook delegates to shared script
The `.githooks/commit-msg` hook SHALL extract the first line from the commit message file and pass it to `scripts/validate-commit-msg.sh`. The hook SHALL use the script's exit code as its own.

#### Scenario: Hook rejects invalid commit locally
- **WHEN** a developer commits with message `"wip"`
- **THEN** the hook calls the shared script, which exits 1, and the commit is rejected

#### Scenario: Hook accepts valid commit locally
- **WHEN** a developer commits with message `"fix(adf): correct scope resolution"`
- **THEN** the hook calls the shared script, which exits 0, and the commit proceeds
