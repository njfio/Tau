# Plan: Issue #3707 - Match `tau-tui` command palette queries against usage and scaffold text

Status: Reviewed

## Approach
Extend `command_palette_matches` so non-empty queries also compare against each
command's `usage` and scaffold text during contains-style matching. Keep exact,
prefix, and fuzzy name/alias behavior intact so ranking stays predictable while
making argument-token discovery stronger. Limit the slice to filtering logic and
direct regression coverage.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - include `usage` and scaffold text in command-palette match evaluation
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for usage-token and scaffold-token matching

## Contracts
- Querying tokens found only in `usage` can surface matching commands
- Querying scaffold placeholder tokens like `mission-id` can surface matching
  commands
- Existing exact/prefix/fuzzy name and alias behavior remains intact
- Existing preview/grouping behavior continues to work

## Risks
- New match sources must not drown out stronger exact/prefix matches
- Usage/scaffold matching should not accidentally make every command match
  generic punctuation-heavy queries
- Existing filter ordering should remain stable enough for current tests

## Verification Strategy
- Add failing tests first for usage-token and scaffold-token command matching
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
