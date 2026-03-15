## 1. Remove version from internal path dependencies

- [ ] 1.1 Remove `, version = "0.1.0"` from `atlassy-contracts` dep in `crates/atlassy-adf/Cargo.toml`
- [ ] 1.2 Remove `, version = "0.1.0"` from `atlassy-adf`, `atlassy-confluence`, and `atlassy-contracts` deps in `crates/atlassy-pipeline/Cargo.toml`
- [ ] 1.3 Remove `, version = "0.1.0"` from `atlassy-confluence`, `atlassy-contracts`, and `atlassy-pipeline` deps in `crates/atlassy-cli/Cargo.toml`

## 2. Verify

- [ ] 2.1 Run `cargo check` to confirm workspace compiles without errors
