# M293 - Workspace gate stabilization post-e2e coverage

Status: In Progress

## Context
Post-e2e integration waves introduced deterministic workspace gate regressions on clean `master` runs.
M293 tracks scoped, test-first remediation for gate blockers so `fmt`, `clippy`, and `cargo test` remain green.

## Scope
- Fix deterministic workspace gate regressions with spec-bound acceptance criteria.
- Require RED/GREEN evidence for every blocker.
- Keep fixes minimal and behavior-preserving unless explicitly scoped.

## Linked Issues
- Task: `#3414` (workspace gate regressions)

## Success Signals
- Clean workspace runs pass on `origin/master` after remediation:
  - `cargo fmt --all -- --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- Regression suites for fixed blockers remain green.
