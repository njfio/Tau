# Plan: Issue #3665 - Keep TUI client timeout above gateway runtime and provider budgets

## Approach
1. Treat the launcher `--request-timeout-ms` as the runtime/provider budget.
2. Derive a larger interactive TUI HTTP client timeout that includes the
   bounded gateway retry headroom plus a small transport grace window.
3. Keep the bootstrap `up` command on the original runtime/provider timeout.
4. Extend launcher shell tests to assert the split timeout behavior for default
   and override flows.

## Proposed Design
### Interactive client timeout derivation
- Add a shell helper in `tau-unified.sh` that mirrors the bounded gateway retry
  policy:
  - first attempt uses the full runtime/provider timeout
  - follow-up action retries use the reduced retry budget
  - the interactive client timeout becomes:
    `runtime_timeout + bounded_retry_headroom + transport_grace`
- Use the derived timeout only for `tau-tui interactive`.

### Runtime/bootstrap timeout handling
- Keep `cmd_up` and `build_up_command` on the operator-specified
  `request_timeout_ms`.
- In `cmd_tui`, use the derived timeout only when building the interactive TUI
  command and the runner-mode trace for that path.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3665"
  change_surface:
    - symbol: "tau-unified interactive client timeout derivation"
      location: "scripts/run/tau-unified.sh"
      change_type: "modification"
      current: "interactive TUI client uses the same timeout as runtime/provider requests"
      proposed: "interactive TUI client uses a derived timeout above the runtime/provider budget"
      compatibility: "safe"
      reason: "narrows spurious transport aborts without changing runtime/provider request budgets"
  overall_compatibility: "safe"
  approach:
    strategy: "Separate interactive client and runtime/provider timeout budgets in the launcher"
    steps:
      - "derive interactive client timeout from the runtime budget plus bounded retry headroom"
      - "keep bootstrap runtime/provider timeout unchanged"
      - "cover default and override separation in launcher shell tests"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: launcher derivation drifts from gateway retry policy.
  Mitigation: document that the shell helper mirrors the bounded gateway retry
  constants and keep the test values explicit.
- Risk: tests only prove command shape, not live runtime behavior.
  Mitigation: keep the regression focused on the launcher contract that caused
  the observed client-side transport aborts.

## Verification
- `bash scripts/run/test-tau-unified.sh`
- `bash scripts/dev/test-root-just-launcher.sh`
