# Issue 3797 Plan

## Approach

Use the existing connector snapshot data already driving `/ops/channels`.
Enhance only the route presentation: add KPI cards, wrap the table in a
contained scrolling region, and render row actions as button-like links with
explicit enabled and disabled semantics.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks / Mitigations

- Risk: Existing channel action marker tests rely on stable attributes.
  Mitigation: Keep existing IDs and `data-action` / `data-action-enabled`
  attributes in the same order and add semantics after them.
- Risk: Table styling could affect other routes.
  Mitigation: Scope channel-specific CSS under `#tau-ops-channels-panel`.

## Interfaces / Contracts

- `data-visual-contract="channel-operator-console"` on the channels panel.
- `#tau-ops-channels-kpi-grid` with online, offline, and degraded cards.
- `#tau-ops-channels-table-wrap` with contained horizontal overflow.
- `.tau-ops-channel-actions` around login, logout, and probe controls.
- `role="button"` and `aria-disabled` on channel action links.
