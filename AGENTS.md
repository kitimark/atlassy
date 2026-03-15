# AGENTS.md

## Setup

```bash
make setup          # activate git hooks (commit message validation)
cargo test --workspace   # run all tests (must pass before commit)
cargo clippy --workspace # lint (zero warnings required)
```

## Project Structure

Rust (stable) Cargo workspace with 5 crates:

- `atlassy-cli` — CLI entry point
- `atlassy-pipeline` — 10-state pipeline orchestrator
- `atlassy-adf` — ADF parsing, patching, ordering, validation
- `atlassy-confluence` — Confluence API client trait
- `atlassy-contracts` — shared types, enums, error codes

Other directories: `roadmap/` (planning docs), `openspec/` (change artifacts and specs), `ideas/` (incubating concepts), `qa/` (evidence and playbooks).

## Commit Messages

Format: `<type>(<scope>): <description>`

### Choosing the Type

Ask: **"Does this change make something possible that wasn't possible before?"**

- **YES** — use `feat` (new enum variant, new function, new error code, new pipeline state, new CLI flag)
- **NO** — then ask: "Does it fix a bug?" → `fix`. "Does it restructure code with identical behavior?" → `refactor`. "Is it docs only?" → `docs`.

The critical distinction:

- `feat` = new behavior exists after this commit that did not exist before
- `refactor` = same inputs produce same outputs, code is just organized differently

Examples:
- Adding `Operation::Insert` variant → `feat` (new capability)
- Renaming `build_patch_ops` to `validate_operations` with same logic → `refactor` (same behavior)
- Adding `ERR_INSERT_POSITION_INVALID` error code → `feat` (new error handling)
- Extracting verify checks into separate functions with identical behavior → `refactor`
- Updating roadmap docs → `docs`
- Creating OpenSpec change artifacts (proposal, design, specs, tasks) → `docs(openspec)`
- Archiving an OpenSpec change → `chore(openspec)`
- Syncing delta specs to main specs → `docs(openspec)`

### Choosing the Scope

Scope = the single crate or area affected. **Omit scope when changes span 2+ crates.**

Available scopes: `cli`, `pipeline`, `adf`, `confluence`, `contracts`, `roadmap`, `openspec`, `qa`, `ideas`

Rule: count the crate directories your changes touch. If more than one, omit scope.

- `refactor(adf): extract ordering module` — good (only atlassy-adf changed)
- `feat: add block insert/remove support` — good (contracts + adf + pipeline + cli all changed)
- `refactor(pipeline): unify operation model` — wrong if contracts/adf/cli also changed

### Subject Line Rules

- 72 characters max
- Start description with lowercase letter
- No trailing period
- Body optional, separated by blank line

For full type/scope tables and more examples, see `CONTRIBUTING.md`.

## Testing

Before every commit:

```bash
cargo test --workspace    # all tests must pass
cargo clippy --workspace  # zero warnings
```

## OpenSpec Workflow

Changes are tracked via OpenSpec artifacts in `openspec/changes/<name>/`:
- `proposal.md` → `design.md` + `specs/` → `tasks.md`
- Behavioral specs live in `openspec/specs/<capability>/spec.md`
- Use `/opsx-*` commands to manage the workflow
