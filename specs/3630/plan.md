# Plan: Issue #3630 - Split live_rl_runtime below oversized-file threshold

1. Map the largest cohesive seams inside
   `crates/tau-coding-agent/src/live_rl_runtime.rs`. The likely extraction
   targets are:
   - configuration/types that do not require the full runtime body;
   - report/span serialization helpers;
   - PPO/APO helper routines that can move behind private module boundaries;
   - auxiliary data structures and formatting helpers.
2. Extract private submodules incrementally into
   `crates/tau-coding-agent/src/live_rl_runtime/` or adjacent sibling files,
   keeping the root module as the stable integration surface.
3. After each extraction, re-run targeted selectors so the split remains
   behavior-preserving instead of becoming a hidden rewrite.
4. Finish by verifying the line count and oversized-file guard against repo
   policy inputs, without adding an exemption.

## Affected Modules
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- likely new private files under `crates/tau-coding-agent/src/live_rl_runtime/`
- `crates/tau-coding-agent/src/lib.rs` or binary module wiring only if required
  for `mod` declarations
- no planned policy-file edits unless verification proves existing metadata is
  stale for unrelated reasons

## Risks / Mitigations
- Risk: hidden coupling across private helpers makes extraction noisier than the
  line-count target suggests.
  - Mitigation: prefer moving cohesive helper groups first and keep the root
    module as the orchestration surface.
- Risk: the split drifts into behavior changes under cover of file movement.
  - Mitigation: no logic rewrites without separate justification; verify with
    targeted selectors after each coherent extraction.
- Risk: the root file stays above 4000 lines after an initial split.
  - Mitigation: capture line count early and continue extraction until the
    policy threshold is actually met.

## Interface / Contract Notes
- No intended public API, trait, schema, or wire-format changes.
- The accepted implementation shape is an internal source-layout refactor.
- If a temporary exemption becomes unavoidable, that requires a separate
  explicit decision because this story's acceptance criteria assume none.

## Test Plan
- RED:
  - `wc -l crates/tau-coding-agent/src/live_rl_runtime.rs`
  - `python3 .github/scripts/oversized_file_guard.py --repo-root . --exemptions-file tasks/policies/oversized-file-exemptions.json --policy-guide docs/guides/oversized-file-policy.md --json-output-file /tmp/oversized-file-guard.json`
- GREEN / REGRESSION:
  - `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c02_functional_optimizer_runs_on_update_interval -- --test-threads=1`
  - `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c18_regression_live_curriculum_aggregates_persisted_to_resources -- --test-threads=1`
  - `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c21_regression_live_apo_algorithm_failure_reports_deterministic_reason_code -- --test-threads=1`
- VERIFY:
  - rerun the line-count check
  - rerun oversized-file guard
  - run `cargo fmt --check`
  - run scoped `cargo clippy -p tau-coding-agent --bin tau-coding-agent -- -D warnings`
