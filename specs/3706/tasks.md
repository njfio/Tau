# Tasks: Issue #3706 - Make active scaffold placeholders atomic for `Ctrl+A`/`Ctrl+E` in `tau-tui` command palette

- [x] T1. Write RED tests for active-placeholder `Ctrl+A` and `Ctrl+E`
      line-boundary escape behavior in the command palette.
- [x] T2. Update the command-palette start/end movement helpers so active
      placeholders escape to their own boundaries before ordinary absolute line
      movement runs.
- [x] T3. Verify placeholder-aware `Ctrl+A`/`Ctrl+E` movement and confirm
      earlier M335 REPL tests still pass.
