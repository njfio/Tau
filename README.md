# Tau

Tau is a Rust-first agent runtime and operator control plane for running model-driven workflows, persistent sessions, gateway APIs, and multi-channel automations with explicit operational guardrails.

## What Tau Is

Tau combines:
- a primary CLI runtime (`tau-coding-agent`) for interactive and one-shot execution,
- gateway and dashboard surfaces for operators,
- tool, memory, and safety policy controls,
- deterministic demos and validation scripts for local and CI workflows.

The workspace is intentionally multi-crate and contract-driven. See full membership in [`Cargo.toml`](Cargo.toml).

## Who Tau Is For

- Operators who need repeatable runtime controls, rollout/rollback runbooks, and diagnostics.
- Integrators who need OpenAI-compatible gateway routes and transport bridges.
- Contributors working in a spec-driven, TDD-oriented Rust workspace.

## What You Can Do Today

- Run interactive and one-shot agent flows from `tau-coding-agent`.
- Use session persistence and lifecycle operations (branching, resume, export/import/repair).
- Route model calls across multiple provider/auth modes.
- Run gateway API surfaces and dashboard ops routes.
- Use built-in tools with policy controls (filesystem/shell/http/path/rate/sandbox).
- Run channel and bridge runtimes (GitHub Issues, Slack, Discord, Telegram/WhatsApp pipelines).
- Operate prompt-optimization workflows with SQLite-backed state and optional proxy attribution.
- Execute deterministic demo suites and validation scripts in local/CI loops.

## Capability Boundaries

Tau includes some surfaces as diagnostics-first or fixture/live-input flows rather than end-user UX products:

- Voice:
  - live/contract runners are available,
  - fixture/file-driven inputs are supported,
  - built-in microphone capture UX is not bundled in this repository.
- Browser automation:
  - live runner exists,
  - execution depends on an external Playwright-compatible CLI,
  - no embedded browser engine is shipped in Tau.
- Dashboard/custom-command/memory contract runners:
  - older contract-runner flags were removed from active dispatch,
  - current behavior is routed through runtime/gateway diagnostics and operations guides.
- Training:
  - canonical CLI training mode is prompt optimization,
  - true-RL building blocks (for example PPO/GAE components and proof scripts) exist in-repo, but are staged separately from the canonical prompt-optimization path.

## 5-Minute Quickstart

Run commands from repo root.

1. Prerequisites

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

5. Optional: run the TUI smoke demo

```bash
cargo run -p tau-tui -- --frames 2 --sleep-ms 0 --width 56 --no-color
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

Deterministic demo index:

```bash
./scripts/demo/index.sh --list
./scripts/demo/index.sh --only onboarding,gateway-auth,gateway-remote-access --fail-fast
```

Run all demo wrappers:

```bash
./scripts/demo/all.sh --list
./scripts/demo/all.sh --only local,rpc,events --fail-fast
```

Clean generated local artifacts:

```bash
./scripts/dev/clean-local-artifacts.sh
```

## Workspace Feature Map

Core execution and orchestration:
- `crates/tau-coding-agent`
- `crates/tau-agent-core`
- `crates/tau-runtime`
- `crates/tau-orchestrator`

Gateway, dashboard, and ops:
- `crates/tau-gateway`
- `crates/tau-dashboard`
- `crates/tau-dashboard-ui`
- `crates/tau-ops`

Model/provider and tooling:
- `crates/tau-ai`
- `crates/tau-provider`
- `crates/tau-tools`
- `crates/tau-safety`

Sessions, memory, extensions:
- `crates/tau-session`
- `crates/tau-memory`
- `crates/tau-extensions`
- `crates/tau-skills`

Transports and channels:
- `crates/tau-github-issues-runtime`
- `crates/tau-slack-runtime`
- `crates/tau-discord-runtime`
- `crates/tau-multi-channel`

Training and algorithm primitives:
- `crates/tau-training-types`
- `crates/tau-training-store`
- `crates/tau-training-tracer`
- `crates/tau-training-runner`
- `crates/tau-training-proxy`
- `crates/tau-trainer`
- `crates/tau-algorithm`

## Docs by Role

Primary doc index: [`docs/README.md`](docs/README.md)

Operator guides:
- [`docs/guides/operator-deployment-guide.md`](docs/guides/operator-deployment-guide.md)
- [`docs/guides/operator-control-summary.md`](docs/guides/operator-control-summary.md)
- [`docs/guides/dashboard-ops.md`](docs/guides/dashboard-ops.md)
- [`docs/guides/gateway-ops.md`](docs/guides/gateway-ops.md)
- [`docs/guides/memory-ops.md`](docs/guides/memory-ops.md)

Integrator/API guides:
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

Planning/roadmap references:
- [`docs/planning/true-rl-roadmap-skeleton.md`](docs/planning/true-rl-roadmap-skeleton.md)
- [`docs/guides/roadmap-execution-index.md`](docs/guides/roadmap-execution-index.md)
- [`docs/guides/doc-density-scorecard.md`](docs/guides/doc-density-scorecard.md)

## Packaging and Release Artifacts

Local Docker smoke build:

```bash
./scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke
```

Release workflow and artifact details:
- [`.github/workflows/release.yml`](.github/workflows/release.yml)
- [`docs/guides/release-automation-ops.md`](docs/guides/release-automation-ops.md)
- GitHub Releases: <https://github.com/njfio/Tau/releases>

## Security and Contribution

- Security reporting policy: [`SECURITY.md`](SECURITY.md)
- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Issue/spec workflow contract: [`AGENTS.md`](AGENTS.md)

This repository expects issue-first, spec-driven, test-driven changes with explicit validation evidence.
