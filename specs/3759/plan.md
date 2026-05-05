# Plan: Issue #3759 - Harness static preview action guard

## Cleanup Plan

1. Preserve all existing harness form actions and methods.
2. Add a small harness-local guard that activates only under `window.location.protocol === "file:"`.
3. Store blocked preview state on the submitted form and a hidden status element for tests and accessibility.
4. Avoid visible explanatory chrome so the operator UI stays dense.
5. Verify with targeted dashboard tests and an in-app browser file-preview click.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3759/`

## Risks / Mitigations

- Risk: preview guard accidentally blocks real gateway actions. Mitigation: guard exits unless the protocol is exactly `file:`.
- Risk: script becomes global UI behavior. Mitigation: attach it to the harness panel and require forms to be inside that panel.
- Risk: static string tests miss actual click behavior. Mitigation: regenerate standalone SSR HTML and validate with Browser Use.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3759`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3756 functional_spec_3757 functional_spec_3758`
- Regression: `cargo test -p tau-dashboard-ui`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Visual/interaction: Browser Use opens rendered `file://` HTML, clicks Run Benchmark, and confirms URL plus blocked-state marker.
