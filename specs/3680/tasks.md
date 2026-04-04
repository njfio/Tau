# Tasks: Issue #3680 - Add local session persistence to `tau-tui` REPL

- [x] T1. Write RED tests for local session restore, save, and invalid-state
      fallback.
- [x] T2. Implement file-backed local session-state load/save for interactive
      `tau-tui`.
- [x] T3. Wire the persistence snapshot into startup, interaction, and shutdown
      without regressing existing gateway behavior.
- [x] T4. Verify the persistence slice and confirm earlier M335 REPL tests still
      pass.
