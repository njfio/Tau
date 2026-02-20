# Tasks: Issue #2927 - Panic/unsafe audit with guardrail enforcement

1. [x] T1 (RED): capture baseline signals showing panic/unsafe growth risk and absence of deterministic guardrail report.
2. [x] T2 (GREEN): implement deterministic audit tooling + baseline policy artifacts.
3. [x] T3 (GREEN): review/remediate production-facing occurrences or document justified exceptions.
4. [x] T4 (REGRESSION): rerun no-regression quality gates (`cargo test -p tau-cli`) after tooling changes.
5. [x] T5 (VERIFY): run fmt/clippy/audit guardrail + sanitized live validation (`scripts/dev/provider-live-smoke.sh`).
