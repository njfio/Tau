# Tasks: Issue #3740 - Restore tau-provider cli_executable clippy hygiene

- [x] T1 (RED): confirm the `tau-provider` clippy lane fails on dead code in `cli_executable.rs`.
- [x] T2 (GREEN): remove the unused helper symbols without changing executable-discovery behavior.
- [x] T3 (VERIFY): rerun targeted clippy, tests, and formatting.
