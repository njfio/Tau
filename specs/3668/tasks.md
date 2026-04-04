# Tasks: Issue #3668 - Isolate CLI provider backends from repo context bleed

- [x] T1 (RED): add provider adapter regressions that prove the subprocess cwd
      is isolated from the repository working tree.
- [x] T2 (GREEN): run codex / claude / gemini CLI provider subprocesses from an
      ephemeral temp directory.
- [x] T3 (VERIFY): rerun the scoped `tau-provider` verification stack.

## Tier Mapping
- Functional: textual tool-call promotion still works through the isolated cwd
- Regression: CLI subprocess cwd is no longer the repository cwd
