# Plan: Issue #3642 - Stabilize tau-coding-agent package-scoped tests exposed by fast-validate

## Approach
1. Reproduce the currently failing `tau-coding-agent` tests with focused test
   selectors.
2. Remove the blocking live-RL snapshot path from local runtime startup by
   making bridge registration compatible with async callers.
3. Add a small drop-based current-directory guard in the affected tests so the
   original cwd is restored even if a runtime assertion panics.
4. Rerun the focused failing tests, then rerun the exact `fast-validate`
   reproduction.

## Affected Areas
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `specs/milestones/m330/index.md`
- `specs/3642/`

## Compatibility / Contract Notes
- No user-facing behavior changes are intended.
- The live RL bridge still returns the same startup snapshot information; only
  the locking path changes for async safety.
- Test hardening should stay local to the tests that mutate process cwd.

## Risks / Mitigations
- Risk: changing live-RL bridge registration to async could affect startup
  sequencing.
  Mitigation: keep the call graph single-caller and preserve the same returned
  snapshot contents.
- Risk: the observed test failures may include more than one root cause.
  Mitigation: fix the shared-state/runtime-panic causes first, then rerun the
  exact failing selectors before broad validation.

## Verification
- `cargo test -p tau-coding-agent --bin tau-coding-agent regression_spec_2542_c03_run_local_runtime_prompt_executes_model_call -- --test-threads=1`
- `cargo test -p tau-coding-agent --bin tau-coding-agent regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent -- --test-threads=1`
- `cargo test -p tau-coding-agent --bin tau-coding-agent integration_tool_hook_subscriber_dispatches_pre_and_post_tool_call_hooks -- --test-threads=1`
- `cargo test -p tau-coding-agent --bin tau-coding-agent integration_run_prompt_with_cancellation_executes_textual_codex_write_tool_call_through_provider_client -- --test-threads=1`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
