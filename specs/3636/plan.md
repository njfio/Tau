# Plan: Issue #3636 - Scope fast-validate fmt checks to changed Rust surface

## Approach
1. Add two deterministic shell fixtures to `scripts/dev/test-fast-validate.sh`:
   - unrelated fmt drift outside the diff should not fail PR-scoped validation,
   - unformatted changed Rust file inside the diff should still fail.
2. Update `scripts/dev/fast-validate.sh` so fmt behavior becomes:
   - `cargo fmt --all -- --check` for `--full` or full-workspace scope,
   - `rustfmt --check <changed .rs files>` for PR-scoped package validation,
   - no-op fmt when there are no changed Rust files.
3. Re-run the script regression suite and the exact `#3631` reproduction
   command to confirm the workspace-wide false positive is gone.

## Affected Areas
- `scripts/dev/fast-validate.sh`
- `scripts/dev/test-fast-validate.sh`
- `specs/milestones/m330/index.md`
- `specs/3636/`

## Compatibility / Contract Notes
- Full-workspace validation semantics stay unchanged.
- Only PR-scoped fmt behavior changes, and only to align with the existing
  package-scoped clippy/test model.
- This intentionally does not bless unrelated formatting drift; it only stops
  scoped PRs from inheriting unrelated workspace debt.

## Risks / Mitigations
- Risk: changed-file fmt could miss a generated or indirectly affected Rust
  file.
  Mitigation: preserve full-workspace fmt for explicit full-scope cases and
  keep scope limited to actual changed `.rs` files for PR mode.
- Risk: shell test fixtures become brittle.
  Mitigation: keep temp repos tiny and assert only stable signals.

## Verification
- `./scripts/dev/test-fast-validate.sh`
- `./scripts/dev/fast-validate.sh --base <base_sha>` in the `#3631` branch
- Targeted docs checks only if spec artifacts change trigger them:
  - `python3 .github/scripts/docs_link_check.py --repo-root .`
  - `scripts/dev/roadmap-status-sync.sh --check --quiet`
