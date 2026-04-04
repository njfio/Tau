# Tasks: Issue #3700 - Add placeholder-aware Left/Right cursor escape to `tau-tui` command palette

- [x] T1. Write RED tests for active-placeholder `Left` and `Right` escape
      behavior in the command palette.
- [x] T2. Update the command-palette cursor helpers so active placeholders are
      escaped atomically before ordinary character-wise movement resumes.
- [x] T3. Verify placeholder-aware cursor escape and confirm earlier M335 REPL
      tests still pass.
