# Tasks: Issue #2826 - Command-center control confirmation SSR markers

## Ordered Tasks
1. [x] T1 (RED): add failing UI + gateway tests for control confirmation marker contracts.
2. [x] T2 (GREEN): add deterministic confirmation marker attributes to pause/resume/refresh buttons.
3. [x] T3 (REGRESSION): run phase-1A..1K command-center regression suites.
4. [x] T4 (VERIFY): run fmt/clippy/tests/mutation/guardrails and set spec status to `Implemented`.

## Tier Mapping
- Unit: N/A.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: control confirmation marker assertions.
- Conformance: C-01..C-04.
- Integration: gateway `/ops` render marker assertions.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-dashboard-ui -p tau-gateway`.
- Regression: phase-1A..1K contract suites.
- Performance: N/A.

## Verification Evidence
- RED:
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1` (failed on missing confirmation markers)
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1` (failed on missing confirmation markers)
- GREEN + regression:
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2786 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2794 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2798 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2806 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2810 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2814 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2818 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2822 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2786 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2794 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2798 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2806 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2810 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2814 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2818 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2822 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  - `cargo test -p tau-dashboard-ui`
  - `cargo test -p tau-gateway`
  - `python3 .github/scripts/oversized_file_guard.py`
  - `cargo mutants --in-diff /tmp/mutants_2826.diff -p tau-dashboard-ui -p tau-gateway` (`2/2` caught)
  - `cargo test` (workspace run executed; failures observed in unrelated existing `tau-coding-agent` tests, none in touched crates)
