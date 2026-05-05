# Plan 3786: Proof Header Metadata No-Wrap

## Approach

Add a scoped metadata fit contract to the Mission Detail proof header:

- Mark the proof header as `no-wrap` for compact metadata.
- Override the general panel `overflow-wrap: anywhere` rule for proof-header metadata values.
- Tighten the metadata grid gap/columns enough that five metadata pairs still fit in the compact header area.
- Preserve the existing metadata labels and values.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3786/`

## Risks and Mitigations

- Risk: No-wrap metadata values could force horizontal overflow.
  Mitigation: Scope the grid to compact max-content columns and verify document overflow plus proof-window viewport containment in browser geometry.

- Risk: Tightening metadata spacing could reduce readability.
  Mitigation: Keep labels and values aligned in the same two-column definition list and change only the proof header scope.

## Interfaces

No Rust API, route, data, or wire-format changes. This is a rendered HTML/CSS contract.

## Verification

- Red/green conformance test for `functional_spec_3786`.
- Existing 376x, 377x, and 378x harness regression tests.
- Full `tau-dashboard-ui` crate tests, fmt, clippy, gateway integration, and browser geometry.
