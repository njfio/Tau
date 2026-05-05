# Plan: Issue #3760 - Ops shell static preview navigation guard

## Cleanup Plan

1. Preserve all existing `href` route contracts.
2. Add a small shell-local click guard that activates only for `file://` previews.
3. Guard only absolute local links inside `#tau-ops-shell`; allow hash and external links to behave normally.
4. Store blocked preview state on the clicked link and a hidden live status element.
5. Verify with targeted dashboard UI tests and an in-app browser click.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3760/`

## Risks / Mitigations

- Risk: link guard blocks real gateway navigation. Mitigation: guard exits unless `window.location.protocol === "file:"`.
- Risk: guard blocks anchors or external links. Mitigation: guard only handles one-slash absolute local links.
- Risk: static string tests miss actual click behavior. Mitigation: validate with Browser Use against generated SSR HTML.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3760`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3759`
- Regression: `cargo test -p tau-dashboard-ui`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Interaction: Browser Use opens rendered `file://` HTML, clicks Agent Fleet, and confirms URL plus blocked-link marker.
