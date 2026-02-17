# Tasks: Issue #2393 - Implement /tau send-file dispatch and audit logging

## Ordered Tasks
1. T1 (tests first): add failing C-01..C-05 tests in runtime/outbound test modules.
2. T2: implement parser/help/renderer and command execution metadata for send-file.
3. T3: implement outbound `deliver_file(...)` and runtime suppression-path integration.
4. T4: run fmt/clippy/full crate tests for `tau-multi-channel`.
5. T5: run mutation-in-diff and close any escapes.
6. T6: update issue process log, open PR with AC mapping and tier matrix, merge when green.

## Tier Mapping
- Unit: C-01, C-02 parser/help.
- Functional: C-03, C-04 runtime behavior.
- Regression: C-05 preserved command behavior.
- Integration: outbound provider-mode file request shaping test(s).
- Mutation: `cargo mutants --in-diff` for changed files.
