# Tasks: Issue #3558 - Interactive start marker timeout clarity

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED): update/add runtime-loop marker contract test expecting both
   timeout fields.
2. [x] T2 (GREEN): wire request timeout through `InteractiveRuntimeConfig`
   construction and start marker emission.
3. [x] T3 (VERIFY): run focused `tau-coding-agent` tests for interactive marker
   behavior and timeout wiring.
4. [x] T4 (DOC): update README/operator guide marker examples.

## Test Tier Intent
| Tier | Planned |
| --- | --- |
| Unit | start-line formatter contract |
| Functional | interactive path config wiring |
| Regression | marker output contract stability |
| Integration | N/A (single-module behavioral correction) |

## TDD Evidence
### RED
- Prior behavior showed ambiguous start marker (`timeout_ms=0`) for active
  request-timeout runs; unit contract updated to fail under old format.

### GREEN
- `cargo test -p tau-coding-agent unit_interactive_turn_progress_line_contract_is_stable -- --nocapture`
- `cargo test -p tau-coding-agent interactive_turn_progress -- --nocapture`
- `cargo test -p tau-coding-agent regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent -- --nocapture`

### VERIFY
- `cargo fmt --check`
