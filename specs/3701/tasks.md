# Tasks: Issue #3701 - Add placeholder-aware Alt+B/Alt+F word escape to `tau-tui` command palette

- [x] T1. Write RED tests for active-placeholder `Alt+B` and `Alt+F` word
      escape behavior in the command palette.
- [x] T2. Update the command-palette word-movement helpers so active
      placeholders are escaped atomically before ordinary word-wise movement
      resumes.
- [x] T3. Verify placeholder-aware word escape and confirm earlier M335 REPL
      tests still pass.
