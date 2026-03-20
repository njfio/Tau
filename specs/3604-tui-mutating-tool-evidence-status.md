## Objective

Surface mutating-tool evidence state in the interactive TUI so build/create turns no longer look idle or deceptively complete while Tau has only read from the repo or has not written anything yet.

## Inputs/Outputs

- Inputs:
  - `App.last_submitted_input`
  - `App.tools.entries()`
  - `App.status.agent_state`
- Outputs:
  - main-shell live activity text shows whether the current build/create turn has no mutating evidence yet or is still read-only
  - run-state card summary reflects the same evidence state

## Boundaries/Non-goals

- No runtime-policy changes. This issue does not change tool execution, retries, or provider behavior.
- No full TUI layout redesign.
- No file preview, diff view, or artifact browser changes.
- No new persistence layer or new operator-state protocol fields.

## Failure modes

- A build/create prompt with no successful tool entries shows no evidence warning. This is incorrect and must fail tests.
- A build/create prompt with only successful non-mutating tool entries shows no read-only warning. This is incorrect and must fail tests.
- A build/create prompt with a successful mutating tool entry still shows a no-mutation warning. This is incorrect and must fail tests.
- A non-build prompt shows the mutating-evidence warning. This is incorrect and must fail tests.

## Acceptance criteria

- [ ] For a build/create prompt with no successful tool results, `Live activity` renders `no mutating evidence yet`.
- [ ] For a build/create prompt with successful non-mutating tool results only, `Live activity` renders `read-only so far`.
- [ ] For a build/create prompt with a successful mutating tool result, the mutating-evidence warning is absent from `Live activity`.
- [ ] For a build/create prompt with successful non-mutating tool results only, the run-state card summary renders `still read-only`.
- [ ] For a non-build prompt, neither `Live activity` nor the run-state card render mutating-evidence warning text.
- [ ] At least one integration-style TUI render test exercises the real render path with side panels still enabled.

## Files to touch

- `crates/tau-tui/src/interactive/ui_activity.rs`
- `crates/tau-tui/src/interactive/ui_run_state_model.rs`
- `crates/tau-tui/src/interactive/ui_tool_visibility_tests.rs`
- `crates/tau-tui/src/interactive/ui_tests/transcript.rs`

## Error semantics

- This issue does not add new runtime errors.
- When evidence state is unavailable or not applicable, the TUI omits the mutating-evidence warning rather than inventing one.
- The TUI must derive evidence strictly from known successful tool entries and the current submitted prompt.

## Test plan

- Add failing render tests for:
  - build/create prompt + no successful tools
  - build/create prompt + successful `read`
  - build/create prompt + successful `write`
  - non-build prompt + successful `read`
- Run targeted `cargo test -p tau-tui` tests for the new render assertions.
- Run full `cargo test -p tau-tui`.
