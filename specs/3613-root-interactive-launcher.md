# 3613 Root Interactive Launcher

## Objective
Make the sanctioned root-path dev loop launch the graphical `tau-tui interactive` entrypoint instead of the legacy `tau-tui agent` shell path.

## Inputs/Outputs
- Input: `just tui`, `just tui-fresh`, or `./scripts/run/tau-unified.sh tui` from the root repo.
- Output: a command path that launches `cargo run -p tau-tui -- interactive --profile <profile> --model <model>` and preserves the validated root dev defaults for timeout and retry behavior where applicable.

## Boundaries/Non-goals
- Do not redesign the interactive UI.
- Do not remove the `agent` subcommand from `tau-tui`.
- Do not change provider/runtime semantics beyond what is required to launch the correct TUI entrypoint.
- Do not add new dependencies.

## Failure Modes
- `tau-unified.sh tui` still launches `tau-tui agent` instead of `interactive`.
- `just tui` or `just tui-fresh` still route to the legacy shell path.
- Request-timeout / retry flags are dropped or translated incorrectly in the launcher path.
- Root smoke launch fails because the runtime bootstrap and interactive TUI path are not wired together.

## Acceptance Criteria
- [ ] `./scripts/run/tau-unified.sh tui --help` and implementation support the graphical interactive path as the default TUI mode.
- [ ] `just tui` launches the graphical `tau-tui interactive` mode from the root repo path.
- [ ] `just tui-fresh` launches the same graphical mode after resetting session state.
- [ ] The root launcher preserves the validated model default `gpt-5.3-codex`.
- [ ] At least one integration test proves the root launcher resolves to `interactive`, not `agent`.
- [ ] A real root-path smoke test validates the command wiring.

## Files To Touch
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `justfile`
- `scripts/dev/test-root-just-launcher.sh`
- `specs/3613-root-interactive-launcher.md`

## Error Semantics
- Unknown or incompatible launcher flags must hard-fail with a non-zero exit and a clear error message.
- Missing or invalid integer timeout / retry values must hard-fail during argument parsing.
- The launcher must not silently fall back from `interactive` to `agent`.

## Test Plan
- Add red assertions in launcher tests proving the root `tui` path resolves to `interactive`.
- Update shell-based integration checks for `just --dry-run tui` and `tui-fresh` to assert the resolved command path.
- Run targeted launcher tests.
- Smoke-test the root path with `just stack-down`, `just stack-up-fast`, and a dry-run / live TUI invocation as appropriate.
