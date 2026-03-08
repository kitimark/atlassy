## 1. Confluence Test Move

- [ ] 1.1 Create `crates/atlassy-confluence/tests/stub_client.rs` with the 3 tests from `src/tests.rs`, using `use atlassy_confluence::*` imports
- [ ] 1.2 Remove `#[cfg(test)] mod tests;` from `crates/atlassy-confluence/src/lib.rs`
- [ ] 1.3 Delete `crates/atlassy-confluence/src/tests.rs`
- [ ] 1.4 Run `cargo test -p atlassy-confluence` to verify parity

## 2. ADF Inline Tests — scope.rs

- [ ] 2.1 Add `#[cfg(test)] mod tests` block in `crates/atlassy-adf/src/scope.rs`
- [ ] 2.2 Add tests for `heading_level`: extracts from attrs, defaults to 6 when missing, defaults to 6 for out-of-range values
- [ ] 2.3 Add tests for `find_heading_paths`: matches exact heading text, returns empty on no match, recurses into nested content
- [ ] 2.4 Add tests for `find_block_paths`: matches `attrs.id`, matches `attrs.localId`, returns empty on no match
- [ ] 2.5 Add tests for `expand_heading_to_section`: includes nodes until same-level heading, returns empty for non-`/content/N` paths, returns empty for nested heading paths
- [ ] 2.6 Add tests for `full_page_resolution`: sets fallback fields correctly, passes through reason string

## 3. ADF Inline Tests — index.rs

- [ ] 3.1 Add `#[cfg(test)] mod tests` block in `crates/atlassy-adf/src/index.rs`
- [ ] 3.2 Add tests for `collect_text`: extracts text from text nodes, concatenates nested text, returns empty for non-text nodes
- [ ] 3.3 Add tests for `build_node_path_index_inner` (via `build_node_path_index`): handles empty doc, detects duplicate paths

## 4. ADF Inline Tests — path.rs

- [ ] 4.1 Add `#[cfg(test)] mod tests` block in `crates/atlassy-adf/src/path.rs`
- [ ] 4.2 Add tests for `compare_path_segments`: numeric ordering (`/2` < `/10`), prefix ordering (shorter < longer), equal paths
- [ ] 4.3 Add tests for `is_json_pointer`: accepts `/`-prefixed, rejects non-`/`-prefixed, rejects empty
- [ ] 4.4 Add tests for `escape_pointer_segment`: escapes `~` to `~0`, `/` to `~1`, passes through clean segments
- [ ] 4.5 Add tests for `parent_path`: root `"/"` returns `None`, `"/content/0"` returns `Some("/content")`, `"/content"` returns `Some("/")`

## 5. Contracts Inline Tests — validation.rs

- [ ] 5.1 Add `#[cfg(test)] mod tests` block in `crates/atlassy-contracts/src/validation.rs`
- [ ] 5.2 Add tests for `is_valid_git_sha`: accepts 40-char hex, rejects wrong length, rejects non-hex, rejects empty
- [ ] 5.3 Add tests for `is_within_scope`: root `"/"` allows all, exact match, prefix + `/` match, rejects non-prefix overlap

## 6. Quality Gates

- [ ] 6.1 `cargo fmt --all -- --check`
- [ ] 6.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 6.3 `cargo test --workspace` — all tests pass
- [ ] 6.4 Verify new test count with `grep -r '#\[test\]' crates/ --include='*.rs' | wc -l`
