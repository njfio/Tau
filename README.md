# Tau

Tau is a Rust-first agent runtime and operator control plane with a connected core path:
CLI runtime -> sessions/tools/safety -> gateway APIs -> transport and operator workflows.

Short answer to the integration question: the core runtime path is integrated and runnable today, while some subsystems are still staged or partial.

## What Tau Is

Tau combines:
- a primary CLI runtime (`tau-coding-agent`) for interactive and one-shot execution,
- persistent session and tool-policy/safety controls,
- gateway and operator surfaces for API and operations workflows,
- deterministic demos and validation scripts for local and CI loops.

The workspace is intentionally multi-crate and contract-driven. Full crate membership is in [`Cargo.toml`](Cargo.toml).

## Who Tau Is For

- Operators who need repeatable runtime controls, diagnostics, and rollback-friendly workflows.
- Integrators who need OpenAI-compatible gateway routes and transport bridges.
- Contributors working in a spec-driven, TDD-oriented Rust workspace.

## Integrated End-to-End Paths

These are the paths that operate as connected flows today.

| Path | Start Point | Integrated Components | Primary Evidence |
|---|---|---|---|
| Local operator loop | `cargo run -p tau-coding-agent -- --onboard --onboard-non-interactive` then prompt mode | CLI runtime, agent core loop, sessions, tools, safety policies | [`docs/guides/quickstart.md`](docs/guides/quickstart.md), [`docs/guides/operator-control-summary.md`](docs/guides/operator-control-summary.md) |
| Gateway auth/session loop | `./scripts/demo/gateway-auth-session.sh` | Gateway auth/session handling, API route contracts, runtime policies | [`docs/guides/gateway-auth-session-smoke.md`](docs/guides/gateway-auth-session-smoke.md), [`docs/guides/gateway-api-reference.md`](docs/guides/gateway-api-reference.md) |
| Multi-channel ingress loop | `./scripts/demo/multi-channel.sh` | Multi-channel runtime, transport normalization, routing pipeline | [`docs/guides/multi-channel-event-pipeline.md`](docs/guides/multi-channel-event-pipeline.md), [`docs/guides/transports.md`](docs/guides/transports.md) |
| Prompt optimization loop | [`docs/guides/training-ops.md`](docs/guides/training-ops.md) runbook flow | Training runner/store/tracer/proxy + rollout controls | [`docs/guides/training-ops.md`](docs/guides/training-ops.md), [`docs/guides/training-proxy-ops.md`](docs/guides/training-proxy-ops.md) |
| Connected operator GA loop | `./scripts/verify/m296-ga-readiness-gate.sh` | RL maturity wave + auth/readiness checks + rollback trigger validation + closeout signoff criteria | [`docs/guides/m296-ga-readiness-gate.md`](docs/guides/m296-ga-readiness-gate.md), `artifacts/operator-ga-readiness/verification-report.json` |

## What You Can Do Today

- Run interactive and one-shot agent flows from `tau-coding-agent`.
- Use session persistence and lifecycle operations (branch, resume, export/import/repair).
- Route model calls across multiple provider/auth modes.
- Run gateway API surfaces and operator routes.
- Use built-in tools with policy controls (filesystem/shell/http/path/rate/sandbox).
- Run channel and bridge runtimes (GitHub Issues, Slack, Discord, Telegram/WhatsApp paths).
- Operate prompt-optimization workflows with SQLite-backed rollout state and optional proxy attribution.
- Execute deterministic demo suites and validation scripts in local/CI loops.

## Capability Boundaries

Some surfaces are intentionally diagnostics-first or staged:

- True RL:
  - deterministic end-to-end harness is available (`tau-trainer`),
  - broader production policy-optimization operating loops remain an expansion track.
- Dashboard:
  - route and diagnostics surfaces exist,
  - not all desired product UX workflows are fully integrated live-mutation paths.
- Auth verification:
  - provider auth-mode matrix and gateway auth/session lifecycle conformance are covered in deterministic suites (`scripts/verify/m295-operator-maturity-wave.sh`),
  - auth-depth lifecycle and edge-path coverage is aggregated in `scripts/verify/m303-auth-workflow-depth.sh`,
  - live third-party credential/network validation remains environment-specific.
- TUI:
  - includes operator-shell and `shell-live` modes with status/auth/training/alerts/actions panels,
  - `shell-live` now reports deterministic malformed/missing artifact diagnostics for operator triage,
  - remains complementary to (not a replacement for) web dashboard workflows.

## Maturity Matrix

| Capability Area | Status | Meaning | Primary Reference |
|---|---|---|---|
| Core CLI runtime + sessions + tools | Integrated | Production-like operating loop available | [`docs/guides/quickstart.md`](docs/guides/quickstart.md) |
| Gateway auth/session APIs | Integrated | Deterministic auth/session smoke flows and documented API contracts | [`docs/guides/gateway-auth-session-smoke.md`](docs/guides/gateway-auth-session-smoke.md) |
| Auth workflow conformance | Integrated | Provider matrix + gateway session lifecycle validated by dedicated suites | [`crates/tau-provider/tests/auth_workflow_conformance.rs`](crates/tau-provider/tests/auth_workflow_conformance.rs), [`scripts/verify/m295-operator-maturity-wave.sh`](scripts/verify/m295-operator-maturity-wave.sh) |
| Multi-channel and bridge transports | Operational | Runnable with connector-specific maturity differences | [`docs/guides/transports.md`](docs/guides/transports.md) |
| Dashboard operator UX | Partial | Ops routes and diagnostics available; broader UX still expanding | [`docs/guides/dashboard-ops.md`](docs/guides/dashboard-ops.md) |
| Prompt optimization training | Integrated | Canonical training path today | [`docs/guides/training-ops.md`](docs/guides/training-ops.md) |
| True RL | Integrated | Deterministic end-to-end harness emits rollout + GAE/PPO artifact evidence | [`crates/tau-trainer/src/rl_e2e.rs`](crates/tau-trainer/src/rl_e2e.rs), [`crates/tau-trainer/src/bin/rl_e2e_harness.rs`](crates/tau-trainer/src/bin/rl_e2e_harness.rs) |
| TUI | Integrated | Operator-shell and state-backed `shell-live` modes with conformance-tested diagnostics | [`crates/tau-tui/src/main.rs`](crates/tau-tui/src/main.rs), [`crates/tau-tui/src/lib.rs`](crates/tau-tui/src/lib.rs), [`scripts/verify/m295-operator-maturity-wave.sh`](scripts/verify/m295-operator-maturity-wave.sh) |

## Current Gaps and Execution Plan

| Gap | Current State | Execution Plan Links |
|---|---|---|
| True RL productionization depth | deterministic end-to-end harness, promotion/rollback gate verification, and GA readiness gate are delivered; larger-scale policy operations still expanding | [`docs/planning/integration-gap-closure-plan.md`](docs/planning/integration-gap-closure-plan.md), [`docs/planning/true-rl-roadmap-skeleton.md`](docs/planning/true-rl-roadmap-skeleton.md), [`docs/guides/training-ops.md`](docs/guides/training-ops.md), [`scripts/verify/m301-rl-promotion-rollback-gate.sh`](scripts/verify/m301-rl-promotion-rollback-gate.sh), [`scripts/verify/m296-ga-readiness-gate.sh`](scripts/verify/m296-ga-readiness-gate.sh) |
| Dashboard maturity expansion | ops routes exist, richer workflow UX pending | [`docs/planning/integration-gap-closure-plan.md`](docs/planning/integration-gap-closure-plan.md), [`docs/guides/dashboard-ops.md`](docs/guides/dashboard-ops.md), [`docs/guides/operator-deployment-guide.md`](docs/guides/operator-deployment-guide.md) |
| Extended auth live-env verification | deterministic matrix/lifecycle conformance and auth-depth edge-path gating are delivered; external credential/live-env permutations remain environment-specific | [`docs/planning/integration-gap-closure-plan.md`](docs/planning/integration-gap-closure-plan.md), [`docs/provider-auth/provider-auth-capability-matrix.md`](docs/provider-auth/provider-auth-capability-matrix.md), [`docs/guides/gateway-auth-session-smoke.md`](docs/guides/gateway-auth-session-smoke.md), [`scripts/verify/m303-auth-workflow-depth.sh`](scripts/verify/m303-auth-workflow-depth.sh), [`scripts/verify/m296-live-auth-validation.sh`](scripts/verify/m296-live-auth-validation.sh) |
| TUI interaction depth | operator shell and resilient `shell-live` diagnostics are delivered; richer interactive workflows continue to evolve | [`docs/planning/integration-gap-closure-plan.md`](docs/planning/integration-gap-closure-plan.md), [`crates/tau-tui`](crates/tau-tui), [`docs/guides/demo-index.md`](docs/guides/demo-index.md) |

## 5-Minute Quickstart

Run commands from repository root.

1. Prerequisite

```bash
rustup default stable
```

2. Fast validation loop

```bash
./scripts/dev/fast-validate.sh
```

3. Initialize local Tau state

```bash
cargo run -p tau-coding-agent -- --onboard --onboard-non-interactive
```

4. Run your first prompt

```bash
cargo run -p tau-coding-agent -- --prompt "Summarize src/lib.rs"
```

5. Optional TUI operator shell

```bash
cargo run -p tau-tui -- shell --width 88 --profile local-dev --no-color
```

6. Optional live TUI shell from runtime artifacts

```bash
cargo run -p tau-tui -- shell-live --state-dir .tau/dashboard --width 88 --profile local-dev --no-color
```

For a deeper walkthrough, use [`docs/guides/quickstart.md`](docs/guides/quickstart.md).

## Common Workflows

Fast local compile-focused loop:

```bash
./scripts/dev/fast-validate.sh --check-only --direct-packages-only --skip-fmt
```

Full pre-merge gate:

```bash
./scripts/dev/fast-validate.sh --full
```

Interactive runtime mode:

```bash
cargo run -p tau-coding-agent -- --model openai/gpt-4o-mini
```

Gateway auth/session smoke:

```bash
./scripts/demo/gateway-auth-session.sh
```

Dashboard demo path:

```bash
./scripts/demo/dashboard.sh
```

Demo index and selective runs:

```bash
./scripts/demo/index.sh --list
./scripts/demo/index.sh --only onboarding,gateway-auth,gateway-remote-access --fail-fast
```

```bash
./scripts/demo/all.sh --list
./scripts/demo/all.sh --only local,rpc,events --fail-fast
```

RL end-to-end deterministic harness:

```bash
cargo run -p tau-trainer --bin rl_e2e_harness -- --run-id local --output-dir artifacts/rl-e2e --print-json
```

Operator maturity wave verification (TUI + RL + auth):

```bash
./scripts/verify/m295-operator-maturity-wave.sh
```

M296 GA readiness gate (Connected operator GA loop):

```bash
./scripts/verify/m296-ga-readiness-gate.sh
```

Clean generated local artifacts:

```bash
./scripts/dev/clean-local-artifacts.sh
```

## Examples and Starter Assets

Checked-in example assets and starter package references:

- `./examples/starter/package.json`
- `./examples/extensions`
- `./examples/extensions/issue-assistant/extension.json`
- `./examples/extensions/issue-assistant/payload.json`
- `./examples/events`
- `./examples/events-state.json`

See `./examples/README.md` for package and asset walkthrough details.

## Docs by Role

Primary docs index: [`docs/README.md`](docs/README.md)

## Current Operator Surfaces

Operator deployment guide: `docs/guides/operator-deployment-guide.md`
Gateway API reference (70+ routes): `docs/guides/gateway-api-reference.md`
Contributor guide: `CONTRIBUTING.md`
Security policy: `SECURITY.md`

Operator runbooks:
- [`docs/guides/operator-deployment-guide.md`](docs/guides/operator-deployment-guide.md)
- [`docs/guides/operator-control-summary.md`](docs/guides/operator-control-summary.md)
- [`docs/guides/dashboard-ops.md`](docs/guides/dashboard-ops.md)
- [`docs/guides/gateway-ops.md`](docs/guides/gateway-ops.md)
- [`docs/guides/memory-ops.md`](docs/guides/memory-ops.md)
- [`docs/guides/m296-ga-readiness-gate.md`](docs/guides/m296-ga-readiness-gate.md)

Integrator/API references:
- [`docs/guides/gateway-api-reference.md`](docs/guides/gateway-api-reference.md)
- [`docs/guides/transports.md`](docs/guides/transports.md)
- [`docs/provider-auth/provider-auth-capability-matrix.md`](docs/provider-auth/provider-auth-capability-matrix.md)

Contributor references:
- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`AGENTS.md`](AGENTS.md)
- [`docs/tau-coding-agent/code-map.md`](docs/tau-coding-agent/code-map.md)
- [`docs/architecture/crate-dependency-diagram.md`](docs/architecture/crate-dependency-diagram.md)
- [`docs/guides/startup-di-pipeline.md`](docs/guides/startup-di-pipeline.md)
- [`docs/guides/contract-pattern-lifecycle.md`](docs/guides/contract-pattern-lifecycle.md)
- [`docs/guides/multi-channel-event-pipeline.md`](docs/guides/multi-channel-event-pipeline.md)
- [`docs/guides/doc-density-scorecard.md`](docs/guides/doc-density-scorecard.md)

Planning and gap closure:
- [`docs/planning/true-rl-roadmap-skeleton.md`](docs/planning/true-rl-roadmap-skeleton.md)
- [`docs/guides/roadmap-execution-index.md`](docs/guides/roadmap-execution-index.md)
- [`docs/planning/integration-gap-closure-plan.md`](docs/planning/integration-gap-closure-plan.md)

## Workspace Feature Map

Core runtime:
- `crates/tau-coding-agent`
- `crates/tau-agent-core`
- `crates/tau-runtime`
- `crates/tau-orchestrator`

Gateway and ops:
- `crates/tau-gateway`
- `crates/tau-dashboard`
- `crates/tau-dashboard-ui`
- `crates/tau-ops`

Model and policy:
- `crates/tau-ai`
- `crates/tau-provider`
- `crates/tau-tools`
- `crates/tau-safety`

State and extension surfaces:
- `crates/tau-session`
- `crates/tau-memory`
- `crates/tau-extensions`
- `crates/tau-skills`

Transport/bridge runtimes:
- `crates/tau-github-issues-runtime`
- `crates/tau-slack-runtime`
- `crates/tau-discord-runtime`
- `crates/tau-multi-channel`

Training and algorithms:
- `crates/tau-training-types`
- `crates/tau-training-store`
- `crates/tau-training-tracer`
- `crates/tau-training-runner`
- `crates/tau-training-proxy`
- `crates/tau-trainer`
- `crates/tau-algorithm`

## Packaging and Release Artifacts

Local Docker smoke build:

```bash
./scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke
```

Release workflow and artifacts:
- [`.github/workflows/release.yml`](.github/workflows/release.yml)
- [`docs/guides/release-automation-ops.md`](docs/guides/release-automation-ops.md)
- GitHub Releases: <https://github.com/njfio/Tau/releases>

## Security and Contribution

- Security reporting policy: [`SECURITY.md`](SECURITY.md)
- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Issue/spec workflow contract: [`AGENTS.md`](AGENTS.md)

This repository expects issue-first, spec-driven, test-driven changes with explicit validation evidence.
