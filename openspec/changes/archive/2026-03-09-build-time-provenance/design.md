## Context

`git_commit_sha` and `git_dirty` are currently resolved at runtime by spawning `git rev-parse HEAD` and `git status --porcelain` in `provenance.rs`. This means provenance reflects the repository where the CLI runs, not the repository state when the binary was compiled. `pipeline_version` is already a compile-time constant.

## Goals / Non-Goals

**Goals:**

- Embed `git_commit_sha` and `git_dirty` into the CLI binary at compile time.
- Remove runtime git subprocess dependency from provenance collection.
- Keep `collect_provenance()` as the single entry point for building a `ProvenanceStamp`.

**Non-Goals:**

- Changing `ProvenanceStamp` struct fields or JSON output shape.
- Changing `pipeline_version` sourcing (already a compile-time constant).
- Changing `runtime_mode` sourcing (correctly remains a runtime argument).

## Decisions

### D1: Use `build.rs` with `cargo:rustc-env`

Add a `build.rs` to `atlassy-cli` that runs `git rev-parse HEAD` and `git status --porcelain` during compilation. The values are emitted via `cargo:rustc-env=GIT_COMMIT_SHA=<sha>` and `cargo:rustc-env=GIT_DIRTY=<true|false>`, then read in source via `env!()`.

**Alternatives considered:**

- `vergen` or `git2` crate: Adds an external dependency for something achievable in ~15 lines of `build.rs`. Not justified for this scope.
- `option_env!()` with fallback: Would allow builds without git, but provenance is mandatory for decision-grade outputs — a build without git context should fail, not silently degrade.

### D2: Fail the build if git is unavailable

`build.rs` must `panic!` if `git rev-parse HEAD` fails or returns a malformed SHA. A binary without valid provenance is not useful for this project's evidence requirements.

### D3: Rerun directive scoped to `.git/HEAD` and `.git/index`

Use `cargo:rerun-if-changed=.git/HEAD` and `cargo:rerun-if-changed=.git/index` so `build.rs` re-executes when the commit changes or the staging area changes. This keeps incremental builds fast while ensuring the embedded values stay current.

### D4: Rename spec from `runtime-provenance-stamping` to `build-provenance-stamping`

The spec name should reflect that provenance values are now sourced at build time. The existing requirements (presence, validation, consistency) remain unchanged — only the sourcing mechanism and name change.

## Risks / Trade-offs

- [Stale dirty flag on non-Rust file changes] The `cargo:rerun-if-changed` directives track `.git/HEAD` and `.git/index`. If a developer modifies a tracked file without staging it, `git_dirty` may not update until the next commit or stage operation. This is acceptable — `git_dirty` is a best-effort cleanliness signal, not a security boundary. → Mitigation: document that `cargo build` after `git add` or `git commit` ensures accurate dirty state.
- [CI builds require git] `build.rs` will fail in environments without git (e.g., Docker builds from tarballs). → Mitigation: all current CI runs from a git checkout; document the requirement.
