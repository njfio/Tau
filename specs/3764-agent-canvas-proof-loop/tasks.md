# Tasks: Issue #3764 - Agent Canvas proof loop stores comparison evidence

Status: Implemented

- [x] T1 (SPEC): create conflict-safe spec/plan/tasks for the Agent Canvas proof-loop follow-up.
- [x] T2 (RED): extend the proof-script shell test to require proof-loop comparison JSON.
- [x] T3 (GREEN): add before/fix/after/rerun comparison evidence to `ops-chat-canvas-proof.sh`.
- [x] T4 (VERIFY): run shell regression, static checks, and focused proof commands.

## Tier Mapping

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A | | shell proof automation; no Rust unit boundary changed |
| Property | N/A | | no randomized invariant introduced |
| Contract/DbC | N/A | | no `contracts` macro boundary changed |
| Snapshot | N/A | | explicit JSON assertions cover the proof payload |
| Functional | ✅ | `scripts/dev/test-ops-chat-canvas-proof.sh` | |
| Conformance | ✅ | `scripts/dev/test-ops-chat-canvas-proof.sh` proof-loop JSON assertions | |
| Integration | N/A | | no local credentialed gateway/tool provider was launched for this shell-only automation slice |
| Fuzz | N/A | | no parser/codec fuzz boundary changed |
| Mutation | N/A | | shell proof automation slice |
| Regression | ✅ | proof script fake-gateway regression | |
| Performance | N/A | | no runtime performance path changed |

## Verification Evidence

- RED: `scripts/dev/test-ops-chat-canvas-proof.sh` failed with
  `assertion failed (proof loop section): expected '"proof_loop": {'`, proving
  the previous JSON did not store loop comparison evidence.
- GREEN: `scripts/dev/test-ops-chat-canvas-proof.sh` passed after the script
  captured before/after artifact hashes, applied the deterministic fixed-canvas
  marker, reran the route contract check, and emitted comparison evidence.
- REGRESSION: the same shell test parses the output with `jq` and asserts
  `proof_loop.result = "passed"`, two iterations, `artifact_changed = true`,
  `targeted_fix_visible = true`, and `route_contract_stable = true`.
- STATIC: `bash -n scripts/dev/ops-chat-canvas-proof.sh
  scripts/dev/test-ops-chat-canvas-proof.sh` passed.
- STATIC: `git diff --check` passed.
- PROCESS: `scripts/dev/roadmap-status-sync.sh --check --quiet` passed after
  creating GitHub issue `#3764`.
