# Tasks: Issue #3704 - Preserve active placeholder focus across Ctrl+U/Ctrl+K in `tau-tui` command palette

- [x] T1. Write RED tests for active-placeholder `Ctrl+U` and `Ctrl+K`
      focus preservation in the command palette.
- [x] T2. Update the active-placeholder line-clearing helpers so the current
      scaffold field remains focused after clearing the prefix or suffix.
- [x] T3. Verify active-placeholder line-clearing focus preservation and
      confirm earlier M335 REPL tests still pass.
