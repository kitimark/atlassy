## 1. Build-time embedding

- [x] 1.1 Add `build.rs` to `atlassy-cli` that runs `git rev-parse HEAD` and `git status --porcelain`, emits `cargo:rustc-env=GIT_COMMIT_SHA` and `cargo:rustc-env=GIT_DIRTY`
- [x] 1.2 Add `cargo:rerun-if-changed` directives for `.git/HEAD` and `.git/index`
- [x] 1.3 Ensure `build.rs` panics with a clear message if git is unavailable or SHA is malformed

## 2. Provenance collection update

- [x] 2.1 Replace `resolve_git_commit_sha()` and `resolve_git_dirty()` with `env!()` constants in `provenance.rs`
- [x] 2.2 Update `collect_provenance()` to use the compile-time constants instead of subprocess calls
- [x] 2.3 Remove `resolve_git_commit_sha()` and `resolve_git_dirty()` functions

## 3. Spec rename

- [x] 3.1 Rename `openspec/specs/runtime-provenance-stamping/` to `openspec/specs/build-provenance-stamping/`
- [x] 3.2 Update spec content to reflect build-time sourcing

## 4. Testing

- [x] 4.1 Add smoke test asserting `env!("GIT_COMMIT_SHA")` passes `is_valid_git_sha()`
- [x] 4.2 Verify existing validation tests pass (`cargo test -p atlassy-contracts`)
- [x] 4.3 Verify existing integration tests pass (`cargo test -p atlassy-cli`)
