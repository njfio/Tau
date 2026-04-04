# Tasks: Issue #3702 - Collapse scaffold separator whitespace when deleting active placeholders in `tau-tui` command palette

- [x] T1. Write RED tests for active-placeholder `Alt+D` and `Ctrl+W` deletion
      spacing cleanup in the command palette.
- [x] T2. Update the active-placeholder word-deletion helpers so removing a
      scaffold placeholder also normalizes redundant separator whitespace.
- [x] T3. Verify active-placeholder deletion spacing cleanup and confirm earlier
      M335 REPL tests still pass.
