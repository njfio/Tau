# Plan: Issue #3601 - Align CLI backend timeout with request timeout budget

## Approach
1. Add small provider-side timeout selection helpers for CLI backends.
2. Prove the helper floors Codex, Claude, and Gemini backend timeouts by `request_timeout_ms` while preserving larger explicit backend-specific values.
3. Add a Codex mock integration regression where the mock sleeps longer than the old backend timeout but shorter than the caller request timeout budget.
4. Wire the helper into Codex, Claude, Gemini, and Codex app-server client construction without touching HTTP provider timeout behavior.

## Proposed Design
### Shared timeout selector
- Introduce a helper in `crates/tau-provider/src/client.rs`:
  `cli_backend_timeout_ms(backend_timeout_ms, request_timeout_ms) -> u64`.
- Return `backend_timeout_ms.max(1).max(request_timeout_ms.max(1))`.
- Use this helper when constructing:
  - `CodexCliConfig.timeout_ms`
  - `CodexAppServerConfig.timeout_ms`
  - `ClaudeCliConfig.timeout_ms`
  - `GeminiCliConfig.timeout_ms`

### Regression coverage
- Unit tests cover the helper for smaller backend timeout, larger backend timeout, and zero values.
- Provider-level filtered test name `cli_backend_timeout_budget` covers all three backend selections.
- Codex integration coverage can use the existing fake CLI harness to prove a request succeeds when the backend would previously have timed out earlier than the caller budget.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3601"
  change_surface:
    - symbol: "CLI backend timeout selection"
      location: "crates/tau-provider/src/client.rs"
      change_type: "behavioral alignment"
      current: "CLI backend subprocesses use backend-specific timeout flags even when shorter than request_timeout_ms"
      proposed: "CLI backend timeout is max(backend_specific_timeout_ms, request_timeout_ms)"
      compatibility: "safe"
      reason: "shorter backend timeouts are premature relative to the caller request budget; larger explicit backend timeouts remain preserved"
  overall_compatibility: "safe"
  version_impact: "patch"
```

## Risks / Mitigations
- Risk: a user intentionally configured a shorter backend timeout as a fail-fast guard.
  Mitigation: #3601's requirement treats request timeout as the upper-level budget; preserving shorter backend timeouts caused premature false failures in gateway/TUI runs.
- Risk: app-server and subprocess Codex paths diverge.
  Mitigation: use the same timeout selector for both Codex backend constructors.
- Risk: test filters accidentally miss Claude/Gemini.
  Mitigation: use `cli_backend_timeout_budget` in every new test name.

## Verification
- `cargo test -p tau-provider cli_backend_timeout_budget -- --test-threads=1`
- `cargo test -p tau-provider codex_cli_client -- --test-threads=1`
- `cargo test -p tau-provider claude_cli_client -- --test-threads=1`
- `cargo test -p tau-provider gemini_cli_client -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p tau-provider -p tau-tui --tests --no-deps -- -D warnings`
- `git diff --quiet -- Cargo.toml Cargo.lock`
