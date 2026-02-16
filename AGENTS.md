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
