# God-File Split Audit (2026-04-23)

**Status**: Plan only — no code changes in this stage. Any subsequent refactor
is tracked as a separate stage with its own ADR if the split changes the
public API surface.

## Method

Ranked all workspace `*.rs` files by line count (excluding `target/`). Top
three **production** files (excluding tests/ files and integration suites)
emerged as clear god-files by the standard rule of thumb (>2000 lines
production code in one file is a maintainability smell):

| Rank | File | Lines | Primary concerns counted |
|------|------|------:|-------------------------|
| 1 | `crates/tau-dashboard-ui/src/lib.rs` | 3999 | 17 data structs + 4 enums + 7 helpers + 3 render entry points |
| 2 | `crates/tau-agent-core/src/lib.rs` | 3816 | 1 × 2350-line `impl Agent` + 10 enums + 12 internal structs |
| 3 | `crates/tau-memory/src/runtime.rs` | 3660 | 23 data types + `FileMemoryStore` impl + 8 normalization helpers + 13 serde defaults |

Sizes are of the file itself, not of its crate. Tests-only files (e.g.
`tau-gateway/src/gateway_openresponses/tests.rs` at 17211 lines) are a
separate category — they are dense but homogeneous and split naturally by
test scenario; they are not god-files in the architectural sense.

## Shared scoring rubric

Each proposal is rated along four axes:

- **Value** (V): how much does the split improve local reasoning / PR diff size / reviewability?
- **Risk** (R): can the split regress behavior (visibility leaks, cyclic imports, test surface changes)?
- **Effort** (E): roughly how many mechanical steps to execute, assuming no semantic change.
- **Reversibility** (X): how easy is rollback if the split turns out worse?

Scale: `low / med / high` per axis. A split is worth doing when V ≥ med and
R ≤ med and X ≥ med.

---

## Proposal 1 — `tau-dashboard-ui/src/lib.rs` (3999 lines)

### Current structure

```
  7– 217  Enums                 TauOpsDashboardAuthMode/Route/Theme/SidebarState
 219– 348  Data rows            14 *Row structs (AlertFeed, ConnectorHealth, ChatMessage, Memory*, Session*, Tool*, Job)
 350– 609  Snapshots            ChatSnapshot (350–531), CommandCenterSnapshot (533–609)
 611– 631  Shell context        ShellContext + Default
 633– 706  Rendering helpers    markdown detection, graph-node size/color/edge-style contracts
 709– 724  Public entry points  render_tau_ops_dashboard_shell{_for_route,_with_context}
 724–3999  Body of render_*     (~3275 lines of sub-rendering logic — the real beast)
```

### Proposed layout

```
crates/tau-dashboard-ui/src/
├── lib.rs              # mod decls + pub re-exports only, ~50 lines
├── route.rs            # AuthMode, Route, Theme, SidebarState  (~220 lines)
├── rows.rs             # 14 *Row DTO structs  (~140 lines)
├── snapshot/
│   ├── mod.rs
│   ├── chat.rs         # ChatSnapshot + Default  (~185 lines)
│   └── command.rs      # CommandCenterSnapshot + Default  (~80 lines)
├── shell.rs            # ShellContext + Default  (~25 lines)
├── render/
│   ├── mod.rs          # the three pub render_* entry points
│   ├── markdown.rs     # contains_markdown_contract_syntax, extract_first_fenced_code_block, extract_assistant_stream_tokens
│   ├── memory_graph.rs # derive_memory_graph_{node_size,node_color,edge_style}_contracts
│   └── sections/       # one file per major dashboard section used by the 3275-line render body
│       ├── alerts.rs
│       ├── chat.rs
│       ├── memory.rs
│       ├── sessions.rs
│       └── tools.rs
└── tests.rs            # unchanged
```

### Rationale

- The enum cluster + Row DTOs + Snapshot structs are all **pure data types**
  with zero inter-type method coupling. Moving them is mechanical.
- The real win is splitting the 3275-line render body into section renderers.
  Each section (alerts, memory, sessions, tools, chat) is a visually separable
  concern in the rendered HTML and in the caller's intent.
- `markdown.rs` and `memory_graph.rs` are called from multiple sections; they
  become shared helpers the same way they are today.

### Risk

- `tests.rs` is 3702 lines and almost certainly references private items by
  path. Moving types will force many `use super::*` imports; none should
  change behavior but all need to compile. **Recommend: run split in two PRs —
  types first, render body second — so either can roll back independently.**
- No `pub` surface changes expected; every item currently visible stays
  visible from the same `tau_dashboard_ui::*` crate root via `pub use`.

### Scoring

V: **high** (reviewability unblocked — today this file is near-impossible to review whole)
R: **med** (large test suite compiles against current structure)
E: **med** (most mechanical; ~4–6 hours focused)
X: **high** (pure re-export split; revert is a branch squash)

**Verdict: worth doing. Two-PR split recommended.**

---

## Proposal 2 — `tau-agent-core/src/lib.rs` (3816 lines)

### Current structure

```
   30–  41  Sibling mods         agent_channel, circuit_breaker, context_ranking, failure_detector,
                                 metrics, recovery, cortex_runtime, runtime_safety_memory,
                                 runtime_startup, runtime_tool_bridge, runtime_turn_loop, process_types
  123– 286  AgentConfig          + huge Default impl
  287– 339  Cost + async-dispatch metrics
  341– 387  CooperativeCancellationToken
  389– 423  ToolExecutionResult
  425– 643  Assistant-message directive parsers  (skip_response, ReactResponseDirective, SendFileResponseDirective)
  645     pub trait AgentTool
  655– 739  AgentEvent enum      (large variant set)
  741– 793  AgentError enum
  795– 929  AgentDirectMessage*  error + policy
  931–1019  Internal guards/stats structs (RegisteredTool, ToolExecutionStats, SafetyInspection,
                                  PendingWarnCompaction, ReadyWarnCompaction, WarnCompactionState,
                                  BranchRunSlotGuard, BranchWorkerFollowupReport, MemoryRecallMatch)
 1021–3372  pub struct Agent + impl        ← 2351 lines of impl in one block
 3372+      #[cfg(test)] mod tests
```

### Proposed layout

```
crates/tau-agent-core/src/
├── lib.rs                           # mod decls + re-exports, ~80 lines
├── config.rs                        # AgentConfig + Default  (~165 lines)
├── metrics_inline.rs                # AgentCostSnapshot, AsyncEventDispatchMetrics(Inner)  (~55 lines)
├── cancellation.rs                  # CooperativeCancellationToken  (~50 lines)
├── tool_execution_result.rs         # ToolExecutionResult  (~40 lines)
├── tool_trait.rs                    # pub trait AgentTool  (~15 lines — keeps crate root import ergonomic)
├── directives/                      # assistant-message directive parsers
│   ├── mod.rs
│   ├── skip_response.rs             # extract_skip_response_reason + parse helpers
│   ├── react.rs                     # ReactResponseDirective + extractor
│   └── send_file.rs                 # SendFileResponseDirective + extractor
├── events.rs                        # AgentEvent enum  (~90 lines)
├── errors.rs                        # AgentError  (~55 lines)
├── direct_message.rs                # AgentDirectMessageError + AgentDirectMessagePolicy  (~135 lines)
├── internal_types.rs                # pub(crate) RegisteredTool, ToolExecutionStats, etc.  (~90 lines)
└── agent/                           # the 2351-line Agent impl, split by concern
    ├── mod.rs                       # pub struct Agent + tiny constructor/builders
    ├── construction.rs              # impl Agent { new/with_config/with_memory/... }
    ├── turn_execution.rs            # impl Agent { run_turn, drive_turn, ... }       ← the hot path
    ├── tool_dispatch.rs             # impl Agent { dispatch_tool_call, handle_tool_result, ... }
    ├── memory_recall.rs             # impl Agent { recall, rank_memory, ... }
    ├── safety.rs                    # impl Agent { inspect_safety, enforce_policy, ... }
    ├── branch_workers.rs            # impl Agent { spawn_branch, collect_followups, ... }
    └── warn_compaction.rs           # impl Agent { plan_warn_compaction, apply, ... }
```

The team has **already started** this pattern with the sibling `runtime_*` modules
(`runtime_startup`, `runtime_turn_loop`, `runtime_tool_bridge`, `runtime_safety_memory`).
The god-file is effectively an Agent impl that outgrew the existing splits —
the proposal continues the established direction.

### Rationale

- Rust allows `impl Agent {}` to be re-opened across files in the same crate.
  The 2351-line impl can be peeled into concern-specific blocks with **zero
  visibility changes** — private fields stay module-private, `pub(crate)` stays
  `pub(crate)`, pub stays pub.
- Each directive parser already has matching unit tests keyed to the parser
  function name; moving them keeps tests adjacent.

### Risk

- `Agent` has many private fields. If the concern files end up importing them
  via `&self.field_name`, that's fine; but if any helper currently takes `&mut
  self` and touches multiple unrelated fields, splitting it into concern files
  may feel awkward. **Mitigation**: leave any truly cross-cutting helper in
  `agent/mod.rs` until its second co-located caller emerges.
- `pub(crate)` items (`ReactResponseDirective`, `SendFileResponseDirective`,
  `RegisteredTool`, ...) must keep crate visibility — the current file's
  `pub(crate)` turns into `pub(crate)` in the new module, same effect.
- **Biggest risk**: the tests module at line 3372+ is a monster of inlined
  private-field assertions. Count its lines and review its imports *before*
  starting the split.

### Scoring

V: **high** (this is the engine of every runtime; readable = fewer bugs)
R: **high** (large impl + extensive test coupling + central to half the crates)
E: **high** (2–3 staged PRs to do safely: config/DTOs first, directives second, Agent impl third)
X: **med** (split can be rolled back, but with each PR's test churn, later revert is painful)

**Verdict: worth doing but approach with discipline.** Recommend staging:
  1. PR 1: Extract data types (AgentConfig, metrics, cancellation, ToolExecutionResult, events, errors, direct_message) — pure moves.
  2. PR 2: Extract directives/ module.
  3. PR 3: Split `impl Agent {}` into `agent/` sub-modules. Use a single
     commit to rename files but no logic edits; require a full-workspace
     test pass before merging.
  4. Defer further impl splitting until natural fault lines appear under
     normal maintenance.

---

## Proposal 3 — `tau-memory/src/runtime.rs` (3660 lines)

### Current structure

```
   11–  13  Sibling mods         backend, query, ranking
   68–  87  Backend enums        MemoryStorageBackend, ResolvedMemoryBackend
   89– 150  serde defaults       13 × fn default_*  (tiny, homogeneous, already named)
  158– 664  Data types (23)      MemoryRelationType/Relation/Input, MemoryType,
                                 MemoryTypeImportanceProfile, MemoryEmbeddingProviderConfig,
                                 ComputedEmbedding, RuntimeMemoryRecord, MemoryScopeFilter,
                                 MemoryWriteResult, MemorySearchOptions,
                                 MemoryLifecycleMaintenancePolicy/Result,
                                 MemoryIngestionOptions/Llm/WatchPolling/Result,
                                 MemorySearchMatch/Result, MemoryTreeNode/Tree,
                                 RankedTextCandidate/Match
  667–1043  pub struct FileMemoryStore + impl
 1045–1178  Normalization utils  (normalize_scope, sqlite_i64_from_u64, normalize_entry,
                                  normalize_relations, current_unix_timestamp_ms)
 1180+      mod tests
```

### Proposed layout

```
crates/tau-memory/src/runtime/
├── mod.rs              # (formerly runtime.rs) — mod decls + re-exports only
├── defaults.rs         # serde default_* functions  (~70 lines)
├── types/
│   ├── mod.rs
│   ├── backend.rs      # MemoryStorageBackend, ResolvedMemoryBackend
│   ├── relation.rs     # MemoryRelationType, MemoryRelation, MemoryRelationInput
│   ├── record.rs       # MemoryType + importance profile + RuntimeMemoryRecord + ComputedEmbedding + embedding provider config
│   ├── search.rs       # MemoryScopeFilter, MemorySearchOptions, MemorySearchMatch, MemorySearchResult, RankedTextCandidate, RankedTextMatch
│   ├── lifecycle.rs    # MemoryLifecycleMaintenancePolicy/Result, MemoryWriteResult
│   ├── ingestion.rs    # MemoryIngestionOptions/Llm/WatchPolling/Result
│   └── tree.rs         # MemoryTreeNode, MemoryTree
├── file_store.rs       # pub struct FileMemoryStore + impl  (~380 lines)
├── normalize.rs        # normalize_* helpers + current_unix_timestamp_ms  (~140 lines)
├── backend.rs          # (existing sibling)
├── query.rs            # (existing sibling — wait, this is the file we're splitting; existing `mod query` stays at runtime/query.rs)
└── ranking.rs          # (existing sibling)
```

Existing `runtime.rs` becomes `runtime/mod.rs`. The `mod backend; mod query;
mod ranking;` lines continue to work because Rust resolves sub-modules of
`runtime/mod.rs` at `runtime/*.rs`.

### Rationale

- This is the **easiest** of the three. Almost every data type has zero method
  coupling to its neighbor; they're serde-annotated DTOs.
- The cluster already has a `types/` directory habit in adjacent crates
  (`tau-contract`, `tau-events`) — this proposal mirrors that convention.
- `FileMemoryStore` is a single cohesive piece — no internal split needed,
  just move the whole impl to its own file.

### Risk

- **Minimal**. All types are currently accessible through the `tau_memory::
  runtime::*` root; preserving that via `pub use` in the new `runtime/mod.rs`
  keeps every downstream caller working without edits.

### Scoring

V: **med** (nice reviewability improvement; less severe than #1 and #2 because
           the internal organization is already orderly)
R: **low** (pure DTO moves + zero method coupling across types)
E: **low** (1 PR, ~2 hours focused)
X: **high**

**Verdict: worth doing. Single PR.**

---

## Sequencing recommendation

If only one is done, pick **Proposal 3 first** (lowest risk, easiest to
review, establishes the split-by-concern pattern for the team). Then
**Proposal 1** (high value on UI code that changes often). Defer
**Proposal 2** until a quieter period — it's valuable but disruptive to
`tau-agent-core`, which many crates import.

Do not do all three in one release.

## Open questions (for the next stage askQuestions)

1. Is the two-PR split acceptable for Proposal 1, or does the team prefer
   one atomic PR with a heavier review burden?
2. Does Proposal 2 need an ADR (since the `agent/` module reorganization is
   cross-cutting and hard to reverse once downstream crates adjust imports)?
3. Should the `types/` sub-module pattern from Proposal 3 be applied
   retroactively to `tau-contract` and `tau-events` for consistency?

## What was NOT done in this stage

- No code was moved.
- No `Cargo.toml` changes.
- No new modules created.
- No tests added/removed.
- No public API changes.

This document is a planning artifact. The actual splits should be executed
as separate, narrowly-scoped stages with their own verification gates.
