# Plan #2548

1. Add a runtime prompt-template hot-reload bridge that polls workspace template inputs and recomposes startup system prompt with deterministic outcomes (`applied`, `no_change`, `invalid`, `missing_template`).
2. Extend runtime agent surface with a safe system-prompt update path that preserves existing conversation history.
3. Wire bridge lifecycle into `run_local_runtime` start/shutdown flow.
4. Add conformance/regression tests first (RED), implement bridge/update path (GREEN), then mutation harden.
5. Run scoped + full verification gates and live validation.

## Risks
- Updating system prompt in-place may disturb message ordering/history if not done carefully.
- Template parsing failures may cause noisy churn if diagnostics are not deduplicated.

## Mitigations
- Add explicit agent helper that updates only the leading system message deterministically.
- Keep bridge fingerprinting + no-op guard to avoid reapply churn.
- Preserve last-known-good prompt on invalid updates and assert this with regressions.
