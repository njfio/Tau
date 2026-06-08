# Tasks: Provider Verifier Repair Loop

GitHub Issue: #3798
Status: Implemented

- [x] T1 (RED): Add deterministic provider repair tests/script where first
  provider edit fails and second provider edit reaches PR-ready.
- [x] T2 (GREEN): Add bounded provider attempt loop and per-attempt report
  fields.
- [x] T3 (GREEN): Add repair prompt context from verifier stderr/stdout and git
  diff.
- [x] T4 (GREEN): Preserve malformed/invalid/exhausted repair fail-closed
  behavior.
- [x] T5 (VERIFY): Run focused tests, scripts, fmt, diff check, and clippy.

## Test Tiers

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | Done | `cargo test -p tau-coding-agent --bin tau_live_coding_loop_harness provider_backed -- --test-threads=1` | |
| Property | N/A | | No randomized parser or invariant introduced |
| Contract/DbC | N/A | | No formal contract macro surface changed |
| Snapshot | N/A | | JSON report fields asserted directly |
| Functional | Done | repair prompt render test; deterministic repair and exhausted report assertions | |
| Conformance | Done | `provider_attempts` report assertions in unit tests and shell proof | |
| Integration | Done | `scripts/dev/test-provider-verifier-repair-loop.sh`; opt-in live OpenRouter repair proof | |
| Fuzz | N/A | | No untrusted parser/fuzz target changed |
| Mutation | N/A | | Follow-up release gate; focused repair-loop regression coverage in this slice |
| Regression | Done | malformed provider output test; exhausted repair budget script path | |
| Performance | N/A | | Bounded attempts only; no hot path benchmarked |

## Verification Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3798-repair-loop-target scripts/dev/test-provider-verifier-repair-loop.sh`
  initially failed before implementation because
  `--provider-repair-attempts` was not a recognized argument.
- Unit: `CARGO_TARGET_DIR=/tmp/rust_pi-3798-repair-loop-target cargo test -p tau-coding-agent --bin tau_live_coding_loop_harness provider_backed -- --test-threads=1`
  passed with 7 tests.
- Deterministic integration:
  `CARGO_TARGET_DIR=/tmp/rust_pi-3798-repair-loop-target scripts/dev/test-provider-verifier-repair-loop.sh`
  passed successful repair and exhausted-budget fail-closed cases.
- Live provider repair:
  `TAU_LIVE_PROVIDER_REPAIR_PROOF=1 TAU_PROVIDER_PROOF_MODEL=openrouter/deepseek/deepseek-v4-flash TAU_PROVIDER_PROOF_REPORT_DIR=/tmp/tau-3798-repair-proof CARGO_TARGET_DIR=/tmp/rust_pi-3798-repair-loop-target scripts/dev/test-provider-verifier-repair-loop.sh`
  passed. The live report recorded attempt 1 as mock, attempt 2 as live
  OpenRouter, parsed repair context, `notes.txt,status.txt` edit paths, GREEN
  verifier rerun, commit hash, and manual PR-ready output.
