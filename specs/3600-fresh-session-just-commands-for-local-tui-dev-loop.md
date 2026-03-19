## Objective

Add local `just` recipes that let developers start the Tau runtime and TUI against a clean default session without manually deleting gateway session state.

## Inputs/Outputs

- Inputs:
  - Existing local dev `justfile`
  - Local gateway session state under `.tau/gateway/openresponses/sessions/`
- Outputs:
  - `just session-reset`
  - `just stack-up-fresh`
  - `just tui-fresh`
  - A shell-backed regression test that verifies the recipes exist and the reset path works

## Boundaries/Non-goals

- Only the local development `justfile` loop in this worktree is in scope.
- Do not change remote session behavior, gateway protocol behavior, or provider/runtime semantics.
- Do not change existing `stack-up-fast` and `tui` semantics beyond composing them from the new reset recipe.

## Failure modes

- `session-reset` must fail loudly if the session path cannot be cleaned.
- `stack-up-fresh` must fail loudly if the runtime cannot start after reset.
- `tui-fresh` must fail loudly if the reset step fails before the TUI launches.

## Acceptance criteria

- [ ] `just --list` includes `session-reset`, `stack-up-fresh`, and `tui-fresh`.
- [ ] `just session-reset` removes `.tau/gateway/openresponses/sessions/default.jsonl` when present.
- [ ] `just session-reset` is idempotent when the default session file is absent.
- [ ] `just --show stack-up-fresh` composes `session-reset` and `stack-up-fast`.
- [ ] `just --show tui-fresh` composes `session-reset` and `tui`.
- [ ] Existing `stack-up-fast` and `tui` recipes remain available.

## Files to touch

- `justfile`
- `specs/3600-fresh-session-just-commands-for-local-tui-dev-loop.md`
- `scripts/dev/test-just-fresh-session.sh`

## Error semantics

- Recipe failures must be observable and stop execution with a non-zero exit code.
- Missing files during reset are not errors; they are treated as a no-op.
- No silent fallback to old session state when a fresh path is requested.

## Test plan

- Add a shell regression script that:
  - asserts the new recipes appear in `just --list`
  - verifies `stack-up-fresh` and `tui-fresh` show the expected composed commands via `just --show`
  - creates a temporary default session file in `.tau/gateway/openresponses/sessions/default.jsonl`
  - runs `just session-reset`
  - verifies the file was removed
  - reruns `just session-reset` to verify idempotency
- Run the shell regression script.
- Smoke test `just stack-up-fresh` and `just stack-down`.
