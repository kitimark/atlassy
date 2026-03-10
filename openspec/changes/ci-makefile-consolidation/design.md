## Context

The CI workflow (`.github/workflows/ci.yml`) and `Makefile` both define the same cargo commands for formatting, linting, and testing. The Makefile is the local development interface; CI is the server-side gate. Both should run identical commands, but currently they are maintained independently.

Current duplication:

| Step | ci.yml | Makefile |
|------|--------|----------|
| Format check | `cargo fmt --all -- --check` | `cargo fmt --all` (apply, no `--check`) |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | identical |
| Test | `cargo test --workspace` | identical |

## Goals / Non-Goals

**Goals:**
- Single source of truth for cargo commands in the Makefile.
- CI delegates to Makefile targets for format check, lint, and test steps.
- Local developers can run the same checks CI runs via `make` targets.

**Non-Goals:**
- No changes to CI behavior, step ordering, or failure semantics.
- No changes to the commit validation step (CI-specific, no Makefile equivalent).
- No new `make ci` composite target (decided against during exploration).

## Decisions

### D-1: Add `fmt-check` target, keep `fmt` unchanged

The existing `make fmt` applies formatting (`cargo fmt --all`). CI needs verify-only mode (`--check`). Rather than parameterizing `fmt`, add a separate `fmt-check` target. This keeps both use cases explicit and avoids surprising behavior from a `make fmt` that might not actually format.

### D-2: CI step names stay descriptive

CI steps keep human-readable names ("Check formatting", "Run clippy", "Run tests") even though they now call `make` targets. The `run:` field changes; the `name:` field does not.

### D-3: Commit validation step unchanged

The commit validation step uses CI-specific GitHub context variables (`github.event_name`, `github.event.pull_request.title`). This logic has no local Makefile equivalent and stays inline in the workflow.

## Risks / Trade-offs

- **Indirection**: Someone reading `ci.yml` must check the Makefile to see actual commands. Mitigated by the Makefile being small (< 30 lines) with self-explanatory target names.
- **Make dependency on CI runner**: `ubuntu-latest` includes `make` by default. No additional setup needed.
