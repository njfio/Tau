# Tasks: Issue #3692 - Add placeholder-aware editing to `tau-tui` command palette scaffolds

- [x] T1. Write RED tests for placeholder jump, replacement, and clearing
      behavior in the command palette.
- [x] T2. Extend command-palette state so scaffold placeholders can become the
      active edit target after autocomplete.
- [x] T3. Make typing and backspace operate on the active placeholder span while
      preserving existing guardrails and palette browsing behavior.
- [x] T4. Verify placeholder-aware editing and confirm earlier M335 REPL tests
      still pass.
