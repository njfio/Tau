# Pattern: the "Status: Implemented" lie — when spec claims don't survive a mod-graph check

**Category**: patterns
**Slug**: spec-status-lies-2026-04
**Date**: 2026-04-23
**Related**:
  - `docs/solutions/patterns/fallibility-audit-workspace-2026-04.md` (Category A)
  - `specs/3680/spec.md`, `specs/3681/spec.md` (forensic examples)
  - commit `275eb78a` (orphan file removal + spec reclassification)
  - commit `8926bd4a` (where the drift was introduced)

## Problem

Spec documents under `specs/` carry a `Status:` header with values like
`Draft`, `Proposed`, `Implemented`, `Deferred`. The repo has **1052 specs
marked `Status: Implemented`** at the time of this audit. That value is
treated as ground truth by downstream readers: roadmap docs, milestone
trackers, `docs/` narratives, ADRs citing prior work, and future agents
searching via `compound_search` for "is X already built?".

During the 2026-04 fallibility audit we discovered specs #3680 and #3681
were both marked `Status: Implemented`, yet:

- The claimed implementation files
  (`crates/tau-tui/src/interactive/session_state.rs`,
  `transcript_state.rs`, and their `_tests.rs` siblings) **never appeared
  in any `mod` declaration in any git revision**.
- `cargo build -p tau-tui` never touched those files. The tests never ran.
- The runtime types the spec claimed extensions of (`AppConfig`,
  `App::new`) had zero fields matching the spec's acceptance criteria
  (no `local_state_path`, no `prompt_history`, no `input_draft`, no
  `active_mission_id`).
- No workspace reference to any symbol described in the spec existed
  outside the four orphan files themselves.

The feature shipped **zero runtime behavior**. Yet for ~13 months the
status header asserted otherwise.

## Root cause

Three compounding factors:

1. **No build-time link between spec status and compile graph.** The
   `Status:` field is a markdown header typed by a human (or an agent)
   at commit time and never re-validated. The Rust compiler cannot reach
   markdown; the markdown never reaches the compiler.

2. **Broad "topic" commits that sweep unrelated dead files in.** The
   orphan files were introduced in commit `8926bd4a` titled
   `feat(provider): codex app-server WebSocket integration` — a
   completely unrelated topic. The session/transcript files were
   collateral, landed without being wired into `mod.rs`, and nobody
   reviewing the WebSocket diff caught that four `.rs` files under a
   different crate had no mod declaration.

3. **Spec review culture treats the spec as the deliverable.** Once the
   spec/plan/tasks triad was committed and the stage slug was
   `mark_done`'d, the Status header flipped to `Implemented` based on
   agent/operator confidence that "the work ran" — without a mechanical
   check that the symbols named in the spec's acceptance criteria
   actually existed in a reachable module.

## Solution

### Immediate (shipped this audit)

- Reclassified specs #3680 and #3681 to `Status: Not Integrated
  (2026-04-23 reclassification — see below)` with forensic notes
  pointing to the audit trail.
- Deleted the four orphan files (260 lines of uncompiled code) in
  commit `275eb78a`.

### Proposed process fixes (not yet implemented)

**Fix 1: Require grep-testable anchor in every spec.**

Every spec with `Status: Implemented` should name at least one
**grep-testable anchor symbol** — a public function, type, or module
path that, if removed, would obviously break the feature. Add a field
to the spec template:

```markdown
## Implementation anchors

- Primary symbol: `crates/tau-tui/src/interactive/session_state.rs::SessionState`
- Wiring point: `crates/tau-tui/src/interactive/mod.rs` declares `pub mod session_state;`
- Activation call site: `crates/tau-tui/src/interactive/app.rs::App::new` invokes `SessionState::restore(...)`.
```

**Fix 2: Ship a `spec-anchor-check` verifier.**

A script under `scripts/verify/spec-anchor-check.sh` that for each
`Status: Implemented` spec:
1. Extracts the `## Implementation anchors` block.
2. For each path/symbol pair, runs `rg --type rust` to confirm the
   symbol is defined AND is reachable from the crate root via the mod
   graph (`cargo-modules` or manual walk).
3. Fails CI if any anchor is orphaned.

**Fix 3: Auto-detect orphan `.rs` files workspace-wide.**

A simpler, lower-ceremony check: any `.rs` file under `crates/*/src/`
that is not reachable from its crate's `lib.rs` / `main.rs` via the
mod graph is either a bug (forgot to declare `mod foo;`) or dead code.
`cargo-modules generate tree` already produces this data; wrap it in
a pre-commit / CI gate.

**Fix 4: Link spec status to a verify_deliverable run.**

For Gyre-driven work: a spec should not be marked `Status: Implemented`
unless its stage's `verify_deliverable` returned `passed: true` for a
deliverable whose acceptance command exercises the claimed runtime
behavior. The existing `.gyre/state/verified-passing.json` already
tracks this; extend the `backlog_mark_done` hook to also write the
verified spec ID back into the spec's front-matter.

## Prevention

For agents (self-prompt during any future Phase 3 stage that would flip
a `Status: Implemented` bit):

1. **Grep the claimed anchor before flipping status.** If the spec says
   "implements `foo::bar::Baz`", run `rg 'struct Baz|fn baz' crates/foo/`
   and confirm the symbol is defined AND reachable via `mod` declarations
   from the crate root.
2. **Suspect large topic commits.** Any commit adding >5 `.rs` files
   where the commit message doesn't mention all of them is a smell;
   scan each added file for a matching `pub mod <filename>;` line in a
   sibling `mod.rs` or `lib.rs`.
3. **Treat `Status: Implemented` as a claim to be challenged, not a
   fact to be trusted.** When `compound_search` returns a spec that
   claims implementation of something you were about to build, do a
   10-second grep for the claimed symbols before deciding "already
   done, moving on."

For the broader codebase: **1052 specs are currently marked
`Implemented`.** A bounded sampling audit (random 30 → check one anchor
symbol per) would give a useful estimate of how many are lies.
Proposal sizing:
- 30 specs × ~2 min grep per = ~1 hour agent time
- Expected yield: 0–3 additional orphaned features at the same
  category-A severity as #3680/#3681
- Value: each lie discovered is a source of future "oh wait, that's
  not actually built" rework avoided

## Known limitations of the proposed fixes

- Fix 1 requires discipline at spec-authoring time. Specs authored
  before the field existed would need backfill (or be grandfathered).
- Fix 2/3 catch structural orphans (files not in mod graph) but miss
  **semantic orphans** — code that compiles but whose entry point is
  never called from a runtime path. A deeper check would need
  reachability analysis from `main.rs` binaries + public API, which
  is expensive to maintain.
- Fix 4 only covers work done under Gyre. Non-Gyre commits still need
  a separate check path.

## Keywords

spec-status, dead-code, mod-graph, orphan-files, governance, audit,
tau-tui, specs/3680, specs/3681, fallibility-audit, verify-before-done
