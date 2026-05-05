# Plan: Issue #3754 - Close Tau autonomous harness integration gaps

## Approach

1. Add a `tau-agent-core` deterministic mission harness module that converts
   benchmark tasks into completed `MissionSnapshot` proof objects.
2. Add M334 benchmark fixture loading and suite execution helpers that validate
   the intervention model and required proof surfaces.
3. Add a `tau-coding-agent` mission self-improvement adapter that calls the
   existing dry-run pipeline, maps the result into mission proposal evidence,
   and exposes approval-gated safe apply for skill/config/prompt files.
4. Add an operator-runnable `tau_agent_harness` binary that runs the canonical
   fixture and emits a JSON proof summary.
5. Verify with RED/GREEN unit/integration tests, the existing shell benchmark
   validator, scoped cargo tests, fmt, clippy, and PR checks.

## Affected Modules

- `crates/tau-agent-core/src/mission_harness.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-coding-agent/src/mission_self_improvement.rs`
- `crates/tau-coding-agent/src/bin/tau_agent_harness.rs`
- `crates/tau-coding-agent/src/lib.rs`
- `crates/tau-coding-agent/tests/mission_self_improvement.rs`
- `crates/tau-coding-agent/tests/harness_benchmark_bin.rs`
- `specs/3754/`
- `specs/milestones/m334/index.md`

## Interfaces

- `run_harness_mission(task, config) -> MissionHarnessTaskProof`
- `run_autonomy_benchmark_fixture(path, config) -> AutonomyBenchmarkProof`
- `record_self_modification_dry_run_on_mission(...)`
- `apply_approved_mission_improvement(...)`
- `tau_agent_harness --fixture <path> --output <path> --memory-root <path>`

## Risks / Mitigations

- Risk: deterministic proof is mistaken for live LLM autonomy.
  Mitigation: name the runner as a benchmark proof harness and record its
  deterministic execution mode in artifacts/metadata.
- Risk: self-improvement applies unsafe targets.
  Mitigation: reuse mission target-kind validation, runtime target
  classification, workspace containment checks, and tau-safety default rules.
- Risk: adding broad gateway wiring creates too much blast radius.
  Mitigation: first close the operator-runnable proof lane, then wire gateway
  adapters in a later slice if needed.

## Verification

- RED: cargo tests for missing harness runner, benchmark binary, and mission
  self-improvement adapter fail before implementation.
- GREEN: `cargo test -p tau-agent-core mission_harness --lib`
- GREEN: `cargo test -p tau-coding-agent --test mission_self_improvement`
- GREEN: `cargo test -p tau-coding-agent --test harness_benchmark_bin`
- Static: `cargo fmt --check -p tau-agent-core -p tau-coding-agent`
- Static: `cargo clippy -p tau-agent-core -p tau-coding-agent --all-targets --all-features -- -D warnings`
- Script: `scripts/dev/test-m334-tranche-one-autonomy-benchmark.sh`
