## Context

All quality checks (format, lint, test) run manually via `make` targets. A `.githooks/commit-msg` hook enforces conventional commits locally, but nothing validates code or messages server-side. The project is moving to a PR-based workflow with squash merges, requiring automated gates before merge.

The roadmap (`13-ci-and-automation.md`) contains a researched Phase 1 plan with evaluated action versions and explicit rejection rationale. This design implements that plan plus commit message validation and squash-merge configuration.

Key constraints:
- The `build.rs` in `atlassy-cli` runs `git rev-parse HEAD` and `git status --porcelain` at compile time. CI needs a git checkout with `.git` present (default `actions/checkout` behavior).
- All 156 tests are self-contained — no Confluence credentials or external services needed in CI.
- The workspace uses `rustls-tls`, so no system OpenSSL is needed on the runner.

## Goals / Non-Goals

**Goals:**
- Automated format, lint, and test checks on every push and PR.
- Server-side conventional commit message validation matching local hook behavior.
- Reproducible Rust toolchain across local and CI environments.
- Squash-only merge enforcement with PR title as commit message.
- Branch protection gating merges on CI success.

**Non-Goals:**
- Release builds or binary artifact uploads (Phase 2 — deferred).
- Matrix builds across multiple Rust versions or OS targets (Phase 3 — deferred).
- Code coverage reporting (Phase 3 — deferred).
- Live Confluence smoke tests in CI (Phase 3 — deferred).
- Deployment pipelines (no deployment target exists).

## Decisions

### D-1: GitHub Actions action versions

Follow the roadmap's researched selections:

| Action | Ref | Rationale |
|--------|-----|-----------|
| `actions/checkout` | `@v4` | LTS line, no runner version constraints. v5/v6 require newer runners. |
| `dtolnay/rust-toolchain` | `@stable` | 1.5k stars, 70k dependents, maintained by David Tolnay. Toolchain selected by `@ref`. |
| `Swatinem/rust-cache` | `@v2` | Only active major version. Caches `target/` and cargo registry by `Cargo.lock` hash. |

Rejected: `actions-rust-lang/setup-rust-toolchain@v1` (bundles cache internally, higher complexity, 4x fewer stars), `actions-rs/toolchain@v1` (archived Oct 2023).

### D-2: Fail-fast step ordering

Format and clippy run before tests. Rationale: trivial formatting issues are caught in seconds without waiting for full test compilation. Commit message validation runs first — cheapest check, fails fastest.

### D-3: Commit validation scope by trigger

| Trigger | What to validate | Source |
|---------|-----------------|--------|
| `push` to `main` | HEAD commit message | `git log -1 --format='%s'` |
| `pull_request` | PR title | `github.event.pull_request.title` |

PR title validation is sufficient because squash merge uses the PR title as the commit message on `main`. Individual commits in the PR branch are irrelevant.

### D-4: Shared validation script over duplication

Extract conventional commit validation into `scripts/validate-commit-msg.sh`. Both `.githooks/commit-msg` and CI call this script. Eliminates regex duplication — one source of truth for types, scopes, format, and length rules.

The script accepts a commit message string as its first argument and exits 0/1. The git hook reads line 1 from the commit message file and passes it to the script.

### D-5: Squash-only merge enforcement

Configure the repository to allow only squash merges via GitHub API. Set `squash_merge_commit_title=PR_TITLE` so the squash commit message on `main` is the validated PR title. This guarantees every commit on `main` follows conventional commit format when using PRs.

### D-6: Checkout depth

Default `fetch-depth: 1` (shallow clone) is sufficient. The `build.rs` only needs `git rev-parse HEAD` (present in shallow clones) and `git status --porcelain` (works on shallow clones). Full history is not required.

## Risks / Trade-offs

- **Commit validation regex lives in a shell script, not in a compiled language.** The regex is simple and stable. The types/scopes list rarely changes. If it becomes a maintenance burden, a Rust-based validator could replace it later.
  → Mitigation: the shared script is the single source of truth, reducing drift risk to zero between hook and CI.

- **`rust-toolchain.toml` pins `stable` channel, not a specific version.** CI may use a different stable version than local if runner images update.
  → Mitigation: edition 2024 requires Rust >= 1.85.0. Any stable release satisfies this. Pinning a specific version (e.g., `1.94.0`) would require manual bumps. The `stable` channel is the right trade-off for a single-contributor project.

- **Branch protection requires the CI workflow to have run at least once.** GitHub cannot require a status check that has never been reported.
  → Mitigation: push the workflow to `main` first, wait for the run, then configure branch protection.

- **`build.rs` rerun-if-changed paths reference `.git/HEAD` relative to the crate directory, not the workspace root.** These paths resolve to `crates/atlassy-cli/.git/HEAD` which does not exist.
  → Mitigation: not a CI problem (CI always builds clean). Noted as a pre-existing issue outside this change's scope.
