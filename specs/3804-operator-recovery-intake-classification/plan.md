# Plan 3804: Operator Recovery and Issue Intake Classification

## Approach

Keep the existing persisted job and intake schemas additive by adding serde
defaulted fields. Build operator classification from the already-persisted job
record, coding mission checkpoint, heartbeat, lease, and reason code. Expose one
new mutating command, `mark-blocked`, that clears the lease and appends an event
instead of asking operators to edit JSON by hand.

Issue intake classification remains deterministic and local. It does not call
providers or GitHub. It classifies unsafe/broad/vague inputs first, then missing
verifier, missing edit/provider authority, missing credentials, and finally
solvable.

## Affected Modules

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
  - Operator status fields and classifier.
  - `mark_job_blocked`.
  - Issue intake classification and required-authority shaping.
- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
  - `mark-blocked` CLI command.
- `scripts/run/tau-unified.sh`
  - New control-plane markers for operator state.
- `scripts/run/test-tau-unified.sh`
  - Status contract assertions for those markers.

## Risks and Mitigations

- Risk: new fields break older persisted status/intake JSON.
  - Mitigation: additive fields use serde defaults.
- Risk: issue classification overclaims readiness.
  - Mitigation: classifier is conservative and keeps mutation blocked unless
    verifier and edit/provider authority are present.
- Risk: recovery command lacks jobs-state-dir context.
  - Mitigation: expose the state-dir command now and keep richer command-center
    replay/recover UX as follow-up work.

## Verification

- Focused Rust tests:
  - `spec_3804_status_classifies_stale_lease_and_marks_blocked`
  - `spec_3796_c04_issue_intake_without_authority_persists_blocked_plan`
  - `spec_3801_c03_issue_to_merge_without_edit_authority_blocks_before_mutation`
- Shell contract:
  - `scripts/run/test-tau-unified.sh`
- Static checks:
  - `cargo fmt --check`
  - `cargo clippy -p tau-runtime --lib --tests -- -D warnings`
