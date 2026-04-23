# Changelog

All notable changes to this repository are documented in this file.

The format is inspired by Keep a Changelog and follows milestone-based delivery evidence in `specs/`.

## [Unreleased] - 2026-02-19

### Security
- **wasmtime 25.0.3 → 36** across the workspace, clearing RUSTSEC-2026-0096 (CVSS 9.0, aarch64 guest heap miscompile). No API breaks on the `Config/Engine/Module/Store/Linker/Memory` surface used by `tau-runtime::wasm_sandbox_runtime`. See [docs/solutions/patterns/wasmtime-25-to-36-upgrade-no-api-breaks.md](docs/solutions/patterns/wasmtime-25-to-36-upgrade-no-api-breaks.md).
- Hardened `self_modification_runtime::cleanup_self_mod_worktree`: canonicalize-with-parent-fallback containment check, proposal-id ASCII allowlist with `..` rejection, segment-based `classify_modification_target`. See [docs/solutions/patterns/self-modification-worktree-containment.md](docs/solutions/patterns/self-modification-worktree-containment.md).
- **Exited the vulnerable `rustls-webpki 0.102.8` transitive chain** by switching `serenity` from `rustls_backend` to `native_tls_backend`. Clears four advisories (RUSTSEC-2026-0049, -0098, -0099, -0104) locked in by `serenity 0.12.5` (latest crates.io release). `tau-discord-runtime` now uses the OS TLS implementation (SecureTransport / Schannel / OpenSSL). No public-API impact; end-to-end tests green. Revisit when serenity upstream bumps its rustls dependency.
- Added `.cargo/audit.toml` with per-advisory ignore rationale for four non-exploitable unmaintained/unsound-transitive warnings (RUSTSEC-2025-0057 `fxhash`, RUSTSEC-2024-0384 `instant`, RUSTSEC-2024-0436 `paste`, RUSTSEC-2026-0002 `lru` via ratatui). `cargo audit` now reports zero vulnerabilities and zero warnings. Each entry carries the upstream path and review trigger so they are not silent.

### Added
- `tau-coding-agent::self_modification_tool::SelfModificationProposeTool` — `AgentTool` implementation making the dry-run pipeline invokable autonomously via the standard tool-call path. **Gated by the `TAU_AUTONOMOUS_SELF_MOD=1` environment variable**; refuses with `reason_code: "autonomous_self_mod_disabled"` when unset. Never applies changes — always dry-run. Six unit tests cover env-gate on/off/"0", invalid-arguments rejection, allowed-skill happy path, and source-target policy denial.
- `tau-coding-agent::tools::register_builtin_tools` now wraps `tau_tools::tools::register_builtin_tools` and additionally registers `SelfModificationProposeTool` on every agent built through the coding-agent registry path (including the runtime in `events.rs`). Fail-closed remains in effect via the env gate; the tool is visible-but-refused so the model observes a structured error rather than silently missing capability.
- `ToolPolicy::self_modification_propose_enabled` (defaults to `false`) — defense-in-depth registration gate on top of the `TAU_AUTONOMOUS_SELF_MOD` runtime env gate. When `false`, the self-modification tool is not registered at all (model cannot see it); when `true`, the tool is advertised but still refuses to execute unless the env gate is also set. Two independent switches with distinct semantics: policy controls *visibility*, env controls *invocability*. Two new unit tests (`unit_register_builtin_tools_omits_self_modification_propose_by_default`, `unit_register_builtin_tools_includes_self_modification_propose_when_enabled`) pin the behavior.
- `tau-coding-agent::self_modification_synthesis_tool::SelfModificationSynthesizeTool` — the synthesis half of the autonomous self-modification pair. Takes a natural-language intent, calls the agent's `LlmClient` with a strict-JSON system prompt, validates the output (path traversal, absolute paths, ASCII allowlist, change_type enum, max-output-bytes), and returns a structured proposal `{ target, change_type, rationale, proposed_diff, policy_projected }`. The tool never applies changes and never invokes the dry-run pipeline — the LLM chooses whether to chain its output through `self_modification_propose`. **Four independent gates** now protect the autonomous path: policy flag (visibility), env gate (invocability), synthesis-time output validation (parser), and the dry-run boundary on the propose tool. Nine unit tests cover happy path, path-traversal rejection, absolute-path rejection, invalid change_type, malformed JSON, LLM transport error, markdown-fenced output tolerance, empty-intent rejection, and env-gate refusal.
- `ToolPolicy::self_modification_synthesize_enabled` (defaults to `false`) — registration gate for the synthesis tool; paired with `register_self_modification_synthesis` which takes the `LlmClient` + model explicitly because synthesis needs them at construction time. Two registry unit tests pin the gate behavior (`unit_register_self_modification_synthesis_omits_tool_by_default`, `unit_register_self_modification_synthesis_includes_tool_when_enabled`).
- Crate-level test-only `AUTONOMOUS_SELF_MOD_ENV_LOCK` in `tau-coding-agent::lib` — shared across every test module that mutates the env var, so parallel tests in different modules cannot interleave env mutations and observe a torn state.
- `tau-coding-agent`: new operator binary `self-mod-dry-run` exercising the self-modification dry-run pipeline end-to-end. JSON-on-stdout contract; `tracing` on stderr via `TAU_SELF_MOD_LOG`.
- `tau-coding-agent::self_modification_pipeline` module wiring validate → classify → policy → worktree → cleanup with structured `tracing` events. Previously the self-modification runtime had zero production call sites; now exposed via the operator bin and a minimal `tau_coding_agent` library seam.
- ADR: [docs/adrs/0001-self-modification-dry-run-pipeline.md](docs/adrs/0001-self-modification-dry-run-pipeline.md) records the library-seam + standalone-bin decision.
- Integration test [crates/tau-coding-agent/tests/self_mod_dry_run_bin.rs](crates/tau-coding-agent/tests/self_mod_dry_run_bin.rs) exercises the operator bin argv → JSON contract with three adversarial cases (allowed-skill, denied-source, hostile-proposal-id).
- Integration test [crates/tau-coding-agent/tests/synthesize_then_propose_chain.rs](crates/tau-coding-agent/tests/synthesize_then_propose_chain.rs) — end-to-end composability check: fake `LlmClient` → `SelfModificationSynthesizeTool` → `SelfModificationProposeTool` in one process. Covers (a) skill-target happy path flowing synthesis output unchanged into propose, and (b) source-target path where synthesis projects `would_be_blocked` and propose correctly returns a structured `auto_apply_source_disabled` denial. Closes the "each tool works in isolation but are they composable?" gap.
- `tau-agent-core::agent_channel`: `tracing` instrumentation on message drops (subscriber-count tracking + structured `from_agent_id`/`message_type` fields).

### Changed
- `tau-coding-agent`: dropped `#![allow(dead_code)]` from `self_modification_runtime`; all public surface now has live callers.
- `tau-coding-agent::self-mod-dry-run`: `tracing` subscriber emits to stderr (not stdout) and defaults to silent — preserves the stdout JSON contract for downstream consumers.
- `tau-memory::runtime`: extracted `defaults` (serde default functions, 81 lines), `normalize` (scope/entry/relation/sqlite-int normalization plus `current_unix_timestamp_ms`, 153 lines), and `file_store` (the `FileMemoryStore` struct + ~380-line impl block, 409 lines total) into sibling modules under `crates/tau-memory/src/runtime/`. Behavior-preserving — internal helpers are `pub(super)` and re-exported under their original identifiers at the runtime module root; `FileMemoryStore` is `pub use`'d from the crate root. First two increments of the split plan in [docs/planning/god-file-split-audit-2026-04-23.md](docs/planning/god-file-split-audit-2026-04-23.md). `runtime.rs` shrank from 3660 → 3095 lines (-565); all 97 tau-memory unit tests remain green.

### Dependencies
- `wasmtime`: 25.0.3 → 36
- `wasmparser`: 0.225 → 0.232

## [Unreleased] - 2026-02-19 (pre-agent-safety-harness)

### Added
- M104 remediation baseline artifacts:
  - `.env.example` for operator-ready environment bootstrap.
  - `rustfmt.toml` for explicit formatting policy anchoring.
  - Expanded `tau-safety` conformance/regression test coverage.

### Changed
- `tasks/tau-gaps-issues-improvements.md` refreshed with evidence-backed per-item status mapping and follow-up links.

## [2026-02-17..2026-02-19] - Spacebot parity completion waves (M95-M103)

### Added
- G8 local embeddings completion and validation.
- G2 context compaction phases 2-5 completion, including warn-tier LLM summarization.
- G16 hot-reload completion.
- G4 branch-as-tool phase 2 runtime orchestration and concurrency controls.

### Changed
- Memory enhancement closure validation for G5/G6/G7.
- Process-type profile routing and prompt/template hot-reload integration.

## [2026-02-08..2026-02-16] - Readiness and quality waves (M55-M94)

### Added
- Cargo-fuzz baseline harness and repeatable verification workflow.
- Gateway/dashboard quality and operational hardening waves.
- Skip/react/send-file tool contracts and adapter closure work.

### Changed
- README and roadmap accuracy refresh waves.
- CI/process gates for milestone/spec alignment and faster preflight checks.

## Historical

- Earlier milestone history (M1..M54) is captured in `specs/milestones/` and associated issue/PR evidence.
