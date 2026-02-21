# M224 - Whats-Missing Accuracy Resync

Status: In Progress

## Context
`tasks/whats-missing.md` exists as a stale local inventory and no longer matches the current repository state. Several entries now conflict with implemented capabilities, creating operational confusion.

## Scope
- Reconcile `tasks/whats-missing.md` with current verified codebase evidence.
- Add deterministic conformance checks preventing stale marker regressions.
- Keep unresolved gaps explicit and evidence-based.

## Linked Issues
- Epic: #3186
- Story: #3187
- Task: #3188

## Success Signals
- `scripts/dev/test-whats-missing.sh`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
