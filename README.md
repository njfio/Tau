# Tau

Tau is a Rust-native runtime for operating AI agents as real software systems:
model calls, tool use, session state, gateway APIs, background jobs, operator
controls, and verification evidence live in one workspace.

The product direction is straightforward: give Tau a task, the authority it is
allowed to use, and the verifier that decides whether the work is done. Tau then
runs the loop and returns either a verified result or a clear blocked reason with
the evidence needed to continue.

Today Tau is strongest as an operator-controlled agent runtime and autonomous
coding harness. It can run local and provider-backed coding loops, patch multiple
files, rerun verifiers, commit PR-ready work, expose runtime status, and preserve
run artifacts. Its issue-to-merge loop can run hands-off when verifier and edit
authority are supplied; without that authority, it blocks before mutation and
records what is missing.

## Why Tau Exists

Useful agents need more than a chat prompt. They need:

- controlled tools for reading, writing, editing, shelling out, and calling HTTP,
- durable state for sessions, jobs, memory, checkpoints, and resumes,
- provider/auth routing that can be tested without pretending every live account
  is always available,
- verifiers that turn "looks good" into RED/GREEN evidence,
- operator surfaces that show what is running, what changed, what failed, and
  what authority is missing.

Tau is the runtime layer for that work. The repo is intentionally multi-crate and
contract-driven; full workspace membership is in [`Cargo.toml`](Cargo.toml).

## What Works Today

- CLI and TUI agent sessions through `tau-coding-agent` and `tau-tui`.
- Persistent sessions, model/provider routing, memory surfaces, and tool-policy
  controls.
- Gateway APIs and operator routes for sessions, memory, jobs, routines,
  deploy/process state, and dashboard diagnostics.
- Built-in tools for policy-checked filesystem, shell, HTTP, read/write/edit,
  multi-file `write_many`, and surgical `edit_many` patches from exact strings
  or unified diffs.
- Autonomous coding mission loops that can branch a repo, run a RED verifier,
  apply controlled multi-file changes, rerun GREEN verification, commit, and
  produce PR-ready evidence.
- Provider-backed coding flows where a configured model supplies edits,
  verifier failures are fed back with stdout/stderr and git diff, and Tau reruns
  the repair loop within a bounded budget. OpenRouter-compatible repair is a
  built-in adapter that reads local env/`.env` configuration and stores
  sanitized provider-call metadata.
- Background autonomous coding job records with submit/run/replay/recover/status
  commands, operator recovery classification, `mark-blocked`, plus guarded
  GitHub auto-merge requests when explicit policy, GitHub auth, PR URL, and
  branch protections allow it.
- A one-command `issue-to-merge` loop that ingests issue context, runs a durable
  coding job, produces PR-ready or draft-PR evidence, and optionally requests
  protected-branch-safe auto-merge.
- Deterministic validation scripts for runtime claims, gateway/auth paths,
  operator maturity, dashboard contracts, TUI behavior, training workflows, and
  RL harness evidence.

## Autonomous Coding Status

Tau now has a real coding harness, not just isolated file generation.

What is landed:

- `CodingMissionRunner` can handle multi-file edits and resume from checkpoints.
- The real-repo harness runs against a temporary worktree of this repository and
  records RED/GREEN verifier output, commit hash, PR-ready body, and publication
  status.
- Provider-backed runs can use configured provider auth, including OpenRouter,
  and stores sanitized provider/model evidence.
- The verifier repair loop can take a failed verifier plus git diff, ask the
  provider for a targeted follow-up patch, rerun verification, and stop cleanly
  if the repair budget is exhausted.
- Prompt-mode agents can create multiple files with `write_many` and patch
  existing files with `edit_many`, including multi-file unified-diff hunks.
- Autonomous coding jobs can request normal GitHub auto-merge with `gh pr merge
  --auto` only after explicit policy/auth/PR-ready gates pass. Tau does not use
  admin override flags.
- `issue-to-merge` can run intake, job submission, verifier-gated execution,
  PR-ready or draft-PR publication, and optional auto-merge request in one
  command.
- Arbitrary issue intake without verifier/edit authority produces a durable
  blocked authority plan instead of mutating the repository. Intake now
  classifies blockers such as unsafe, too broad, underspecified, missing
  verifier, missing edit/provider authority, and missing credentials.

What is still product work:

- arbitrary issue selection and broad issue-to-PR autonomy with minimal human
  steering when no verifier/edit authority has been supplied,
- a polished command-center UX for durable stuck-job recovery, replay, and
  crash-resume across every coding path,
- arbitrary spec/issue-to-PR work with automatic PR opening when verifier and
  provider/edit authority are not supplied,
- large-scale production policy optimization for RL.

## Operator Experience

The clearest operator path is `tau-unified`:

```bash
./scripts/run/tau-unified.sh up --auth-mode localhost-dev
./scripts/run/tau-unified.sh status
./scripts/run/tau-unified.sh tui --no-color
./scripts/run/tau-unified.sh down
```

`status` emits stable `control_plane.*` markers for health, logs, runtime
artifacts, sessions, memory, jobs/routines, deploy/process state, active coding
missions, and durable autonomous coding jobs. These are visibility markers. They
do not, by themselves, mean every recovery and replay workflow is
product-polished.

## Evidence Map

Tau keeps product claims tied to executable checks. Start here when validating
the repo:

| Capability | Command | What It Proves |
| --- | --- | --- |
| Fast local validation | `./scripts/dev/fast-validate.sh` | Formatting/build-focused developer loop |
| Runtime claim boundary | `./scripts/dev/runtime-reality-gate.sh --output-json /tmp/tau-runtime-reality.json --output-md /tmp/tau-runtime-reality.md` | Deterministic product-claim check with explicit unsupported-claim boundaries |
| Unified operator runtime | `./scripts/dev/prove-tau-product.sh --check --report /tmp/tau-product-proof-check.json` | Static proof for the unified product path |
| Unified operator status | `scripts/run/test-tau-unified.sh` | Runtime status markers for jobs, provider repair, replay/recover state, stale leases, and mark-blocked command |
| Full local coding lifecycle | `./scripts/dev/test-full-autonomous-coding-loop.sh` | Branch, RED verifier, fix, GREEN verifier, commit, PR-ready bundle, blocked-state evidence |
| Real-repo coding harness | `./scripts/dev/test-real-repo-autonomous-coding-harness.sh` | Temporary worktree, multi-file edits, real git commit, PR-ready evidence |
| Provider-backed coding | `TAU_LIVE_PROVIDER_PROOF=1 ./scripts/dev/test-provider-backed-autonomous-coding-loop.sh` | Configured-provider edit supply with RED/GREEN verifier and commit evidence |
| Provider repair loop | `TAU_LIVE_PROVIDER_REPAIR_PROOF=1 ./scripts/dev/test-provider-verifier-repair-loop.sh` | Verifier failure -> targeted provider repair -> rerun -> PR-ready or blocked |
| Built-in OpenRouter repair adapter | `scripts/dev/test-openrouter-repair-adapter.sh` | OpenRouter-compatible JSON-mode provider call -> strict repair contract -> durable issue-to-merge verifier repair, with sanitized metadata |
| Guarded auto-merge/intake | `scripts/dev/test-autonomous-coding-automerge-intake.sh` | Protected-branch-safe auto-merge request gates and no-authority issue intake |
| Issue-to-merge orchestration | `scripts/dev/test-autonomous-coding-issue-to-merge.sh` | One-command issue intake -> durable job -> verifier -> draft PR -> guarded auto-merge, plus no-authority block |
| Autonomous coding gauntlet | `scripts/dev/test-autonomous-coding-gauntlet.sh` | Real fixture repos for provider full-file repair, unified diff repair, malformed provider block, missing verifier block, built-in OpenRouter-compatible repair, and safe auto-merge flags |
| Gateway auth/session | `./scripts/demo/gateway-auth-session.sh` | Gateway auth/session lifecycle smoke path |
| Operator maturity | `./scripts/verify/m295-operator-maturity-wave.sh` | TUI, RL, and auth maturity checks |

Provider-backed checks are intentionally opt-in because they depend on local
credentials, network access, and the configured model.

## Maturity Snapshot

| Area | Status | Plain Meaning |
| --- | --- | --- |
| CLI runtime, sessions, tools | Integrated | Usable local agent runtime with persistent state and policy-checked tools |
| Gateway APIs | Integrated | Auth/session routes and documented API contracts have deterministic coverage |
| TUI | Integrated | Operator shell, interactive agent mode, and live state-watch diagnostics exist |
| Multi-channel transports | Operational | Bridges exist with connector-specific maturity and live credential requirements |
| Prompt optimization/training | Integrated | Canonical training and rollout-state paths exist |
| True RL | Integrated harness, not production policy ops | Deterministic rollout/GAE/PPO evidence exists; large-scale promotion operations are still expanding |
| Dashboard/operator UX | Partial | Routes and diagnostics exist; polished command-center workflows are still being built |
| Unified control plane | Partial | `tau-unified status` exposes broad runtime and autonomous coding job visibility, including stale lease/replay/recover/mark-blocked markers; proactive recovery/replay UX is not complete |
| Autonomous coding | Partial but real | Controlled/provider-backed coding loops, durable provider repair, and authorized one-command issue-to-merge work; low-touch operation without supplied verifier/authority is intentionally blocked |

## Current Build Priorities

- Make the autonomous coding loop more durable: stuck-job recovery, replay,
  crash-resume, and operator status that are easy to trust.
- Harden provider-backed repair inside durable `issue-to-merge` jobs with larger
  real-repo gauntlets, broader live-provider adapters, and clearer blocked-state
  recovery.
- Keep collapsing scattered entrypoints into the `tau-unified` operator
  experience.
- Continue dashboard extraction and UX cleanup so operator routes feel like a
  command center instead of diagnostics stitched together.
- Grow RL and learning work through long-horizon evals, safety-constrained
  promotion, rollback drills, and statistically useful evidence.

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

6. Optional interactive TUI agent mode from runtime artifacts

```bash
cargo run -p tau-tui -- agent \
  --dashboard-state-dir .tau/dashboard \
  --gateway-state-dir .tau/gateway \
  --request-timeout-ms 45000 \
  --agent-request-max-retries 0 \
  --width 88 \
  --profile local-dev \
  --no-color
```

7. Optional live TUI watch mode (read-only, multi-cycle refresh)

```bash
cargo run -p tau-tui -- shell-live --state-dir .tau/dashboard --width 88 --profile local-dev --watch --iterations 3 --interval-ms 1000 --no-color
```

For a deeper walkthrough, use [`docs/guides/quickstart.md`](docs/guides/quickstart.md).

## Common Workflows

Fast local compile-focused loop:

```bash
./scripts/dev/fast-validate.sh --check-only --direct-packages-only --skip-fmt
```

Unified one-command runtime entrypoint:

```bash
./scripts/dev/prove-tau-product.sh --check
./scripts/dev/prove-tau-product.sh --check --report /tmp/tau-product-proof-check.json
./scripts/dev/prove-tau-product.sh --run
./scripts/dev/prove-tau-product.sh --run --report /tmp/tau-product-proof-run.json
./scripts/dev/prove-tau-product.sh --run --webchat-smoke --report /tmp/tau-product-proof-webchat.json
./scripts/run/tau-unified.sh up --auth-mode localhost-dev
./scripts/run/tau-unified.sh status
./scripts/run/tau-unified.sh tui --no-color
./scripts/run/tau-unified.sh tui --request-timeout-ms 90000 --agent-request-max-retries 1 --no-color
./scripts/run/tau-unified.sh tui --live-shell --iterations 3 --interval-ms 1000 --no-color
./scripts/run/tau-unified.sh down
```

`./scripts/run/tau-unified.sh status` emits grep-safe `control_plane.*`
markers for the active runtime. Treat these as operator visibility markers:
they show available endpoints and state files for health/logs/sessions/memory/
jobs/routines/deploy plus the latest autonomous coding job status, provider
repair evidence, PR state, event log, heartbeat, and lease. Durable replay,
stuck-job recovery, crash-resume, and production policy operations are still not
complete claims.

`tau-unified.sh tui` defaults to fast-fail interactive policy:
- `--request-timeout-ms 45000`
- `--agent-request-max-retries 0`

Override defaults with flags above or env vars:
- `TAU_UNIFIED_TUI_REQUEST_TIMEOUT_MS`
- `TAU_UNIFIED_TUI_AGENT_REQUEST_MAX_RETRIES`

Interactive TTY turns now emit progress markers to `stderr` while requests are
in-flight:
- `interactive.turn=start turn_timeout_ms=... request_timeout_ms=...`
- `interactive.turn=running elapsed_ms=...`
- `interactive.turn=end status=... elapsed_ms=...`

Full pre-merge gate:

```bash
./scripts/dev/fast-validate.sh --full
```

Interactive runtime mode:

```bash
cargo run -p tau-coding-agent -- --model openai/gpt-5.2
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

TUI interactive agent loop from runtime artifacts:

```bash
cargo run -p tau-tui -- agent \
  --dashboard-state-dir .tau/dashboard \
  --gateway-state-dir .tau/gateway \
  --request-timeout-ms 45000 \
  --agent-request-max-retries 0 \
  --profile local-dev \
  --no-color
```

TUI live watch loop from dashboard artifacts:

```bash
cargo run -p tau-tui -- shell-live --state-dir .tau/dashboard --profile local-dev --watch --iterations 3 --interval-ms 1000 --no-color
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
- `./examples/pacman-tetris`
- `./examples/pacman-tetris-ws`

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
