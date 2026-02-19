# Plan #2573

## Approach
1. Run mapped #2572 conformance/regression tests and capture pass evidence.
2. Run scoped `cargo fmt --check`, `cargo clippy -- -D warnings`, and crate tests.
3. Run `cargo mutants --in-diff` for touched phase-4 paths and resolve any escapes.
4. Run sanitized `provider-live-smoke` and capture summary output.
5. Update issue logs/checklists and package evidence in PR.

## Risks & Mitigations
- Risk: mutation escapes reveal coverage gaps.
  - Mitigation: add targeted regression tests before finalizing PR.
- Risk: provider key drift causes live-smoke failures.
  - Mitigation: use sanitized keyfile and deterministic skip strategy.

## Interfaces / Contracts
- Verification-only subtask; no external behavior changes expected.
