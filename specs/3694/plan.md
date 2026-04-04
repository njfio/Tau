# Plan: Issue #3694 - Auto-focus first placeholder when command-palette scaffolds a parameterized command

Status: Reviewed

## Approach
Extend the existing autocomplete path so, when the selected command scaffold
contains unresolved placeholders, Tau both inserts the scaffold and activates
the first placeholder immediately. Reuse the existing placeholder-span state
and keep simple commands on the current no-placeholder path.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - update scaffold autocomplete to prime the first placeholder when the
    inserted scaffold is parameterized
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for canonical and alias-driven single-Tab placeholder
    focus behavior

## Contracts
- Parameterized command autocomplete inserts the scaffold and makes the first
  unresolved placeholder active
- Alias-selected parameterized autocomplete still uses the canonical scaffold
  and activates its first placeholder
- Non-parameterized command autocomplete still leaves no active placeholder

## Risks
- The new auto-focus path must not regress the explicit placeholder cycling from
  `#3692` and `#3693`
- Autocomplete must remain predictable for simple commands and not synthesize
  phantom placeholder state
- Cursor state needs to stay coherent after autocomplete and subsequent typing

## Verification Strategy
- Add failing tests first for canonical and alias-driven auto-focus behavior
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
