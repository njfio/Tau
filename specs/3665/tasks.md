# Tasks: Issue #3665 - Keep TUI client timeout above gateway runtime and provider budgets

- [x] T1 (RED): extend `tau-unified` shell tests to prove interactive TUI uses
      a larger timeout than the runtime/provider bootstrap path for default and
      override flows.
- [x] T2 (GREEN): derive a larger interactive client timeout in
      `tau-unified.sh` while keeping runtime/provider timeouts unchanged.
- [x] T3 (VERIFY): run the launcher shell test suite.

## Tier Mapping
- Functional: interactive TUI client timeout exceeds runtime/provider timeout
- Regression: default and override launcher command shape remains covered
