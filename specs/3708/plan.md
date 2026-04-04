# Plan: Issue #3708 - Normalize separator punctuation in `tau-tui` command palette queries

Status: Reviewed

## Approach
Extend command-palette matching with a lightweight normalization pass that
treats separator punctuation like spaces and hyphens equivalently when comparing
queries against command names, usage, and scaffold text. Keep the existing
exact/prefix/fuzzy structure intact so the slice only improves match tolerance
for separator variations rather than changing ranking strategy.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - add separator-normalized comparison alongside existing match checks
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for space-vs-hyphen query normalization

## Contracts
- `copy last` can surface `copy-last`
- `mission id` can surface commands using `<mission-id>`
- Existing exact/prefix/fuzzy and explicit scaffold submission behavior remains
  intact

## Risks
- Over-normalization could make unrelated commands collide too often
- Ranking should remain stable even when more commands become match candidates
- Separator normalization must coexist cleanly with the usage/scaffold matching
  added in `#3707`

## Verification Strategy
- Add failing tests first for space-vs-hyphen query matching
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
