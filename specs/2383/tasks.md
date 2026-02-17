# Tasks: Issue #2383

## T1 (Red): Spec-derived tests first
- [x] Add C-01 parser test for `/tau skip [reason]`.
- [x] Add C-02 functional runtime test asserting skip suppresses outbound delivery and logs status.
- [x] Add C-04 help text test for skip command visibility.
- [x] Add C-03 regression assertion for existing `/tau status` flow.

## T2 (Green): Minimal implementation
- [x] Extend command enum/parser/render/help with skip support.
- [x] Add command execution metadata to flag suppressed delivery for skip.
- [x] Add outbound branch for skip status logging and no delivery dispatch.

## T3 (Refactor)
- [x] Keep skip reason code/status constants centralized with command constants.
- [x] Keep non-skip outbound path structurally unchanged.

## T4 (Verify)
- [x] `cargo fmt --check`
- [x] `CARGO_TARGET_DIR=target-fast-2383 cargo clippy -p tau-multi-channel -- -D warnings`
- [x] `CARGO_TARGET_DIR=target-fast-2383 cargo test -p tau-multi-channel unit_parse_multi_channel_tau_command_supports_initial_command_set -- --nocapture`
- [x] `CARGO_TARGET_DIR=target-fast-2383 cargo test -p tau-multi-channel functional_runner_executes_tau_skip_command_without_outbound_delivery -- --nocapture`
- [x] `CARGO_TARGET_DIR=target-fast-2383 cargo test -p tau-multi-channel functional_runner_executes_tau_status_command_and_persists_command_metadata -- --nocapture`
