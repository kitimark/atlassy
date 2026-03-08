## 1. ErrorCode enum in atlassy-contracts

- [x] 1.1 Replace 12 `&str` error code constants in `constants.rs` with `ErrorCode` enum (12 variants, `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`)
- [x] 1.2 Add `as_str(&self) -> &'static str` method returning original `ERR_*` strings
- [x] 1.3 Implement `Display` for `ErrorCode` delegating to `as_str()`
- [x] 1.4 Implement `Serialize` for `ErrorCode` producing the `as_str()` value
- [x] 1.5 Add inline unit tests: round-trip `as_str()` for all 12 variants, `Display` matches `as_str()`
- [x] 1.6 Update `lib.rs` re-exports (`ErrorCode` replaces the 12 `ERR_*` constants)

## 2. PipelineError::Hard typed code field

- [x] 2.1 Change `PipelineError::Hard.code` from `String` to `ErrorCode` in `atlassy-pipeline/src/lib.rs`
- [x] 2.2 Update `PipelineError::Hard` Display impl to use `code` (may need `.as_str()` or `{}` formatting)
- [x] 2.3 Update all `PipelineError::Hard` construction sites in pipeline to use `ErrorCode::*` variants instead of `ERR_*.to_string()`

## 3. Typed error mapping functions

- [x] 3.1 Narrow `to_hard_error` signature from `impl Display` to `AdfError`; replace substring matching with exhaustive `match` on `AdfError` variants
- [x] 3.2 Update `confluence_error_to_hard_error` to use `ErrorCode::*` variants instead of `ERR_*.to_string()`
- [x] 3.3 Update `From<AdfError> for PipelineError` and `From<ConfluenceError> for PipelineError` impls (they delegate to the mapping functions — should compile without changes)

## 4. CLI bridge updates

- [x] 4.1 Update `PipelineError::Hard` construction in `main.rs` (line ~611) to use `ErrorCode::RuntimeBackend`
- [x] 4.2 Update `summary.error_codes` comparison sites to use `ErrorCode::*.as_str()` instead of `ERR_*` constants
- [x] 4.3 Update CLI test assertions that reference `ERR_*` constants to use `ErrorCode::*.as_str()`
- [x] 4.4 Update import statements in CLI (`ERR_*` constants → `ErrorCode`)

## 5. Verification

- [x] 5.1 `cargo fmt --all -- --check`
- [x] 5.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 5.3 `cargo test --workspace` — all existing tests pass with no output changes
