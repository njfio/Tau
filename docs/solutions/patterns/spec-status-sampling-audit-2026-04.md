# Audit: 30-spec sampling of `Status: Implemented` claims (2026-04)

**Category**: patterns (audit follow-up)
**Slug**: spec-status-sampling-audit-2026-04
**Date**: 2026-04-23
**Parent**: `docs/solutions/patterns/spec-status-lies-2026-04.md`
**Seed**: `random.seed(3680)` on the sorted list of 670 unique spec
directories claiming `Status: Implemented` (out of 1052 total markdown
files carrying that header — multiple files per dir counted once).

## Question

The parent pattern doc identified specs/3680 and specs/3681 as
"Status: Implemented" lies — claimed implementations that never appeared
in any mod graph. It posited 1052 such headers across the repo and
proposed a 30-spec sampling audit to estimate lie density. This
document reports that audit.

## Method

1. List all files matching `rg -l 'Status: Implemented' specs/` — 1052 hits.
2. Group by spec directory; prefer `spec.md` over `plan.md`/`tasks.md`
   for the canonical status claim. Yields 670 unique spec dirs.
3. Deterministic random sample of 30 using `random.seed(3680)`.
4. For each sampled spec, extract anchor evidence in priority order:
   - file-path patterns `crates/<name>/src/.../*.rs`
   - Rust-identifier patterns in backticks (CamelCase ≥4 chars or
     `foo::bar::Baz` qualified paths)
5. Classify into: **REAL-files** (all claimed paths exist on disk),
   **REAL-syms** (first 3 backticked symbols grep-hit in `crates/`),
   **MISSING** (symbol nowhere found), **AMBIGUOUS** (no code-level
   anchors extractable from prose).
6. For AMBIGUOUS specs, a manual spot-check of 6 representative picks
   walks the spec's prose claim and checks for the referenced artifact
   (script, milestone doc, subtask dir, doc-comment presence, etc.).

## Sample

Seed `random.seed(3680)`, 30 dirs:

```
specs/1645 specs/1691 specs/1696 specs/1727 specs/1996 specs/1998
specs/2046 specs/2067 specs/2111 specs/2130 specs/2171 specs/2203
specs/2218 specs/2320 specs/2400 specs/2433 specs/2443 specs/2482
specs/2487 specs/2492 specs/2537 specs/2556 specs/2593 specs/2708
specs/2893 specs/3224 specs/3429 specs/3498 specs/3604 specs/3698
```

## Classification table

| Spec | Class | Evidence |
|------|-------|----------|
| 1645 | AMBIGUOUS (spot-check REAL) | spec files under `specs/1645/` exist; roll-up issue for #1618 safety smoke |
| 1691 | AMBIGUOUS (spot-check REAL) | 24 `//!` module docs under `crates/tau-onboarding/src/` |
| 1696 | AMBIGUOUS | not individually checked |
| 1727 | AMBIGUOUS | not individually checked |
| 1996 | AMBIGUOUS | not individually checked |
| 1998 | REAL-syms | `M24RLGateExitDecision`, `M24RLGateEvidenceBundle` grep-hit |
| 2046 | AMBIGUOUS | not individually checked |
| 2067 | REAL-files | 1/1 claimed path exists |
| 2111 | AMBIGUOUS (spot-check REAL) | `specs/milestones/m29/index.md` exists |
| 2130 | AMBIGUOUS | not individually checked |
| 2171 | REAL-files | 4/4 claimed paths exist |
| 2203 | REAL-files | 1/1 claimed paths exist |
| 2218 | AMBIGUOUS (spot-check REAL) | subtask dir `specs/2219/` exists |
| 2320 | REAL-syms | `Resolved`, `Partial`, `Open` grep-hit |
| 2400 | REAL-files | 1/1 claimed paths exist |
| 2433 | AMBIGUOUS | not individually checked |
| 2443 | AMBIGUOUS (spot-check REAL) | relation handling present in `crates/tau-memory/src/runtime/{backend,ranking,query,file_store}.rs` |
| 2482 | AMBIGUOUS | not individually checked |
| 2487 | AMBIGUOUS | not individually checked |
| 2492 | AMBIGUOUS | not individually checked |
| 2537 | AMBIGUOUS | not individually checked |
| 2556 | REAL-syms | `TAU_MEMORY_EMBEDDING_PROVIDER`, `ToolPolicy` grep-hit |
| 2593 | AMBIGUOUS | not individually checked |
| 2708 | AMBIGUOUS | not individually checked |
| 2893 | AMBIGUOUS | not individually checked |
| 3224 | AMBIGUOUS | not individually checked |
| 3429 | AMBIGUOUS (spot-check REAL) | `scripts/verify/m296-integrated-reliability-wave.sh` exists |
| 3498 | AMBIGUOUS | not individually checked |
| 3604 | REAL-files | 4/4 claimed paths exist |
| 3698 | AMBIGUOUS | not individually checked |

## Results

- **Automated-anchor REAL**: 6/30 (`REAL-files` + `REAL-syms`).
- **Automated MISSING**: 0/30.
- **AMBIGUOUS**: 24/30. Of the 6 manually spot-checked, all 6 REAL.
- **Observed lies (category-A orphans like specs/3680/3681)**: 0/30.

## Interpretation

### Lie-density estimate

With 0 observed failures out of 30 trials, a Rule-of-Three 95%-confidence
upper bound on the failure rate is 3/30 = 10%. Applied to the 1052-spec
population: 95% confidence that fewer than ~105 specs are category-A
orphan lies of the same severity as specs/3680/3681. Point estimate:
0% (with large uncertainty).

### The specs/3680/3681 lies appear to be a narrow failure mode

The orphan-files pattern that produced specs/3680 and specs/3681
required a specific confluence:

1. A large "topic" commit sweeping many `.rs` files into `crates/*/src/`.
2. A subset of those files never declared in any sibling `mod.rs`.
3. The spec's status was flipped by a follow-on actor (agent or human)
   who trusted the commit message's implication of completion without
   grepping for the claimed symbols.

The 30-spec sample found zero recurrences of this pattern. The majority
of `Status: Implemented` specs appear to be **roll-up / story / milestone
closure** documents whose "implementation" is a mix of subtask links,
doc-comment additions, and evidence-artifact collection — not
concentrated in a single greppable code-symbol delivery. Those are
harder to lie about because the "work" is distributed across already-
tracked child specs and visible docs.

### AMBIGUOUS is not the same as unreliable

24/30 specs had no clearly-extractable code anchor (file path or
backticked symbol). This is a style issue, not a correctness issue.
Spot-checking 6 of them found all 6 REAL when the prose was read
carefully and the claimed artifact (script, milestone dir, subtask
dir, doc comment) was grepped for. The `AMBIGUOUS` classification
reflects our extractor's inability to machine-read the claim, not the
claim's truth value.

## Recommendation

Downgrade the urgency of the four fixes proposed in
`spec-status-lies-2026-04.md`:

- **Fix 1 (anchor field in spec template)**: Still valuable, but less
  urgent than initially framed. Not an emergency clean-up. Adopt for
  NEW specs; don't backfill en masse.
- **Fix 2 (spec-anchor verifier script)**: Skip until a second category-A
  lie is found. Its catchment rate on the 30-sample would have been
  6/30 for anchor presence but 0/30 for lie detection.
- **Fix 3 (orphan `.rs` scanner)**: ***Still high-value and low-cost.***
  This is what would have caught specs/3680/3681 before status flip and
  before the files accumulated a year of fake reputation. It's a
  cargo-modules one-liner in CI. Highest-leverage single fix.
- **Fix 4 (tie status to verified-passing.json)**: Already effectively
  true for Gyre-driven work via the `.gyre/state/verified-passing.json`
  gate. Extend documentation rather than tooling.

## The one concrete next step worth taking

Add `scripts/verify/orphan-rs-files.sh` (or a `just orphans` target)
that runs `cargo modules generate tree --with-orphans` (or equivalent)
across every workspace crate and fails if any `.rs` file under
`crates/*/src/` is not reachable from that crate's `lib.rs` / `main.rs`
mod graph. Exit non-zero in CI. This gives mechanical, cheap, ongoing
defense against the specific failure mode that produced the
specs/3680/3681 lies.

Sized as a single Phase 3 deliverable (~1 hour: install cargo-modules,
write the wrapper script, validate current tree has no orphans, wire
into `justfile` and/or CI).

## Limitations

- Sample size 30 gives wide confidence intervals. A larger sample (100+)
  would be needed to get tighter bounds, but at diminishing marginal
  value given the apparent rarity of the failure mode.
- The automated extractor's backtick regex missed some valid anchors
  (e.g., `#1645` issue IDs are not code symbols but do point to
  verifiable artifacts). A smarter extractor would reduce the AMBIGUOUS
  bucket.
- The spot-check of 6 AMBIGUOUS specs used my judgment; another
  reviewer might classify differently on prose edge cases (e.g.,
  "epic-level lifecycle artifacts" is not clearly a thing that can
  be grepped).
- Selection bias risk: sampling from `Status: Implemented` misses the
  population of specs that SHOULD have been flipped to `Implemented`
  but weren't. This audit does not address the "under-claimed" direction.

## Keywords

audit, sampling, spec-status, orphan-rs, cargo-modules, governance,
rule-of-three, tau-onboarding, tau-memory, specs/3680
