# Tasks: Issue #3731 - Hoist action history I/O out of per-attempt loop

- [x] T1 (RED): add unit coverage for in-memory record append without disk I/O.
- [x] T2 (RED): add unit coverage for in-memory bulletin rendering parity with
      the existing disk-backed helper.
- [x] T3 (GREEN): add in-memory action-history append/render helpers in
      `learning_runtime.rs`.
- [x] T4 (GREEN): load the action-history store once in
      `openresponses_execution_handler.rs`, reuse it through the retry loop, and
      save it once after the loop exits.
- [x] T5 (VERIFY): run targeted `tau-gateway` tests plus `cargo check -p
      tau-gateway` and `cargo fmt --check`.
