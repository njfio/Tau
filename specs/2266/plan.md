# Plan #2266

Status: Reviewed
Spec: specs/2266/spec.md

## Approach

1. Add conformance-first tests in `tau-ops` daemon runtime test module:
   - explicit spec-named coverage for systemd template sections and lifecycle checks.
   - failing regression for executable-path-with-spaces in `ExecStart`.
2. Implement minimal rendering fix for systemd executable argument formatting.
3. Run scoped validation for `tau-ops` (`fmt`, `clippy`, targeted tests, full crate
   tests).

## Affected Modules

- `crates/tau-ops/src/daemon_runtime.rs`
- `specs/2266/spec.md`
- `specs/2266/plan.md`
- `specs/2266/tasks.md`

## Risks and Mitigations

- Risk: overly aggressive quoting could alter standard service rendering.
  - Mitigation: preserve existing output for normal paths; add regression tests for
    both standard and spaced paths.
- Risk: lifecycle regressions from unrelated daemon behavior.
  - Mitigation: run existing full `tau-ops` test suite after change.

## Interfaces / Contracts

- Public API remains unchanged:
  - `render_systemd_user_unit(...) -> String`
  - daemon lifecycle functions in `tau-ops` (`install/start/stop/uninstall/inspect`)
- Behavior contract tightened around `ExecStart` binary-path safety.
