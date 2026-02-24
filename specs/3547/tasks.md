# Tasks: Issue #3547 - Full legacy mini-model reference purge

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED): add conformance guard script/test that fails while legacy model
   strings still exist.
2. [x] T2 (GREEN): replace legacy mini-model references across active source/docs/
   scripts/tests with GPT-5 equivalents.
3. [x] T3 (GREEN): update edge-case files manually where automated replacement is
   unsafe (codex-only model paths, historical notes).
4. [x] T4 (VERIFY): run focused tests/lint/script checks and capture AC evidence.
5. [x] T5 (DOC): mark spec/plan/tasks `Implemented` and publish issue closure
   summary.

## TDD Evidence
### RED
- `bash scripts/run/test-model-reference-policy.sh`
  - pre-implementation failure: `legacy model reference policy failed: found 397 match(es)`.

### GREEN
- `bash scripts/run/test-model-reference-policy.sh` passed.
- `bash scripts/run/test-tau-unified.sh` passed.
- `cargo test -p tau-provider` passed.
- `cargo test -p tau-onboarding` passed.
- `cargo test -p tau-gateway` passed.
- `cargo test -p tau-coding-agent --test cli_integration auth_provider` passed.
- `cargo test -p tau-tui` passed.
- `cargo test -p tau-cli --lib` passed.

### REGRESSION
- `cargo test --workspace --no-run` passed.
- `cargo fmt --check` passed.
- `cargo clippy -p tau-provider -p tau-coding-agent -p tau-onboarding -p tau-gateway -p tau-tui -p tau-cli --all-targets -- -D warnings` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-provider`; `cargo test -p tau-onboarding`; `cargo test -p tau-gateway`; `cargo test -p tau-tui`; `cargo test -p tau-cli --lib` |  |
| Property | N/A |  | No invariant/property logic introduced |
| Contract/DbC | N/A |  | No contract macro surfaces changed |
| Snapshot | N/A |  | No snapshot suites added/changed |
| Functional | ✅ | `bash scripts/run/test-tau-unified.sh`; `cargo test -p tau-coding-agent --test cli_integration auth_provider` |  |
| Conformance | ✅ | `bash scripts/run/test-model-reference-policy.sh` |  |
| Integration | ✅ | `cargo test -p tau-gateway`; `cargo test -p tau-onboarding`; `cargo test -p tau-coding-agent --test cli_integration auth_provider` |  |
| Fuzz | N/A |  | No untrusted parser/decoder surface changes |
| Mutation | N/A |  | String/config consistency scope; no critical algorithm path changes |
| Regression | ✅ | RED->GREEN guard progression; `cargo test --workspace --no-run`; targeted crate suites above |  |
| Performance | N/A |  | No hotspot or throughput behavior changes |
