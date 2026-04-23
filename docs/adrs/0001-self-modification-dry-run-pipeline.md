# ADR 0001: Self-modification dry-run pipeline (library + standalone bin)

- **Status**: Accepted
- **Date**: 2026-04-23
- **Deciders**: Gyre SE agent + user

## Context

The `self_modification_runtime` module in `tau-coding-agent` carries configuration,
classification, worktree-containment, and proposal-id validation primitives, but
has zero production callers. The file was flagged `#![allow(dead_code)]`. The
broader harness claim of "autonomous self-improving agent" is therefore
aspirational — no wiring drives the primitives end-to-end.

Three forcing constraints apply:

1. The top-level CLI is defined in a separate crate (`tau-cli`) and parsing is
   shared by many binaries. Adding a new top-level subcommand there touches many
   unrelated modules.
2. The full productionized pipeline (LLM-driven proposal synthesis, safety eval,
   worktree apply, test run, git commit, rollback) is a multi-turn effort. A
   minimum viable slice needs a clean seam so later stages can plug in.
3. The primitives are already hardened (stage `harden-self-mod-path`); the gap
   is purely one of call-site integration.

## Decision

Introduce two artefacts in `tau-coding-agent`:

1. A new library module `self_modification_pipeline` that exposes a single
   `run_dry_run_pipeline(config, target, override_id) -> anyhow::Result<SelfModificationResult>`
   function. This is the seam — any future driver (CLI subcommand, LLM tool
   handler, HTTP route, orchestrator callback) calls this function.
2. A standalone binary target `self-mod-dry-run` (file:
   `src/bin/self_mod_dry_run.rs`) that wires the pipeline to a small clap-based
   argv parser and tracing subscriber, producing a JSON result on stdout.

Remove `#![allow(dead_code)]` from `self_modification_runtime.rs`; the module's
functions are now called by the pipeline.

## Consequences

### Positive
- The self-modification primitives now have a real caller; removing the
  dead-code pragma is observable proof of wiring.
- A stable library seam exists for future stages to plug actual apply / LLM
  / safety logic into — the shape `run_pipeline(config, target, ...) -> Result<R>`
  does not change when those get filled in.
- The standalone bin does not touch `tau-cli` and therefore has zero blast
  radius on unrelated command parsing.
- Dry-run is the safest default; users opt into apply via a later stage.

### Negative
- Two binaries produced by `tau-coding-agent` instead of one — adds a target to
  the build matrix and a name to worry about (`self-mod-dry-run`).
- The seam is slightly awkward: a future top-level subcommand will eventually
  want to call the same pipeline, meaning there will be two call sites to
  maintain until the standalone bin is retired.

### Neutral
- The pipeline is synchronous (blocking I/O via `std::fs`) because the
  hardened primitives are sync. When an async apply step is added later the
  pipeline signature may need to become `async fn`; that is a forward-looking
  migration, not a regression.

## Alternatives considered

1. **Add a top-level subcommand via `tau-cli`.** Rejected for this stage because
   `tau-cli` is shared by many binaries and subcommand dispatch ripples into
   `startup_dispatch`, `commands`, and multiple test harnesses. Higher risk
   for an MVP slice. Can be added later once the pipeline signature is stable.
2. **Wire directly into the `runtime_loop` / agent turn loop.** Rejected:
   coupling the self-modification pipeline to the agent's per-turn loop
   before the pipeline itself has been exercised on its own is the classic
   "two unfinished things glued together" failure mode. Better to stand the
   pipeline up in isolation first.
3. **Delete the module.** Rejected per user direction — the primitives are
   clearly future-load-bearing and deleting them would force re-implementation
   in a later stage.
4. **Expose only a library fn, no binary.** Rejected because then the wiring
   claim is only verifiable via tests — no operator-runnable proof. The bin
   target is cheap and gives a concrete demonstration.

## References
- `.gyre/requirements/wire-self-mod-runtime.md`
- `docs/solutions/patterns/self-modification-worktree-containment.md` (prior stage)
- `crates/tau-coding-agent/src/self_modification_runtime.rs`
