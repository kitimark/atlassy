## 1. Toolchain Configuration

- [x] 1.1 Create `rust-toolchain.toml` at repo root with `channel = "stable"` and components `rustfmt`, `clippy`

## 2. Shared Commit Validation Script

- [x] 2.1 Create `scripts/validate-commit-msg.sh` that accepts a commit message string as `$1`, validates conventional commit format (types, optional scope, description), enforces 72-char subject limit, and allows merge commits through
- [x] 2.2 Slim `.githooks/commit-msg` to extract line 1 from the message file and delegate to `scripts/validate-commit-msg.sh`
- [x] 2.3 Verify local hook still rejects invalid commits and accepts valid ones after the refactor

## 3. CI Workflow

- [x] 3.1 Create `.github/workflows/ci.yml` with triggers on push to `main` and pull requests targeting `main`
- [x] 3.2 Add checkout step (`actions/checkout@v4`), toolchain step (`dtolnay/rust-toolchain@stable`), and cache step (`Swatinem/rust-cache@v2`)
- [x] 3.3 Add commit message validation step: validate HEAD commit on push, PR title on pull request, using `scripts/validate-commit-msg.sh`
- [x] 3.4 Add format check step (`cargo fmt --all -- --check`)
- [x] 3.5 Add lint step (`cargo clippy --workspace --all-targets -- -D warnings`)
- [x] 3.6 Add test step (`cargo test --workspace`)

## 4. Push and Verify

- [x] 4.1 Commit all new and modified files and push to `main`
- [ ] 4.2 Verify CI workflow runs and passes on GitHub Actions

## 5. Repository Configuration

- [ ] 5.1 Configure squash-only merging via `gh api` (`allow_squash_merge=true`, `allow_merge_commit=false`, `allow_rebase_merge=false`, `squash_merge_commit_title=PR_TITLE`)
- [ ] 5.2 Enable branch protection via `gh api` (require `test` check, `strict: true`, `enforce_admins: false`)
