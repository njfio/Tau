# Tasks: Issue #3703 - Collapse scaffold separator whitespace when deleting active placeholders with Backspace/Delete in `tau-tui` command palette

- [x] T1. Write RED tests for active-placeholder `Delete` and `Backspace`
      deletion spacing cleanup in the command palette.
- [x] T2. Update the active-placeholder character-deletion helpers so removing
      a scaffold placeholder also normalizes redundant separator whitespace.
- [x] T3. Verify active-placeholder character-deletion spacing cleanup and
      confirm earlier M335 REPL tests still pass.
