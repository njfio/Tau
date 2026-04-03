# Plan: Issue #3731 - Hoist action history I/O out of per-attempt loop

## Approach
1. Add in-memory helpers in `learning_runtime.rs`:
   - append records to an existing `ActionHistoryStore`
   - render a bulletin from an existing `ActionHistoryStore`
2. Load the action-history store once at the start of
   `execute_openresponses_request`.
3. Thread the mutable store through the retry loop so attempt persistence and
   bulletin refresh reuse the same in-memory state.
4. Save the store once after the retry loop finishes, before returning the final
   request outcome.

## Risks / Mitigations
- Risk: action-history updates could be lost on error paths.
  Mitigation: save once immediately after the retry loop exits, before
  propagating the result.
- Risk: bulletin rendering diverges from the old disk-backed path.
  Mitigation: add a unit test comparing in-memory rendering against the
  persisted-store rendering helper.

## Verification
- `cargo test -p tau-gateway unit_append_gateway_action_history_records_updates_store_without_disk_io -- --nocapture`
- `cargo test -p tau-gateway unit_render_gateway_learning_bulletin_from_store_matches_disk_render -- --nocapture`
- `cargo check -p tau-gateway`
- `cargo fmt --check`
