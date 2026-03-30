# Plan: Issue #3647 - Fix tau-coding-agent cli integration training store lock race in auth provider tests

## Approach
1. Use the GitHub Actions failure on PR `#3631` as RED evidence.
2. Isolate the shared CLI integration subprocess harness from live RL startup by
   disabling `TAU_LIVE_RL_ENABLED` in `binary_command()`.
3. Rerun the auth-provider CLI integration slice under parallel test threads to
   confirm the harness still covers provider behavior while avoiding the shared
   SQLite store.
4. Rerun the exact `fast-validate --base <sha>` reproduction and push the fix
   back to PR `#3631`.

## Affected Areas
- `crates/tau-coding-agent/tests/cli_integration.rs`
- `specs/milestones/m330/index.md`
- `specs/3647/`

## Compatibility / Contract Notes
- No production runtime behavior changes are intended.
- This change only constrains the subprocess-based CLI integration test
  environment so unrelated live RL startup side effects do not bleed into auth
  provider coverage.

## Risks / Mitigations
- Risk: a CLI integration test may implicitly depend on live RL being enabled.
  Mitigation: search the CLI integration suite for live RL coverage; if none
  exist, disable it centrally in the helper rather than patching single tests.
- Risk: the race may also depend on shared current-directory state.
  Mitigation: start with the minimal harness isolation that directly removes the
  failing SQLite initialization path; only widen to temp-dir isolation if the
  race persists.
- Risk: another package-scoped blocker may surface immediately after this fix.
  Mitigation: keep the PR under observation after the push and peel the next
  blocker directly from GitHub logs.

## Verification
- `cargo test -p tau-coding-agent --test cli_integration auth_provider:: -- --test-threads 2`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- PR `#3631` GitHub Actions rerun
