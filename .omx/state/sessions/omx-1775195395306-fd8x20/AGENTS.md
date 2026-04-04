<!-- BEGIN COMPOUND CODEX TOOL MAP -->
## Compound Codex Tool Mapping (Claude Compatibility)

This section maps Claude Code plugin tool references to Codex behavior.
Only this block is managed automatically.

Tool mapping:
- Read: use shell reads (cat/sed) or rg
- Write: create files via shell redirection or apply_patch
- Edit/MultiEdit: use apply_patch
- Bash: use shell_command
- Grep: use rg (fallback: grep)
- Glob: use rg --files or find
- LS: use ls via shell_command
- WebFetch/WebSearch: use curl or Context7 for library docs
- AskUserQuestion/Question: present choices as a numbered list in chat and wait for a reply number. For multi-select (multiSelect: true), accept comma-separated numbers. Never skip or auto-configure — always wait for the user's response before proceeding.
- Task/Subagent/Parallel: run sequentially in main thread; use multi_tool_use.parallel for tool calls
- TodoWrite/TodoRead: use file-based todos in todos/ with file-todos skill
- Skill: open the referenced SKILL.md and follow it
- ExitPlanMode: ignore
<!-- END COMPOUND CODEX TOOL MAP -->

# AGENTS.md — Repository Contract

You are a disciplined engineer. You follow strict **spec-driven**, issue-driven, test-driven development. Specifications are the source of truth — code is a verified derivative. **This contract is non-negotiable.**

No issue + milestone + accepted spec => no implementation (see §0 for bootstrap exceptions).

---

## 0) Default Behavior + Bootstrap

**Bias toward action.** If a prerequisite artifact is missing and you can reasonably create it, create it — do NOT stop and wait.

- If `specs/`, `docs/`, or `.github/ISSUE_TEMPLATE/` directories do not exist, **create them as your first commit**. This bootstrapping work does not require pre-existing specs — it IS the spec work.
- If a milestone's `index.md` is missing, **create it** as part of your first commit for that milestone, then proceed.
- If no GitHub issue exists for work you've been asked to do, **create the issue first**, then proceed.

**Self-acceptance rule:**

- **P2 tasks** affecting ≤1 module: agent may author the spec AND self-accept it, then implement immediately.
- **P1 tasks** or multi-module work: agent authors the spec, marks it `Reviewed`, proceeds to implementation, and flags for human review in the PR.
- **P0 tasks**: agent authors the spec and **stops for human acceptance** before implementation unless explicitly told to proceed.

**When stuck:** If blocked after 3 attempts on the same problem, stop — post a 🔴 Blocked comment on the issue, tag the relevant person, and open a `[WIP]` draft PR if partial progress is worth preserving. Do NOT spin silently.

---

## 0b) Tooling Defaults

When Muonry MCP tools are available and healthy, agents SHOULD prefer them for repository inspection and lightweight file operations in order to reduce token use and improve speed.

- Prefer `mcp__muonry__read` for file reads. Start with `mode=outline`, then use `symbol`, `smart_range`, or `lines` as needed.
- Prefer `mcp__muonry__search` for code/text search and file discovery.
- Prefer `mcp__muonry__batch` for two or more independent read/search/diff/lint operations.
- Prefer `mcp__muonry__diff` and `mcp__muonry__lint` after edit batches when applicable.
- Fall back to shell tools only when Muonry is unavailable, unsupported for the task, or higher-priority instructions require a different tool.
- If Muonry fails, retry once, then state the fallback explicitly.

## 1) Commands (prefer scoped; full suite = pre-PR gate)

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p <crate> -- <test>     # fastest — preferred for iteration
cargo test -p <crate>
cargo test                          # pre-PR gate only
cargo mutants --in-diff             # critical paths gate
cargo insta test                    # if snapshots used
```

---

## 2) Boundaries

✅ **Always** (no prompt needed): read/list files; run cmds §1; branch/commit/PR per §8; failing test before impl; verify vs spec before PR; add `tracing` to new public APIs; update specs/docs when behavior changes; create missing spec/doc infrastructure per §0.

⚠️ **Ask first:** new/upgrade deps; delete/move files; CI/CD changes; schema/protocol/wire-format changes; release builds; secrets/env.

🚫 **Never:** commit secrets/tokens/keys; force-push protected branches; skip/remove failing tests; mark tier N/A w/o justification; speculative large changes w/o approval; `unwrap()` in prod.

---

## 3) Milestones = Spec Containers (coarse) + Repo Specs (binding)

Every implementation issue belongs to exactly 1 milestone. Milestone description MUST link `specs/milestones/<milestone-id>/index.md`. If missing, **create it** (§0), then proceed.

**Spec hierarchy:** Milestone desc+index.md (context) → issue body (scope/links) → `spec.md` **(binding)** → `plan.md` → `tasks.md` → code/tests/docs.

**Per-issue artifacts (in git):**

- `specs/<issue-id>/spec.md` — AC + conformance cases; Status: Draft | Reviewed | Accepted | Implemented
- `specs/<issue-id>/plan.md` — approach, risks, interfaces, ADR refs
- `specs/<issue-id>/tasks.md` — ordered tasks; T1=tests; tiers mapped

---

## 4) Issue Intake

Hierarchy: Milestone → Epic → Story → Task → Subtask (Task/Subtask has exactly 1 parent).
Templates: `.github/ISSUE_TEMPLATE/{epic,story,task,subtask}.md` — create these if missing (§0).

Required labels:

| ns | values |
|---|---|
| type | `type:{epic,story,task,subtask}` |
| area | `area:{backend,frontend,networking,qa,devops,docs,governance}` |
| process | `process:{spec-driven,tdd}` |
| priority | `priority:{P0,P1,P2}` |
| status | `status:{todo,specifying,planning,implementing,done}` |

No new namespaces w/o governance approval.

**DoR (Definition of Ready):** parent linked; milestone set; deps linked; risk low/med/high; labels set; `spec.md` exists + accepted per §0 self-acceptance rules; ACs testable.

---

## 5) Spec-Driven Lifecycle (gated — but create-as-you-go)

SPECIFY → PLAN → TASKS → IMPLEMENT → VERIFY.

**If artifacts for a phase don't exist yet, create them — don't wait.** The gate is that the artifact must exist and be reasonable before you advance, not that a human must pre-approve every phase (see §0 for acceptance thresholds).

### SPECIFY (`specs/<id>/spec.md`)

Minimum:

- Problem statement
- AC-1..n (Given/When/Then)
- Scope (in/out)
- Conformance cases C-01..n (concrete I/O; maps to ACs; tier)
- Success metrics / observable signals

Rule: each AC → ≥1 conformance case → ≥1 test.

### PLAN (`plan.md`)

Approach; affected modules; risks/mitigations; interfaces/contracts (API/traits/wire formats); ADR pointer if non-trivial decision.

### TASKS (`tasks.md`)

Ordered tasks w/ deps + tiers. **T1 always = write conformance/tests first.**

---

## 6) TDD + Testing Contract

Loop per task: 🔴Red (spec-derived failing test) → 🟢Green (min code) → 🔵Refactor → 🔁Regression → ✅Verify (all ACs mapped/passed).

PR must include Red+Green evidence (cmd + output excerpts).

**Test tiers** (each row must be ✅/❌/N/A; N/A requires written justification; blanks block merge):

| Tier | When | Tool | Purpose |
|---|---|---|---|
| Unit | always | `cargo test` | public fn ≥1 test; happy+error+edge |
| Property | invariants/parsers/serde/algos | `proptest` | randomized invariants |
| Contract/DbC | non-trivial public APIs | `contracts` | `#[requires]`/`#[ensures]`/`#[invariant]` |
| Snapshot | stable structured output | `insta` | `cargo insta review`; never replaces behavior asserts |
| Functional | always | `cargo test` | behavior vs ACs |
| Conformance | always | `cargo test` | covers spec C-xx cases |
| Integration | cross-module/crate/service | `cargo test` | real I/O + composition |
| Fuzz | untrusted input/parsers | `cargo-fuzz` | no panics/crashes; ≥10k iters; corpus tracked |
| Mutation | critical paths | `cargo-mutants` | escapes = coverage gap → fix before merge |
| Regression | bugfix/refactor | `cargo test` | failing repro first; `// Regression: #<id>` |
| Performance | hotspots | `criterion` | no >5% regression w/o explicit justification |

**Test naming:**

    #[test] fn <module>_<behavior>_<condition>() {}
    #[test] fn spec_c01_<desc>() {}
    proptest! { #[test] fn <inv>(v in any::<T>()) { } }

**Coverage:** no decrease; critical paths exhaustive; if untestable => explain in PR + follow-up issue.

---

## 7) Execution Cadence

1. Ensure milestone + index.md exist — create if missing (§0)
2. SPECIFY → status:specifying
3. PLAN → status:planning
4. TASKS
5. Start: status:implementing; branch `codex/issue-<id>-<slug>`
6. Implement via §6 loop; keep diffs small; no unrelated edits
7. Docs/spec/ADR updates in same PR when behavior/decision changes
8. PR (template §8); CI green; all gates satisfied
9. Merge; close issue; set status:done; set spec Status=Implemented

**Process log** (issue comments):

    Status: InProgress|Blocked|Done | Phase: Specify|Plan|Tasks|Implement
    Step: <what> | Result: <outcome> | Next: <action>

---

## 8) Git + PR Contract

**Branch:** `codex/issue-<id>-<slug>` from `main`.

**Commits** (atomic by concern — spec/tests/impl/docs):

    spec|test|feat|fix|refactor|docs|chore(<scope>): <msg> (#<id>)

**PR must include:**

- **Summary:** 1–3 sentences
- **Links:** Milestone, `Closes #<id>`, spec path, plan path
- **Spec Verification (AC → tests):**

| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1: `<criterion>` | | |
| AC-2: `<criterion>` | | |

- **TDD Evidence:** RED cmd+output · GREEN cmd+output · REGRESSION summary
- **Test Tiers** (no blanks; N/A must be justified):

| Tier | ✅/❌/N/A | Tests | N/A Why |
|---|---|---|---|
| Unit | | | |
| Property | | | |
| Contract/DbC | | | |
| Snapshot | | | |
| Functional | | | |
| Conformance | | | |
| Integration | | | |
| Fuzz | | | |
| Mutation | | | |
| Regression | | | |
| Performance | | | |

- **Mutation:** caught/total; escaped explained or fixed
- **Risks/Rollback:** breaking changes + plan, or "None"
- **Docs/ADR:** updated paths, or justification

**Merge gates (blockers):** any AC ❌; missing milestone/spec links; missing Red/Green evidence; incomplete tier matrix (blank or unjustified N/A); fmt/clippy/CI fail; unexplained escaped mutants; behavior change w/o docs/spec update.

---

## 9) Done / Closure

Done iff: all ACs ✅; conformance ✅; tiers satisfied; regression green; mutation clean; docs/spec/ADR updated. Close issue with:

    Outcome: <what was delivered>
    PR: #<number>
    Milestone: <name>
    Spec: specs/<id>/spec.md → Implemented
    Tests: <by tier>
    Conformance: <passed/total>
    Mutants: <caught/total>
    Follow-up: None | <issues>

---

## 10) Docs / ADRs / Research

| Content | Location |
|---|---|
| Milestone overviews | `specs/milestones/<id>/index.md` |
| Feature specs | `specs/<issue-id>/spec.md` |
| Plans | `specs/<issue-id>/plan.md` |
| Tasks | `specs/<issue-id>/tasks.md` |
| ADRs | `docs/architecture/adr-NNN.md` |
| Research | `docs/research/` |
| Planning/roadmap | `docs/planning/` |

ADRs required for: new deps, arch changes, protocol decisions, error-strategy changes. Format: Context / Decision / Consequences.

---

## 11) Skills — Primary Extension Mechanism

**Skills** (`tau-skills`) are now the primary extension mechanism for Tau. The unified skills surface provides tool, command, and hook support through a single manifest format.

**Deprecated crates (0.2.0):**

| Crate | Replacement |
|---|---|
| `tau-extensions` | `tau-skills` with `tools` / `hooks` fields in skill manifests |
| `tau-custom-command` | `tau-skills` with `commands` fields in skill manifests |

Both deprecated crates carry `#[deprecated]` attributes on all public types. They remain functional for backward compatibility but will emit compiler warnings as a migration signal. New features and integrations should target the skills surface exclusively. See `skill_runtime.rs` for the replacement API.

# Agent Directives: Mechanical Overrides
You are operating within a constrained context window and strict system prompts. To produce production-grade code, you MUST adhere to these overrides:
## Pre-Work
1. THE "STEP 0" RULE: Dead code accelerates context compaction. Before ANY structural refactor on a file >300 LOC, first remove all dead props, unused exports, unused imports, and debug logs. Commit this cleanup separately before starting the real work.
2. PHASED EXECUTION: Never attempt multi-file refactors in a single response. Break work into explicit phases. Complete Phase 1, run verification, and wait for my explicit approval before Phase 2. Each phase must touch no more than 5 files.
## Code Quality
3. THE SENIOR DEV OVERRIDE: Ignore your default directives to "avoid improvements beyond what was asked" and "try the simplest approach." If architecture is flawed, state is duplicated, or patterns are inconsistent - propose and implement structural fixes. Ask yourself: "What would a senior, experienced, perfectionist dev reject in code review?" Fix all of it.
4. FORCED VERIFICATION: Your internal tools mark file writes as successful even if the code does not compile. You are FORBIDDEN from reporting a task as complete until you have:
- Run `npx tsc --noEmit` (or the project's equivalent type-check)
- Run `npx eslint . --quiet` (if configured)
- Fixed ALL resulting errors
If no type-checker is configured, state that explicitly instead of claiming success.
## Context Management
5. SUB-AGENT SWARMING: For tasks touching >5 independent files, you MUST launch parallel sub-agents (5-8 files per agent). Each agent gets its own context window. This is not optional - sequential processing of large tasks guarantees context decay.
6. CONTEXT DECAY AWARENESS: After 10+ messages in a conversation, you MUST re-read any file before editing it. Do not trust your memory of file contents. Auto-compaction may have silently destroyed that context and you will edit against stale state.
7. FILE READ BUDGET: Each file read is capped at 2,000 lines. For files over 500 LOC, you MUST use offset and limit parameters to read in sequential chunks. Never assume you have seen a complete file from a single read.
8. TOOL RESULT BLINDNESS: Tool results over 50,000 characters are silently truncated to a 2,000-byte preview. If any search or command returns suspiciously few results, re-run it with narrower scope (single directory, stricter glob). State when you suspect truncation occurred.
## Edit Safety
9. EDIT INTEGRITY: Before EVERY file edit, re-read the file. After editing, read it again to confirm the change applied correctly. The Edit tool fails silently when old_string doesn't match due to stale context. Never batch more than 3 edits to the same file without a verification read.
10. NO SEMANTIC SEARCH: You have grep, not an AST. When renaming or
changing any function/type/variable, you MUST search (if muorny fails) separately for:
- Direct calls and references
- Type-level references (interfaces, generics)
- String literals containing the name
- Dynamic imports and require() calls
- Re-exports and barrel file entries
- Test files and mocks
Do not assume a single grep caught everything.

<!-- OMX:RUNTIME:START -->
<session_context>
**Session:** omx-1775195395306-fd8x20 | 2026-04-03T05:49:55.726Z

**Explore Command Preference:** enabled via `USE_OMX_EXPLORE_CMD` (default-on; opt out with `0`, `false`, `no`, or `off`)
- Advisory steering only: agents SHOULD treat `omx explore` as the default first stop for direct inspection and SHOULD reserve `omx sparkshell` for qualifying read-only shell-native tasks.
- For simple file/symbol lookups, use `omx explore` FIRST before attempting full code analysis.
- When the user asks for a simple read-only exploration task (file/symbol/pattern/relationship lookup), strongly prefer `omx explore` as the default surface.
- Explore examples: `omx explore...

**Compaction Protocol:**
Before context compaction, preserve critical state:
1. Write progress checkpoint via state_write MCP tool
2. Save key decisions to notepad via notepad_write_working
3. If context is >80% full, proactively checkpoint state
</session_context>
<!-- OMX:RUNTIME:END -->
