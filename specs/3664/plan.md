# Plan: Issue #3664 - Align tau-unified request timeout with CLI provider backend timeouts

## Approach
1. Treat the unified launcher request timeout as the operator-facing upper bound
   for CLI-backed provider requests.
2. Forward the same timeout to the provider CLI backend flags when building the
   `tau-coding-agent` command.
3. Extend the existing shell tests to assert both default and override cases.

## Proposed Design
### Timeout forwarding
- In `build_up_command`, append:
  - `--openai-codex-timeout-ms <request_timeout_ms>`
  - `--anthropic-claude-timeout-ms <request_timeout_ms>`
  - `--google-gemini-timeout-ms <request_timeout_ms>`
- Keep the launcher interface unchanged; the alignment happens internally.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3664"
  change_surface:
    - symbol: "tau-unified provider timeout forwarding"
      location: "scripts/run/tau-unified.sh"
      change_type: "modification"
      current: "request timeout and provider CLI backend timeouts can diverge"
      proposed: "launcher forwards the request timeout to CLI provider timeout flags"
      compatibility: "safe"
      reason: "narrows operator surprise without changing CLI inputs"
  overall_compatibility: "safe"
  approach:
    strategy: "Align launcher timeout budgets across request and CLI provider backends"
    steps:
      - "append provider CLI timeout flags from request_timeout_ms"
      - "cover default command shape in shell tests"
      - "cover explicit timeout override in shell tests"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: unintended behavior change for non-CLI provider paths.
  Mitigation: only adds extra flags; direct API-key HTTP provider behavior is
  unchanged.
- Risk: launcher tests miss regressions in override handling.
  Mitigation: assert both the default and explicit override command lines.

## Verification
- `bash scripts/run/test-tau-unified.sh`
- `bash scripts/dev/test-root-just-launcher.sh`
