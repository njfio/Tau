# Conformance Matrix: Issue #3400

| Scenario | AC | Case | Status | Test(s) / Evidence |
|---|---|---|---|---|
| F10-01 | AC-1 | C-01 | Covered | `functional_spec_3400_c01_primary_success_returns_without_fallback_invocation` |
| F10-02 | AC-2 | C-02 | Covered | `functional_fallback_client_handoffs_on_retryable_error_and_emits_event` |
| F10-03 | AC-2 | C-03 | Covered | `functional_spec_3400_c03_all_routes_fail_returns_terminal_error` |
| F10-04 | AC-3 | C-04 | Covered | `functional_circuit_breaker_opens_and_skips_temporarily_unhealthy_route` |
| F10-05 | AC-3 | C-05 | Covered | `integration_circuit_breaker_retries_primary_after_cooldown_expires` |
| F10-08 | AC-4 | C-06 | Covered | `functional_fallback_client_handoffs_on_retryable_error_and_emits_event` |
| F10 traceability | AC-5 | C-07 | Covered | `specs/3386/conformance-matrix.md` rows updated to Covered |
