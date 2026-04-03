# Plan: Issue #3740 - Restore tau-provider cli_executable clippy hygiene

## Approach
1. Remove the unused env-sanitization helper symbols from `cli_executable.rs` and `credential_store.rs`.
2. Keep the live executable-discovery and credential behavior unchanged.
3. Verify with targeted clippy and existing executable-discovery tests.

## Risks / Mitigations
- Risk: accidentally removing live behavior instead of dead code.
  Mitigation: touch only the unused symbols and rerun the existing `is_executable_available` tests.

## Verification
- `cargo clippy -p tau-provider --all-targets --all-features -- -D warnings`
- `cargo test -p tau-provider is_executable_available -- --test-threads=1`
- `cargo fmt --check`
