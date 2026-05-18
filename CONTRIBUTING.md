# Contributing to Tau

Thanks for contributing to Tau.

## Prerequisites

- Rust stable toolchain (`rustup default stable`)
- `cargo` available on PATH
- GitHub issue access for issue/spec/PR linking

## Issue and Spec Workflow

1. Start from a GitHub issue with milestone + labels.
2. Create a branch from `master` using `codex/issue-<id>-<slug>`.
3. Create or update issue artifacts in `specs/<issue-id>/`:
   - `spec.md` (acceptance criteria + conformance cases)
   - `plan.md` (approach + risk)
   - `tasks.md` (ordered RED->GREEN->VERIFY checklist)
4. Keep changes scoped to the issue; avoid unrelated edits.

## Testing and Quality Gates

Run these before opening or updating a PR:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p <crate>
cargo check -q
```

For docs/scripts-only slices, run the relevant conformance scripts under `scripts/dev/`.

Use `docs/guides/test-coverage-targets.md` for crate-specific coverage and
conformance expectations. P0/P1 changes should name the spec id in at least one
targeted test command.

## Release Freshness Review

Before each release branch or release candidate:

- Re-read this guide against `.github/pull_request_template.md` and
  `AGENTS.md`; update any stale commands, branch names, or spec requirements.
- Regenerate crate dependency graph artifacts if workspace crate edges changed.
- Sample changed specs and confirm AC -> conformance case -> test command
  mapping is complete.
- Confirm docs-only exceptions have explicit `N/A` tier reasons.
- Record the review in the release checklist or PR description.

## Pull Request Checklist

- [ ] Summary explains what changed and why.
- [ ] Links include milestone, issue, spec, and plan paths.
- [ ] Acceptance criteria map to test evidence.
- [ ] RED/GREEN/REGRESSION evidence is included for TDD slices.
- [ ] Tier matrix is filled (no blank rows; `N/A` includes reason).
- [ ] Risk/rollback section is present.
- [ ] Docs/specs are updated when behavior changes.

## Commit Guidance

Use atomic commits by concern with the repository convention:

```text
spec|test|feat|fix|refactor|docs|chore(<scope>): <message> (#<issue>)
```

## Review Expectations

- Prefer smallest viable diff for the issue scope.
- Add regression tests for bug fixes and behavioral changes.
- Resolve CI failures before requesting merge.
