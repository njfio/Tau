# Tasks: Issue #3667 - Align tau-unified launcher with gateway turn timeout budget

- [x] T1 (RED): extend `tau-unified` shell tests to assert
      `--turn-timeout-ms` is forwarded for default and overridden timeout
      values.
- [x] T2 (GREEN): forward the launcher request timeout budget to
      `--turn-timeout-ms` in the generated runtime command.
- [x] T3 (VERIFY): run the launcher shell verification stack.

## Tier Mapping
- Functional: runtime command includes `--turn-timeout-ms`
- Regression: default and explicit timeout override launcher shape stays covered
