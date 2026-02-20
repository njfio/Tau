# Spec: Issue #2927 - Panic/unsafe audit with guardrail enforcement

Status: Reviewed

## Problem Statement
Quality review signaled sharp growth in `panic!` usage and new `unsafe` usage. Without an auditable inventory and ratchet guardrails, unsafe/error-handling debt can regress silently.

## Scope
In scope:
- Produce auditable inventory for `panic!`/`unsafe` usage in repository crates.
- Classify findings into test-only vs production and document rationale.
- Add repeatable guardrail command(s) and baseline policy artifact for drift detection.
- Remove or remediate unjustified production occurrences discovered in this slice.

Out of scope:
- Large architectural refactors unrelated to panic/unsafe audit.
- CI workflow wiring changes (tracked separately if needed).

## Acceptance Criteria
### AC-1 Auditable panic/unsafe inventory exists
Given repository source,
when audit command is run,
then deterministic report artifacts are generated with total counts and file-level breakdowns.

### AC-2 Production-facing panic/unsafe usage is reviewed and justified
Given inventory results,
when classification is completed,
then all production-facing `panic!`/`unsafe` occurrences are either remediated or documented with explicit justification.

### AC-3 Guardrail ratchet command is deterministic
Given baseline policy artifact,
when guardrail command is run,
then it fails on unapproved growth and passes when counts remain within approved thresholds.

### AC-4 Existing behavior contracts stay green
Given audit/remediation changes,
when scoped specs/regression suites are rerun,
then existing conformance contracts remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | repo source | run audit command | report artifact with deterministic counts is produced |
| C-02 | AC-2 | Functional | inventory output | review/remediate set | production occurrences are justified/remediated in tracked artifact |
| C-03 | AC-3 | Functional | baseline + guardrail command | run guardrail | pass/fail behavior is deterministic on drift |
| C-04 | AC-4 | Regression | prior delivered specs | rerun selected suites | suites remain green |

## Success Metrics / Signals
- Audit command output includes panic/unsafe totals and per-file breakdown.
- Guardrail command returns non-zero on simulated increase and zero on baseline.
- `cargo fmt --check`, `cargo clippy -p tau-agent-core -p tau-gateway -p tau-skills -p tau-cli -- -D warnings`, and `cargo test -p tau-cli` pass.
- Sanitized live provider validation via `scripts/dev/provider-live-smoke.sh` passes for configured providers.
