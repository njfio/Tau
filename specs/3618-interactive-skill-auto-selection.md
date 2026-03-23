# 3618 Interactive Skill Auto-Selection

## Objective
Make the interactive Tau runtime automatically attach task-relevant skill guidance for concrete implementation requests across both the CLI interactive loop and the gateway-backed graphical TUI path, and make the active skill set visible in the graphical TUI so operators can verify what guidance is in effect.

## Inputs/Outputs
- Inputs:
  - interactive user prompt text
  - explicit CLI-selected skills from `cli.skills`
  - runtime skill catalog from `.tau/skills`
  - bundled repo skill catalog from `skills/`
- Outputs:
  - per-turn effective skill selection used in the runtime prompt path
  - visible active-skill indicator in the interactive TUI
  - deterministic fallback behavior when no skill matches

## Boundaries/Non-goals
- No full streaming/progress transport rewrite.
- No automatic loading of every installed skill.
- No changes to external trust or remote skill download semantics.
- No replacement of the current tool policy or orchestrator stack.

## Failure Modes
- Interactive runtime ignores relevant bundled skills when `.tau/skills` is empty.
- TUI shows active skills that do not match the runtime-selected set.
- Explicit CLI-selected skills are dropped when auto-selection runs.
- Unrelated prompts pick noisy skills and degrade baseline behavior.
- Missing bundled skills directory causes a crash instead of a clean empty-selection fallback.

## Acceptance Criteria
- [x] Interactive runtime merges `.tau/skills` with bundled repo `skills/` when composing prompt-time skill selection.
- [x] Prompt-based skill auto-selection runs for interactive implementation/build prompts and preserves explicit `cli.skills` selections.
- [x] A Phaser/web-game build request activates repo-shipped web-game skill guidance in the real gateway/runtime prompt path used by the graphical TUI.
- [x] The graphical interactive TUI shows the active skill names for the current turn before or during submission.
- [x] When no relevant skills match, the runtime and TUI both fall back to an empty auto-selection without errors.
- [x] Integration tests cover the runtime prompt path and the interactive TUI visibility path.

## Files To Touch
- `crates/tau-skills/src/lib.rs`
- `crates/tau-onboarding/src/startup_prompt_composition.rs`
- `crates/tau-onboarding/src/startup_transport_modes.rs`
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_state.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-tui/Cargo.toml`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/status.rs`
- `crates/tau-tui/src/interactive/ui_status.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `crates/tau-tui/src/main.rs`
- `skills/web-game-phaser.md`
- `specs/3618-interactive-skill-auto-selection.md`
- `specs/3618/plan.md`
- `specs/3618/tasks.md`

## Error Semantics
- Missing skill directories must resolve to an empty catalog, not an error.
- Unknown explicit skills remain hard errors through existing selection rules.
- Auto-selection must never suppress explicit skill choices.
- TUI visibility must reflect the same deterministic selection helper used by the runtime.

## Test Plan
- Add red tests for merged catalog loading from runtime and bundled skill directories.
- Add red tests for prompt-based auto-selection on a Phaser/web-game request.
- Add red tests proving explicit skills survive auto-selection.
- Add gateway/runtime tests asserting the graphical TUI transport path augments the system prompt with the bundled skill guidance.
- Add interactive TUI tests asserting active-skill visibility for a matching prompt and empty visibility for non-matching prompts.
- Run focused `tau-skills`, `tau-gateway`, `tau-coding-agent`, and `tau-tui` tests, then the relevant root launcher smoke.

## Status
- Implemented
