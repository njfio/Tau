# Plan: Issue #3667 - Align tau-unified launcher with gateway turn timeout budget

## Approach
1. Treat the launcher request timeout as the base runtime budget for both
   provider requests and gateway turn attempts.
2. Forward `--turn-timeout-ms` alongside the existing request/provider timeout
   flags in `build_up_command`.
3. Extend launcher shell tests to assert default and override propagation.

## Proposed Design
### Runtime timeout forwarding
- Append `--turn-timeout-ms <request_timeout_ms>` to the generated
  `tau-coding-agent` command.
- Keep the user-facing launcher interface unchanged; the fix is internal to the
  generated runtime command.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3667"
  change_surface:
    - symbol: "tau-unified runtime turn-timeout forwarding"
      location: "scripts/run/tau-unified.sh"
      change_type: "modification"
      current: "launcher forwards request/provider timeout budgets but leaves gateway turn timeout disabled"
      proposed: "launcher forwards request timeout budget into --turn-timeout-ms as well"
      compatibility: "safe"
      reason: "aligns runtime behavior with the operator-facing timeout contract"
  overall_compatibility: "safe"
  approach:
    strategy: "Forward gateway turn timeout in the unified launcher"
    steps:
      - "append --turn-timeout-ms to runtime command generation"
      - "cover default propagation in launcher shell tests"
      - "cover explicit timeout override propagation in launcher shell tests"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: future launcher changes drift request and turn timeouts apart again.
  Mitigation: shell tests assert both values on default and override paths.
- Risk: a shared budget is too strict for some long-running tasks.
  Mitigation: operators can still raise the launcher request timeout, and the
  interactive client timeout already derives above that base budget.

## Verification
- `bash scripts/run/test-tau-unified.sh`
- `bash scripts/dev/test-root-just-launcher.sh`
