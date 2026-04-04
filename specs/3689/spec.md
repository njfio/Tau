# Spec: Issue #3689 - Add fuzzy command filtering to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports autocomplete, aliases, paging,
selected-command previews, grouped sections, and argument scaffolding, but
lookup still depends on relatively exact text. Operators still need close
prefixes or explicit substrings to discover commands, which slows down the
launcher under real terminal use. A stronger REPL palette should tolerate
shorter, fuzzier queries while still prioritizing exact and prefix matches.

## Scope
In scope:
- add bounded fuzzy/subsequence matching for command palette lookups
- keep exact, alias, and prefix matches ahead of fuzzy fallbacks
- preserve grouped rendering, preview, and paging behavior with fuzzy matches
- add focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3689/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- full typo-correction or edit-distance ranking
- dynamic command registration
- mouse interactions
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 The command palette matches useful sparse queries
Given the operator types a sparse query that preserves command character order,
when the palette filters commands,
then Tau returns relevant command matches even if the query is not a direct
prefix or substring.

### AC-2 Stronger matches still outrank fuzzy fallbacks
Given the operator types a query that has exact, alias, prefix, and fuzzy
subsequence candidates,
when the palette orders matches,
then Tau keeps stronger match classes ahead of weaker fuzzy results.

### AC-3 Grouped rendering and selected previews remain stable under fuzzy results
Given the palette is showing fuzzy-matched results,
when it renders grouped commands and the selected preview,
then Tau still shows the correct section labels, selection, and preview for the
filtered result set.

### AC-4 Earlier M335 REPL slices do not regress
Given the existing runtime control, history, persistence, transcript export,
palette autocomplete/history, aliases, paging, previews, grouping, and
scaffolding slices,
when fuzzy filtering lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Sparse subsequence queries return relevant commands. Maps to AC-1. Tier:
  Functional.
- C-02 Stronger match classes outrank fuzzy fallbacks. Maps to AC-2. Tier:
  Functional.
- C-03 Group labels and preview remain correct under fuzzy-filtered results.
  Maps to AC-3. Tier: Functional.
- C-04 Existing M335 `interactive::app_gateway_tests` continue to pass after
  fuzzy-filtering changes. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can find commands with fewer exact keystrokes
- Exact and prefix matches still feel deterministic
- Grouped palette render stays stable under fuzzy filtering

## Key Decisions
- Fuzzy matching is bounded to simple subsequence logic
- Exact, alias, and prefix matches remain strictly prioritized ahead of fuzzy
  matches
- Fuzzy filtering changes only result selection, not command execution semantics
