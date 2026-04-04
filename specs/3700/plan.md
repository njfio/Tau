# Plan: Issue #3700 - Add placeholder-aware Left/Right cursor escape to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette cursor movement helpers so they special-case an
active scaffold placeholder before falling back to ordinary character-wise
movement. `Left` should clear placeholder focus but keep the cursor at the
placeholder's start boundary, while `Right` should clear placeholder focus and
jump to the placeholder's end boundary. Keep the scope limited to cursor escape
so it layers cleanly on top of the existing placeholder and editor core.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - update command-palette left/right cursor helpers to recognize an active
    placeholder span
- `crates/tau-tui/src/interactive/app_commands.rs`
  - keep `Left`/`Right` routing on the updated palette helpers
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for placeholder-aware left/right escape behavior

## Contracts
- `Left` from an active placeholder clears placeholder focus and stays at the
  placeholder start boundary
- `Right` from an active placeholder clears placeholder focus and moves to the
  placeholder end boundary
- Ordinary character-wise left/right movement remains unchanged when no
  placeholder is active
- Existing placeholder and editor flows continue to work

## Risks
- Placeholder span boundaries must stay coherent after cursor escape
- Character-wise movement must not regress for non-placeholder text
- Placeholder focus clearing must remain consistent with the rest of the editor

## Verification Strategy
- Add failing tests first for active-placeholder `Left` and `Right`
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
