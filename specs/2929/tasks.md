# Tasks: Issue #2929 - Refactor gateway_openresponses into maintainable submodules

1. [x] T1 (RED): capture baseline oversized-module signals (`wc -l`, exemption artifact, current route wiring).
2. [x] T2 (GREEN): extract dashboard handlers into `gateway_openresponses/dashboard_runtime.rs` and keep router behavior unchanged.
3. [x] T3 (GREEN): extract memory handlers/helpers into `gateway_openresponses/memory_runtime.rs` and keep behavior unchanged.
4. [x] T4 (GREEN): update `tasks/policies/oversized-file-exemptions.json` to reflect reduced module size.
5. [x] T5 (REGRESSION): run targeted gateway regression tests for dashboard/memory behavior.
6. [x] T6 (VERIFY): run fmt/clippy/scoped tests and record updated line-count + policy checks.
