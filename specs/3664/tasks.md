# Tasks: Issue #3664 - Align tau-unified request timeout with CLI provider backend timeouts

- [x] T1 (RED): extend `tau-unified` shell tests to assert provider CLI timeout
      flags are present for both default and overridden request timeouts.
- [x] T2 (GREEN): forward the launcher request timeout to the CLI provider
      backend timeout flags in `tau-unified.sh`.
- [x] T3 (VERIFY): run the launcher shell test suite.

## Tier Mapping
- Functional: launcher command includes aligned provider timeout flags
- Regression: default and explicit timeout override command shape stays covered
