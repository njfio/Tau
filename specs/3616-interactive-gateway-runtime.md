# 3616 Interactive Gateway Runtime

## Objective
Make `tau-tui interactive` use the real gateway/runtime path instead of the local echo fallback, so the sanctioned root graphical TUI is actually functional.

## Inputs/Outputs
- Input: `cargo run -p tau-tui -- interactive ...` or root `just tui` / `just tui-fresh`.
- Output: user prompts are submitted to the gateway-backed runtime and produce real assistant responses, with visible gateway/runtime posture in the interactive TUI.

## Boundaries/Non-goals
- Do not redesign the full interactive UI.
- Do not remove the `agent` subcommand.
- Do not add silent fallback back to the local echo path.
- Do not require streaming support if a non-streaming gateway response is sufficient for a correct first integration.

## Failure Modes
- Interactive still appends `Received your message...` instead of submitting to the gateway.
- Interactive launches without enough configuration to reach the gateway/runtime path.
- Runtime submission fails silently or leaves the TUI looking successful.
- Root `just tui` launches graphical mode but still does not submit real turns.
- Tool/runtime posture remains indistinguishable from local fallback mode.

## Acceptance Criteria
- [ ] `tau-tui interactive` no longer uses the local echo fallback for normal prompt submission.
- [ ] `tau-tui interactive` accepts the gateway/runtime configuration required by the root launcher.
- [ ] Root `just tui` and `just tui-fresh` launch a graphical TUI that submits real turns through the runtime.
- [ ] The interactive status/UI clearly identifies the transport as gateway-backed.
- [ ] At least one integration test proves a prompt through `interactive` reaches the runtime-backed path and does not render `Received your message...`.
- [ ] A root smoke test validates the graphical TUI path on the real launcher flow.

## Files To Touch
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_runtime.rs`
- `crates/tau-tui/src/interactive/status.rs`
- `crates/tau-tui/src/interactive/ui_status.rs`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `specs/3616-interactive-gateway-runtime.md`

## Error Semantics
- Gateway request failures must appear as explicit system/error messages in the interactive transcript.
- Missing or invalid gateway configuration must hard-fail at startup or argument parsing.
- The interactive TUI must not silently fall back to the fake local echo response path.

## Test Plan
- Add failing tests for interactive argument parsing / launcher wiring.
- Add failing tests for prompt submission proving the local echo text is gone on the real path.
- Add an integration test for the interactive runtime-backed path.
- Smoke-test root `just stack-up-fast` + `just tui`.
