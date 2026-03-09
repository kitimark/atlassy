## 1. Build-time embedding

- [ ] 1.1 Add `build.rs` to `atlassy-cli` that runs `git rev-parse HEAD` and `git status --porcelain`, emits `cargo:rustc-env=GIT_COMMIT_SHA` and `cargo:rustc-env=GIT_DIRTY`
- [ ] 1.2 Add `cargo:rerun-if-changed` directives for `.git/HEAD` and `.git/index`
- [ ] 1.3 Ensure `build.rs` panics with a clear message if git is unavailable or SHA is malformed

## 2. Provenance collection update

- [ ] 2.1 Replace `resolve_git_commit_sha()` and `resolve_git_dirty()` with `env!()` constants in `provenance.rs`
- [ ] 2.2 Update `collect_provenance()` to use the compile-time constants instead of subprocess calls
- [ ] 2.3 Remove `resolve_git_commit_sha()` and `resolve_git_dirty()` functions

## 3. Spec rename

- [ ] 3.1 Rename `openspec/specs/runtime-provenance-stamping/` to `openspec/specs/build-provenance-stamping/`
- [ ] 3.2 Update spec content to reflect build-time sourcing

## 4. Testing

- [ ] 4.1 Add smoke test asserting `env!("GIT_COMMIT_SHA")` passes `is_valid_git_sha()`
- [ ] 4.2 Verify existing validation tests pass (`cargo test -p atlassy-contracts`)
- [ ] 4.3 Verify existing integration tests pass (`cargo test -p atlassy-cli`)
