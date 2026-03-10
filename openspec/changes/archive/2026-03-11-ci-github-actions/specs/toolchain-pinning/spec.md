## ADDED Requirements

### Requirement: Rust toolchain pinned via configuration file
A `rust-toolchain.toml` file at the repository root SHALL pin the Rust toolchain to the `stable` channel with `rustfmt` and `clippy` components.

#### Scenario: Toolchain file selects stable channel
- **WHEN** `rustup` reads the `rust-toolchain.toml` file
- **THEN** the `stable` Rust channel is selected

#### Scenario: Components are available
- **WHEN** the toolchain is installed from the configuration file
- **THEN** `rustfmt` and `clippy` components are available

### Requirement: CI uses the pinned toolchain
The CI workflow SHALL install the Rust toolchain using the `rust-toolchain.toml` configuration, ensuring CI and local development use the same channel and components.

#### Scenario: CI toolchain matches local
- **WHEN** the CI workflow installs the Rust toolchain
- **THEN** it uses the `stable` channel with `rustfmt` and `clippy` as specified in `rust-toolchain.toml`
