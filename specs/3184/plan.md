# Plan: Issue #3184 - deterministic directory-path rejection for browser DID report writer

## Approach
1. Add RED conformance test C-01 expecting deterministic directory-path rejection.
2. Run targeted tests to capture RED failure under current behavior.
3. Implement minimal pre-write `path.is_dir()` guard in `write_browser_did_init_report`.
4. Re-run targeted tests to GREEN and execute crate verification checks.

## Affected Modules
- `crates/kamn-sdk/src/lib.rs`
- `specs/milestones/m223/index.md`
- `specs/3184/spec.md`
- `specs/3184/plan.md`
- `specs/3184/tasks.md`

## Risks & Mitigations
- Risk: changing error surface for callers expecting raw OS errors.
  - Mitigation: deterministic message improves contract stability and still fails closed.
- Risk: accidental behavior change for valid file paths.
  - Mitigation: preserve existing nested-write integration test path and rerun.

## Interfaces / Contracts
- `write_browser_did_init_report` error contract for invalid directory output path.
- Existing JSON output contract for valid file paths.

## ADR
No ADR required (single-module fail-closed hardening).
