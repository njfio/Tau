# Plan: Issue #3650 - Fail just tui closed when runtime bootstrap is unavailable or unready

## Approach
1. Add deterministic shell regressions for the launcher bootstrap readiness contract.
2. Introduce a small readiness helper in `scripts/run/tau-unified.sh` so the TUI bootstrap path can distinguish ready runtime, unready runtime, and bind/runtime ownership failures.
3. Change `tui --bootstrap-runtime` from best-effort readiness to fail-closed readiness before launching the TUI.
4. Preserve normal `up`, `status`, `down`, non-bootstrap `tui`, live-shell, and runner-mode behaviors.

## Current Surface Map
- `just tui` runs `scripts/run/tau-unified.sh tui --model gpt-5.4 --request-timeout-ms 600000 --agent-request-max-retries 0`.
- `cmd_tui` defaults `bootstrap_runtime=true` outside runner mode and `false` inside runner mode unless explicitly overridden.
- `bootstrap_runtime_for_tui` calls `cmd_up`, then `wait_for_dashboard_artifacts`.
- Current readiness behavior logs `tau-unified: continuing while dashboard artifacts initialize (...)` and proceeds to launch TUI.
- `scripts/run/test-tau-unified.sh` already uses `TAU_UNIFIED_RUNNER` to fake `up`, `status`, `down`, and `tui` without starting Rust binaries.

## Proposed Design
### Fail-closed readiness helper
- Keep `cmd_up` responsible for starting or reusing the runtime.
- After bootstrap, require readiness before launching TUI.
- In runner mode, allow tests to model readiness explicitly by creating or omitting expected dashboard artifacts under the test dashboard state directory.
- On failure, emit an actionable diagnostic naming the state directory and bind value, then exit nonzero before `runner_mode=tui` is logged.

### Bind/runtime ownership handling
- Preserve pid-file ownership checks via existing `PID_FILE` / fingerprint behavior.
- Ensure an unmanaged bind conflict results in a failed `cmd_up` or readiness failure before TUI launch.
- Do not hide the failure by launching TUI anyway.

### Regression coverage
- Add `tui_bootstrap_readiness` as a focused test selector in `scripts/run/test-tau-unified.sh`.
- Test healthy bootstrap readiness.
- Test missing readiness artifacts fail closed and do not log TUI launch.
- Test bind/conflict-like failed bootstrap exits nonzero with actionable text.
- Keep the full existing launcher test path green.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3650"
  change_surface:
    - symbol: "tui bootstrap readiness"
      location: "scripts/run/tau-unified.sh"
      change_type: "fail-closed behavior"
      current: "logs a warning and launches TUI while readiness may still be absent"
      proposed: "exits nonzero before TUI launch when readiness is absent"
      compatibility: "intentional behavior change"
      reason: "launching a shell against a missing or wrong runtime produces opaque operator failures"
  overall_compatibility: "safe"
  version_impact: "patch"
```

## Risks / Mitigations
- Risk: stricter bootstrap blocks an ad-hoc debugging workflow.
  Mitigation: preserve `--no-bootstrap-runtime` for operators who intentionally want to attach manually.
- Risk: readiness artifacts are delayed on slow machines.
  Mitigation: keep the readiness timeout bounded and configurable only if tests prove the default is too short.
- Risk: shell tests become slow or flaky.
  Mitigation: keep readiness tests in runner mode and model artifacts with local temp files.

## Verification
- `bash scripts/run/test-tau-unified.sh tui_bootstrap_readiness`
- `bash scripts/run/test-tau-unified.sh`
- `bash -n scripts/run/tau-unified.sh`
- `cargo fmt --check`
- `git diff --quiet -- Cargo.toml Cargo.lock`
