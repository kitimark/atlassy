## 1. ErrorCode enum in atlassy-contracts

- [ ] 1.1 Replace 12 `&str` error code constants in `constants.rs` with `ErrorCode` enum (12 variants, `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`)
- [ ] 1.2 Add `as_str(&self) -> &'static str` method returning original `ERR_*` strings
- [ ] 1.3 Implement `Display` for `ErrorCode` delegating to `as_str()`
- [ ] 1.4 Implement `Serialize` for `ErrorCode` producing the `as_str()` value
- [ ] 1.5 Add inline unit tests: round-trip `as_str()` for all 12 variants, `Display` matches `as_str()`
- [ ] 1.6 Update `lib.rs` re-exports (`ErrorCode` replaces the 12 `ERR_*` constants)

## 2. PipelineError::Hard typed code field

- [ ] 2.1 Change `PipelineError::Hard.code` from `String` to `ErrorCode` in `atlassy-pipeline/src/lib.rs`
- [ ] 2.2 Update `PipelineError::Hard` Display impl to use `code` (may need `.as_str()` or `{}` formatting)
- [ ] 2.3 Update all `PipelineError::Hard` construction sites in pipeline to use `ErrorCode::*` variants instead of `ERR_*.to_string()`

## 3. Typed error mapping functions

- [ ] 3.1 Narrow `to_hard_error` signature from `impl Display` to `AdfError`; replace substring matching with exhaustive `match` on `AdfError` variants
- [ ] 3.2 Update `confluence_error_to_hard_error` to use `ErrorCode::*` variants instead of `ERR_*.to_string()`
- [ ] 3.3 Update `From<AdfError> for PipelineError` and `From<ConfluenceError> for PipelineError` impls (they delegate to the mapping functions — should compile without changes)

## 4. CLI bridge updates

- [ ] 4.1 Update `PipelineError::Hard` construction in `main.rs` (line ~611) to use `ErrorCode::RuntimeBackend`
- [ ] 4.2 Update `summary.error_codes` comparison sites to use `ErrorCode::*.as_str()` instead of `ERR_*` constants
- [ ] 4.3 Update CLI test assertions that reference `ERR_*` constants to use `ErrorCode::*.as_str()`
- [ ] 4.4 Update import statements in CLI (`ERR_*` constants → `ErrorCode`)

## 5. Verification

- [ ] 5.1 `cargo fmt --all -- --check`
- [ ] 5.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 5.3 `cargo test --workspace` — all existing tests pass with no output changes
