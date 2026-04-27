# Plan: Issue #3585 - Codex auth runtime rejects unsupported models and stops TUI hangs

## Approach
1. Locate the Codex auth/provider configuration boundary and the TUI error rendering path used by gateway/runtime failures.
2. Add RED tests that reproduce the unsupported `openai/gpt-5.2` Codex-auth path and assert the failure is immediate, structured, and actionable.
3. Implement a narrow validation helper that rejects unsupported Codex-auth model identifiers before the subprocess or runtime request can hang.
4. Preserve supported-model behavior and existing Codex CLI adapter contracts.

## Proposed Design
### Provider-side model compatibility guard
- Introduce a small Codex-auth compatibility predicate or validator in `crates/tau-provider`.
- Reject known incompatible OpenAI API-style model identifiers when the auth mode is Codex/ChatGPT-account based.
- Include the configured model and auth mode in the error message.
- Do not silently rewrite the model or fall back to another provider.

### TUI-facing actionable text
- Keep provider errors structured enough for gateway/TUI callers to display the root cause.
- Add a TUI regression around the existing error conversion/rendering surface so the operator sees guidance such as selecting a supported Codex-auth model or changing auth mode.

### Supported path compatibility
- Add a positive test for a supported Codex-auth model configuration.
- Rerun existing Codex CLI provider tests to catch behavior drift in prompt rendering, textual tool-call promotion, and timeout handling.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3585"
  change_surface:
    - symbol: "Codex auth model compatibility validation"
      location: "crates/tau-provider"
      change_type: "additive validation"
      current: "unsupported Codex-auth models can reach runtime request dispatch and hang"
      proposed: "unsupported Codex-auth models fail closed before or at dispatch with actionable errors"
      compatibility: "safe"
      reason: "the rejected configurations already fail in the underlying Codex backend; Tau will surface the failure earlier and clearer"
    - symbol: "TUI unsupported-model error presentation"
      location: "crates/tau-tui/src/interactive"
      change_type: "error handling improvement"
      current: "operators can see long thinking states or generic timeout text"
      proposed: "operators see model/auth incompatibility guidance in-session"
      compatibility: "safe"
      reason: "only changes the presentation of an existing failure mode"
  overall_compatibility: "safe"
  version_impact: "patch"
```

## Risks / Mitigations
- Risk: the supported Codex-auth model allowlist becomes stale.
  Mitigation: keep the guard narrow to known incompatible API-style model identifiers and include tests for both rejected and accepted examples.
- Risk: TUI changes conflict with unrelated dirty operator-shell edits.
  Mitigation: read current files before editing and keep the TUI change focused on existing error text/conversion surfaces.
- Risk: provider validation accidentally affects API-key OpenAI paths.
  Mitigation: key the rejection to Codex/ChatGPT-account auth mode only and keep existing API-key tests out of scope.

## Verification
- `cargo test -p tau-provider codex_auth_unsupported -- --test-threads=1`
- `cargo test -p tau-provider codex_cli_client -- --test-threads=1`
- `cargo test -p tau-tui codex_auth_unsupported -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p tau-provider -p tau-tui --tests --no-deps -- -D warnings`
- `git diff --quiet -- Cargo.toml Cargo.lock`
