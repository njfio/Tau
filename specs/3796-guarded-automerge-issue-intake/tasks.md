# Tasks: Guarded Auto-Merge And Issue Intake

GitHub Issue: #3796
Status: Implemented

- [x] T1 (RED): Add failing runtime tests for merge success, merge blocked,
  no bypass flags, and no-authority issue intake.
- [x] T2 (GREEN): Implement auto-merge policy/auth/PR URL gates and persisted
  merge evidence.
- [x] T3 (GREEN): Implement durable issue intake authority plans without repo
  mutation.
- [x] T4 (GREEN): Add CLI commands and disposable integration proof.
- [x] T5 (VERIFY): Run focused tests, scripts, fmt, diff check, and clippy.

## Test Tiers

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | Done | `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` | |
| Property | N/A | | No randomized parser or invariant introduced |
| Contract/DbC | N/A | | No formal contract macro surface changed |
| Snapshot | N/A | | JSON status uses field assertions |
| Functional | Done | `spec_3796_c01_auto_merge_requests_gh_auto_merge_when_authorized`; `spec_3796_c04_issue_intake_without_authority_persists_blocked_plan` | |
| Conformance | Done | `scripts/dev/test-autonomous-coding-automerge-intake.sh` asserts `--auto`, merge method, and no `--admin` | |
| Integration | Done | `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target scripts/dev/test-autonomous-coding-automerge-intake.sh`; `scripts/dev/test-autonomous-coding-job-loop.sh` | |
| Fuzz | N/A | | No untrusted parser/fuzz target changed |
| Mutation | N/A | | Follow-up for release gate; focused regression coverage in this slice |
| Regression | Done | `spec_3796_c02_auto_merge_blocks_without_policy_or_pr_url`; existing autonomous job loop proof | |
| Performance | N/A | | No hot path or benchmarked runtime changed |

## Verification Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` failed before implementation because `AutonomousCodingAutoMergeRequest`, `AutonomousCodingMergeMethod`, `AutonomousCodingAutoMergeStatus`, `AutonomousCodingIssueIntakeRequest`, `AutonomousCodingIssueIntakeStatus`, `request_auto_merge`, `intake_issue_without_authority`, and `autonomous_coding_issue_intake_path` were missing.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passed: 7 tests.
- CLI: `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target cargo test -p tau-coding-agent --bin tau_autonomous_coding_job -- --test-threads=1` passed.
- Integration: `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target scripts/dev/test-autonomous-coding-automerge-intake.sh` passed with `autonomous_coding_automerge_intake=pass`.
- Regression: `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target scripts/dev/test-autonomous-coding-job-loop.sh` passed with `autonomous_coding_job_loop=pass`.
- Lint: `cargo fmt --check`, `git diff --check`, `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target cargo clippy -p tau-runtime --lib --tests -- -D warnings`, and `CARGO_TARGET_DIR=/tmp/rust_pi-3796-target cargo clippy -p tau-coding-agent --bin tau_autonomous_coding_job -- -D warnings` passed.
