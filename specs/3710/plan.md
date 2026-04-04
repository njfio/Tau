# Plan: Issue #3710 - Match tau-tui command palette queries against section labels

Status: Reviewed

## Goal
Teach the `tau-tui` command palette matcher to consider each command's section
label as part of the searchable metadata so grouped operator concepts like
`Clipboard` and `Runtime` become discoverable through the same filter flow.

## Approach
1. Extend `command_palette_matches` in
   `crates/tau-tui/src/interactive/app_commands.rs` so section labels
   participate in contains-tier and multi-token metadata matching.
2. Keep exact, prefix, compact, and fuzzy ordering intact so section matching
   broadens discovery without destabilizing established ranking behavior.
3. Add focused RED/GREEN tests in
   `crates/tau-tui/src/interactive/app_gateway_tests.rs` for cross-field queries
   that require section participation.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `specs/3710/spec.md`
- `specs/3710/tasks.md`
- `specs/milestones/m335/index.md`

## Risks / Mitigations
- Risk: section terms could over-broaden results.
  Mitigation: keep section-aware matching in the existing contains tier rather
  than promoting it to exact or prefix ranking.
- Risk: explicit scaffold submission behavior could regress.
  Mitigation: keep the change isolated to filter logic and run the existing
  `interactive::app_gateway_tests` regression suite.

## Interfaces / Contracts
- No external API changes.
- `command_palette_matches` remains the single filter entry point for command
  palette suggestion lookup.
