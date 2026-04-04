# Plan: Issue #3709 - Match multi-token `tau-tui` command palette queries across command metadata

Status: Reviewed

## Approach
Extend command-palette matching with a combined normalized metadata view that
lets multi-token queries match across command fields instead of requiring one
field to contain the whole normalized query. Keep the existing exact/prefix/
contains/fuzzy structure intact so the slice only broadens metadata discovery
for multi-token searches without changing the visible command catalog or command
submission behavior.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - add multi-token metadata matching alongside the existing field-by-field
    comparison
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for multi-token metadata-aware command matching

## Contracts
- `copy assistant` can surface `copy-last`
- `save path` can surface `save-transcript`
- Existing exact/prefix/fuzzy, separator-normalized, and explicit scaffold
  submission behavior remains intact

## Risks
- Combined metadata matching could become too broad if token requirements are
  not strict enough
- Ranking should remain stable even when more commands become contains matches
- The new matcher should not reintroduce the scaffold-submission regression
  fixed in `#3707`

## Verification Strategy
- Add failing tests first for multi-token metadata-aware command matching
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
