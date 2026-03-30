# Spec: Issue #3630 - Split live_rl_runtime below oversized-file threshold

Status: Reviewed
Priority: P1
Milestone: M330
Parent: #3629

## Problem Statement
`crates/tau-coding-agent/src/live_rl_runtime.rs` is currently 4460 lines, which
exceeds the repo's default oversized-file threshold of 4000 lines. The
oversized-file guard now fails on the current base branch and on PRs that do
not touch the file, so the debt has become a shared delivery blocker rather
than a localized cleanup task.

The module also concentrates multiple live RL concerns in one file: runtime
configuration, rollout collection, PPO/APO update orchestration, span/report
emission, and supporting helper logic. This story reduces the file below the
policy threshold through domain-oriented source splits while preserving existing
runtime behavior and targeted test contracts.

## Scope
In scope:
- refactor `crates/tau-coding-agent/src/live_rl_runtime.rs` into smaller
  internal modules/files so the root file is `<= 4000` lines;
- keep public behavior and existing live RL runtime contracts unchanged;
- keep the oversized-file guard green without adding a new exemption entry for
  `live_rl_runtime.rs`;
- update internal module declarations/imports required by the split;
- rerun targeted `tau-coding-agent` runtime selectors and the oversized-file
  guard as verification.

Out of scope:
- new RL features, reward logic changes, or APO/PPO behavior changes;
- new dependencies;
- protocol/wire-format changes;
- broad trainer/runtime rearchitecture beyond what is needed to split the file;
- adding a temporary exemption instead of performing the split.

## Acceptance Criteria
### AC-1 Root live RL runtime file is under policy threshold
Given the default oversized-file threshold is 4000 lines,
when line count is measured for
`crates/tau-coding-agent/src/live_rl_runtime.rs`,
then the file is `<= 4000` lines without a new exemption.

### AC-2 Existing live RL runtime behavior remains stable
Given the runtime split is internal-only,
when targeted live RL runtime conformance/regression selectors are rerun,
then they remain green without semantic changes to the covered behaviors.

### AC-3 Oversized-file guard passes for the module
Given oversized-file policy inputs,
when `.github/scripts/oversized_file_guard.py` is run against repo policy
paths,
then it reports `issues=0` and does not require an exemption entry for
`crates/tau-coding-agent/src/live_rl_runtime.rs`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | current source tree | run `wc -l crates/tau-coding-agent/src/live_rl_runtime.rs` | reported line count is `<= 4000` |
| C-02 | AC-2 | Regression | targeted live RL runtime selectors | rerun selected `tau-coding-agent` tests | existing runtime behavior remains green |
| C-03 | AC-3 | Functional | repo policy inputs | run `.github/scripts/oversized_file_guard.py` with policy paths | `issues=0` and no new exemption is needed |

## Success Metrics / Observable Signals
- `wc -l crates/tau-coding-agent/src/live_rl_runtime.rs` reports `<= 4000`.
- `python3 .github/scripts/oversized_file_guard.py --repo-root . --exemptions-file tasks/policies/oversized-file-exemptions.json --policy-guide docs/guides/oversized-file-policy.md --json-output-file <path>` reports `issues=0`.
- Targeted `tau-coding-agent` live RL runtime selectors pass after the split,
  including at minimum:
  - `live_rl_runtime::tests::spec_c02_functional_optimizer_runs_on_update_interval`
  - `live_rl_runtime::tests::spec_c18_regression_live_curriculum_aggregates_persisted_to_resources`
  - `live_rl_runtime::tests::spec_c21_regression_live_apo_algorithm_failure_reports_deterministic_reason_code`
