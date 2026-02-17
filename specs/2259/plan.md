# Plan #2259

Status: Reviewed
Spec: specs/2259/spec.md

## Approach

1. Add PostgreSQL client dependency to `tau-session` (subject to repo contract approval
   for new dependency).
2. Introduce backend helpers in `session_storage.rs`:
   - DSN resolution for `TAU_SESSION_POSTGRES_DSN`
   - stable session key derivation from store path
   - schema bootstrap for entries + usage tables
3. Implement `read_session_entries` and `write_session_entries_atomic` Postgres branches
   using transaction-scoped delete+insert semantics to match existing backend behavior.
4. Implement Postgres-backed usage summary read/write paths so
   `record_usage_delta` persists in the same backend.
5. Add conformance tests first (RED), then implement minimal code (GREEN), then run
   regressions across JSONL/SQLite tests.

## Affected Modules

- `crates/tau-session/Cargo.toml`
- `crates/tau-session/src/session_storage.rs`
- `crates/tau-session/src/session_store_runtime.rs` (only if backend-specific usage flow
  requires targeted changes)
- `crates/tau-session/src/tests.rs`

## Risks and Mitigations

- Risk: New dependency introduces compile/CI surface area.
  - Mitigation: add minimal dependency set and keep API usage synchronous to align with
    existing `SessionStore` contract.
- Risk: Postgres integration tests can be flaky without a test database.
  - Mitigation: gate live Postgres integration tests behind
    `TAU_TEST_POSTGRES_DSN`; keep non-live unit/functional checks always-on.
- Risk: Session-key collisions across different path spellings.
  - Mitigation: canonicalize when possible and use normalized path string as key.

## Interfaces / Contracts

- Existing `SessionStore` public API remains unchanged.
- Backend selection contract remains env-driven:
  - `TAU_SESSION_BACKEND=postgres`
  - `TAU_SESSION_POSTGRES_DSN=<dsn>`
- Postgres schema is internal and auto-created by `tau-session` runtime.
