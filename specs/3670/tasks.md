# Tasks: Issue #3670 - Recover read-only gateway timeouts and clear in-flight tools

- [x] T1 (RED): add gateway regressions for read-only timeout recovery and
      pending-tool timeout cleanup.
- [x] T2 (GREEN): finalize pending tool starts on timeout/error and route
      recoverable read-only timeouts into the bounded outer retry path.
- [x] T3 (VERIFY): rerun the scoped gateway verification stack.
