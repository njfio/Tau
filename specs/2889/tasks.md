# Tasks: Issue #2889 - session reset confirmation and clear-session contracts

1. [x] T1 (RED): add failing `functional_spec_2889_*` UI tests for session detail reset-confirmation form contract markers.
2. [x] T2 (RED): add failing `functional_spec_2889_*` + `integration_spec_2889_*` gateway tests for reset action, redirect contracts, cleared detail view, and non-target isolation.
3. [x] T3 (RED): add failing UI/gateway assertions for the missing browser-native session confirmation guard.
4. [x] T4 (GREEN): implement reset form SSR markers, scoped browser confirmation guard, and ops reset POST handler on session detail route.
5. [x] T5 (REGRESSION): rerun `spec_2889` and adjacent session/action confirmation suites.
6. [x] T6 (VERIFY): run fmt/clippy/scoped tests + live browser validation.
