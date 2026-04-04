# Plan: Issue #3682 - Add transcript export commands to `tau-tui` REPL

Status: Implemented

## Approach
Extend the existing transcript copy helper into a full transcript export helper
that can either pipe the rendered transcript to the clipboard or write it to a
file. Keep the export format plain text and stable, and expose it via new slash
commands in the interactive command surface.

## Affected Modules
- `crates/tau-tui/src/interactive/app_copy_target.rs`
  - add transcript rendering + copy/save helpers
- `crates/tau-tui/src/interactive/app_commands.rs`
  - parse `/copy-transcript` and `/save-transcript [path]`
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - update help overlay for the new transcript export commands
- `crates/tau-tui/src/interactive/ui_status.rs`
  - update the compact help line
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/green coverage for copy/save success and failure paths

## Contracts
- `/copy-transcript` copies the current transcript text
- `/save-transcript` writes to a default path under `.tau/tui/exports/`
- `/save-transcript <path>` writes to an explicit path
- Export format includes timestamp, role label, and message content for each
  transcript entry

## Risks
- Large transcripts can produce big clipboard payloads; plain text is still the
  right default for operator portability
- Saving arbitrary paths should fail clearly without crashing if the path is
  invalid
- Export commands should not mutate transcript content themselves

## Verification Strategy
- Add failing tests first for transcript copy/save behavior and error handling
- Re-run existing `interactive::app_gateway_tests`
- Build `tau-tui` after the scoped tests pass
