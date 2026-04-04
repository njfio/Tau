# Tasks: Issue #3681 - Add local transcript persistence to `tau-tui` REPL

- [x] T1. Write RED tests for transcript restore, save, invalid-state fallback,
      and welcome-banner deduplication.
- [x] T2. Implement file-backed local transcript-state load/save for
      interactive `tau-tui`.
- [x] T3. Wire transcript restore/persist into startup, transcript mutations,
      and shutdown without regressing gateway behavior.
- [x] T4. Verify the transcript persistence slice and confirm earlier M335 REPL
      tests still pass.
