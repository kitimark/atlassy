# Contributing

Use a design-first flow.

- Add or update roadmap docs before implementation changes.
- Record major tradeoffs and defaults in decision notes.
- Keep proposals testable with measurable success criteria.

## Setup

After cloning, activate the git hooks:

```bash
make setup
```

This configures git to use the project's `.githooks/` directory for commit validation.

## Commit Messages

This project follows the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <description>

[optional body]
```

- Scope is optional. Include it when the change is localized to one area. Omit it when the change spans multiple crates or areas.
- Description starts with a lowercase letter.
- No trailing period on the subject line.
- Subject line must be 72 characters or fewer.
- Body is optional. Separate it from the subject with a blank line.

### Types

| Type | Purpose |
|------|---------|
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `refactor` | Code restructuring (no behavior change) |
| `test` | Adding or updating tests |
| `chore` | Maintenance, archiving, scaffolding |
| `build` | Build system or dependencies |
| `ci` | CI/CD configuration |
| `perf` | Performance improvement |
| `style` | Formatting, whitespace (no logic change) |

### Choosing the Right Type

Ask: **"Does this change make something possible that wasn't possible before?"**

- **YES** → `feat` (new enum variant, new function, new error code, new pipeline state, new CLI flag)
- **NO** → "Does it fix a bug?" → `fix`
- **NO** → "Does it restructure code with identical behavior?" → `refactor`
- **NO** → "Is it docs only?" → `docs`
- **NO** → "Is it test-only?" → `test`
- **NO** → "Is it maintenance/archiving/scaffolding?" → `chore`

The critical distinction between `feat` and `refactor`:

- `feat` = new behavior exists after this commit that did not exist before
- `refactor` = same inputs produce same outputs, code is just organized differently

### Common Mistakes

- Using `refactor` when adding new `Operation` variants or error codes → should be `feat` (new capability)
- Using `refactor(pipeline)` for changes spanning pipeline + contracts + adf → should omit scope: `refactor: ...`
- Using `feat` for renaming types or extracting modules → should be `refactor` (same behavior, different structure)
- Using `feat(openspec)` for creating change artifacts → should be `docs(openspec)` (planning documents, not code)
- Using `docs(openspec)` for archiving a change → should be `chore(openspec)` (maintenance/bookkeeping)

### Scopes

| Scope | Area |
|-------|------|
| `cli` | atlassy-cli crate |
| `pipeline` | atlassy-pipeline crate |
| `adf` | atlassy-adf crate |
| `confluence` | atlassy-confluence crate |
| `contracts` | atlassy-contracts crate |
| `roadmap` | roadmap docs |
| `openspec` | openspec artifacts |
| `qa` | QA playbooks and evidence |
| `ideas` | incubating ideas docs |

### Examples

```
feat(cli): embed git provenance at build time
```

```
refactor(pipeline): modularize pipeline crate into focused modules
```

```
docs(roadmap): clarify git_commit_sha and git_dirty as build-time values
```

```
chore(openspec): archive build-time-provenance change
```

```
test: extract inline test modules from production files
```

The last example omits the scope because the change spanned all five crates.

### Enforcement

The `.githooks/commit-msg` hook validates commit messages on every commit. Non-conforming messages are rejected. Merge commits are allowed through.
