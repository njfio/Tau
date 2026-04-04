# Plan: Issue #3689 - Add fuzzy command filtering to `tau-tui` command palette

Status: Implemented

## Approach
Extend the existing command matcher with a final subsequence fallback stage after
exact, alias, prefix, and contains checks. Preserve the current grouped render
and preview behavior by keeping the result type as the same command-entry list,
just with a stronger ranking pipeline.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - add subsequence fallback matching and ranking
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN tests for sparse fuzzy queries and ordering guarantees

## Contracts
- Exact and alias-exact matches still win immediately
- Prefix matches stay ahead of contains and fuzzy-subsequence matches
- Fuzzy results remain compatible with grouped rendering and selected previews

## Risks
- Overly permissive fuzzy matches could create noisy results
- Match ordering could drift if exact/prefix/fuzzy buckets are not kept separate
- Sparse queries should remain predictable rather than “magical”

## Verification Strategy
- Add failing tests first for sparse fuzzy lookup and ordering
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
