# Plan 3787: Proposal Patch Summary Readability

## Approach

Add a scoped summary readability contract to the selected proposal detail:

- Mark the proposal detail as `full-text` for summary fit.
- Mark only the Patch Summary value as the row allowed to wrap.
- Tighten the compact detail row spacing enough that all seven rows stay visible.
- Preserve existing proposal data, operator actions, policy, audit, and TUI layout.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3787/`

## Risks and Mitigations

- Risk: Wrapping the Patch Summary row could push the audit log or TUI companion out of the first viewport.
  Mitigation: Keep the change scoped to one row, reduce proposal detail gaps slightly, and verify viewport geometry.

- Risk: Making all detail values wrap would reduce scannability.
  Mitigation: Only the long Patch Summary value is allowed to wrap; short safety and evidence rows remain compact.

## Interfaces

No Rust API, route, data, or wire-format changes. This is a rendered HTML/CSS contract.

## Verification

- Red/green conformance test for `functional_spec_3787`.
- Existing 377x and 378x harness regression tests.
- Full `tau-dashboard-ui` crate tests, fmt, clippy, gateway integration, and browser geometry.
