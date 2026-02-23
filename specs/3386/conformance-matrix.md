# Conformance Matrix: Issue #3386

| Scenario | Priority | Status | Test(s) | Notes |
|---|---|---|---|---|
| A2-01 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-02 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-03 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-04 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-05 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-06 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-07 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-08 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-09 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| A2-10 | P0 | Covered | tier_pr_a2_agent_session_flow_matrix | Covers non-stream/stream/session continuity/append/reset/context-pressure. |
| B6-01 | P1 | Covered | integration_spec_2885_c02_c03_c04_ops_sessions_branch_creates_lineage_derived_target_session | Ops branching behavior covered in existing integration tests. |
| B6-02 | P1 | Covered | integration_spec_2885_c02_c03_c04_ops_sessions_branch_creates_lineage_derived_target_session | Ops branching behavior covered in existing integration tests. |
| B6-03 | P1 | Covered | tier_nightly_b6_tool_navigation_matrix | Branch tool-call flow creates target branch session with lineage and prompt continuity. |
| B6-04 | P1 | Covered | tier_nightly_b6_tool_navigation_matrix | Undo tool-call flow reports applied transition and updates session navigation state. |
| B6-05 | P1 | Covered | tier_nightly_b6_tool_navigation_matrix | Redo tool-call flow reports applied transition and restores active branch head. |
| B6-06 | P1 | Covered | integration_openresponses_http_roundtrip_persists_session_state | Session persistence validated across requests; restart lifecycle not exposed as testable API event. |
| C5-01 | P1 | N/A | n/a | Inbound channel webhook/polling message ingestion endpoints are outside current gateway-openresponses route surface. |
| C5-02 | P1 | N/A | n/a | Inbound channel webhook/polling message ingestion endpoints are outside current gateway-openresponses route surface. |
| C5-03 | P1 | N/A | n/a | Inbound channel webhook/polling message ingestion endpoints are outside current gateway-openresponses route surface. |
| C5-04 | P1 | N/A | n/a | Inbound channel webhook/polling message ingestion endpoints are outside current gateway-openresponses route surface. |
| C5-05 | P1 | Covered | tier_nightly_p1_runtime_matrix | Lifecycle endpoint coverage via logout/status action contract. |
| C5-06 | P1 | Covered | tier_nightly_p1_runtime_matrix | Lifecycle endpoint coverage via logout/status action contract. |
| C5-07 | P1 | N/A | n/a | Inbound channel webhook/polling message ingestion endpoints are outside current gateway-openresponses route surface. |
| C5-08 | P1 | N/A | n/a | Inbound channel webhook/polling message ingestion endpoints are outside current gateway-openresponses route surface. |
| CH15-01 | P3 | Covered | tier_weekly_ch15_chaos_matrix | Timeout/malformed provider/flood/disconnect chaos paths covered. |
| CH15-02 | P3 | Covered | tier_weekly_ch15_chaos_matrix | Timeout/malformed provider/flood/disconnect chaos paths covered. |
| CH15-03 | P3 | Covered | tier_weekly_ch15_chaos_matrix | Timeout/malformed provider/flood/disconnect chaos paths covered. |
| CH15-04 | P3 | Covered | tier_weekly_ch15_chaos_matrix | Timeout/malformed provider/flood/disconnect chaos paths covered. |
| CH15-05 | P3 | Covered | tier_weekly_ch15_chaos_matrix | Session lock-contention retry path is exercised with deterministic lock-file release and persisted-session assertions. |
| CH15-06 | P3 | Covered | tier_weekly_ch15_chaos_matrix | High session-cardinality pressure run (`100` sessions, multi-turn history) validates persistence and post-pressure responsiveness. |
| D12-01 | P2 | Covered | tier_nightly_p2_observability_matrix | Dashboard status/widgets/alerts/stream/timeline covered. |
| D12-02 | P2 | Covered | tier_nightly_p2_observability_matrix | Dashboard status/widgets/alerts/stream/timeline covered. |
| D12-03 | P2 | Covered | tier_nightly_p2_observability_matrix | Dashboard status/widgets/alerts/stream/timeline covered. |
| D12-04 | P2 | Covered | tier_nightly_p2_observability_matrix | Dashboard status/widgets/alerts/stream/timeline covered. |
| D12-05 | P2 | Covered | integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates | SSE reconnect behavior covered in existing integration test. |
| D12-06 | P2 | Covered | tier_nightly_p2_observability_matrix | Dashboard status/widgets/alerts/stream/timeline covered. |
| E14-01 | P2 | Covered | functional_tool_builder_tool_builds_wasm_artifacts | Tool builder generates deterministic WASM/module/manifest artifacts. |
| E14-02 | P2 | Covered | integration_tool_builder_generated_tool_executes_through_extension_runtime | Generated tool executes through extension runtime with deterministic success payload. |
| E14-03 | P2 | Covered | regression_build_generated_wasm_tool_fails_closed_when_wasm_spins_forever | Misbehaving infinite-loop WASM fails closed via sandbox validation guardrails. |
| E14-04 | P2 | Covered | integration_spec_3396_c01_c02_gateway_tools_inventory_includes_mcp_prefixed_tool | MCP-prefixed tool registrations are surfaced by `/gateway/tools` inventory contract. |
| F10-01 | P1 | Covered | functional_spec_3400_c01_primary_success_returns_without_fallback_invocation | Primary route success is returned directly without fallback route invocation. |
| F10-02 | P1 | Covered | functional_fallback_client_handoffs_on_retryable_error_and_emits_event | Retryable primary failure falls back and returns secondary response. |
| F10-03 | P1 | Covered | functional_spec_3400_c03_all_routes_fail_returns_terminal_error | Retryable failures across all configured routes return deterministic terminal error behavior. |
| F10-04 | P1 | Covered | functional_circuit_breaker_opens_and_skips_temporarily_unhealthy_route | Circuit opens after threshold failures and skips unhealthy primary route. |
| F10-05 | P1 | Covered | integration_circuit_breaker_retries_primary_after_cooldown_expires | Half-open cooldown probe retries primary route and recovers when healthy. |
| F10-06 | P1 | Covered | tier_nightly_p1_runtime_matrix | Rate-limit enforcement exercised. |
| F10-07 | P1 | Covered | tier_nightly_p1_runtime_matrix | Rate-limit enforcement exercised. |
| F10-08 | P1 | Covered | functional_fallback_client_handoffs_on_retryable_error_and_emits_event | Fallback telemetry event contract (`from_model`/`to_model`/error metadata) is asserted. |
| G1-01 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| G1-02 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| G1-03 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| G1-04 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| G1-05 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| G1-06 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| G1-07 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| G1-08 | P0 | Covered | tier_pr_g1_gateway_lifecycle_matrix | Covers lifecycle/auth/models/stream completion. |
| K13-01 | P1 | Covered | tier_nightly_p1_runtime_matrix | Password-session token issuance and protected endpoint access covered. |
| K13-02 | P1 | Covered | tier_nightly_p1_runtime_matrix | Password-session token issuance and protected endpoint access covered. |
| K13-03 | P1 | Covered | tier_nightly_p1_runtime_matrix | Password-session token issuance and protected endpoint access covered. |
| K13-04 | P1 | Covered | regression_gateway_password_session_token_expires_and_fails_closed | Token expiry rejection covered by existing regression test. |
| K13-05 | P1 | Covered | functional_spec_2786_c01_gateway_auth_bootstrap_endpoint_reports_token_mode_contract | Auth bootstrap contract covered. |
| K13-06 | P1 | N/A | n/a | Credential rotation API route is not part of current gateway-openresponses endpoint contract. |
| M7-01 | P1 | Covered | tier_nightly_p1_runtime_matrix | Memory CRUD and graph endpoint coverage. |
| M7-02 | P1 | Covered | tier_nightly_p1_runtime_matrix | Memory CRUD and graph endpoint coverage. |
| M7-03 | P1 | Covered | tier_nightly_p1_runtime_matrix | Memory CRUD and graph endpoint coverage. |
| M7-04 | P1 | Covered | tier_nightly_p1_runtime_matrix | Memory CRUD and graph endpoint coverage. |
| M7-05 | P1 | Covered | tier_nightly_p1_runtime_matrix | Memory CRUD and graph endpoint coverage. |
| M7-06 | P1 | Covered | tier_nightly_p1_runtime_matrix | Memory CRUD and graph endpoint coverage. |
| M7-07 | P1 | Covered | integration_spec_2909_c01_c02_c03_ops_memory_scope_filters_narrow_results | Scope filtering covered by existing ops integration tests. |
| M7-08 | P1 | Covered | integration_spec_2913_c01_c02_c03_ops_memory_type_filter_narrows_results | Type filtering covered by existing ops integration tests. |
| O3-01 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| O3-02 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| O3-03 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| O3-04 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| O3-05 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| O3-06 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | Request-side `tools`/`tool_choice` now fail closed with deterministic `unsupported_tools` client error (no silent ignore). |
| O3-07 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | Tool-role continuation message content is preserved and forwarded into provider request context. |
| O3-08 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | Multi-choice requests (`n > 1`) fail gracefully with deterministic `unsupported_n` client error. |
| O3-09 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| O3-10 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | `max_tokens` is forwarded to provider request and OpenAI payload finish reason reflects provider completion reason (e.g., `length`). |
| O3-11 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| O3-12 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | OpenAI compatibility surfaces covered. |
| R8-01 | P1 | N/A | n/a | Per-rollout reward-dimension and optimizer interval internals are not exposed through stable gateway contract assertions. |
| R8-02 | P1 | N/A | n/a | Per-rollout reward-dimension and optimizer interval internals are not exposed through stable gateway contract assertions. |
| R8-03 | P1 | N/A | n/a | Per-rollout reward-dimension and optimizer interval internals are not exposed through stable gateway contract assertions. |
| R8-04 | P1 | N/A | n/a | Per-rollout reward-dimension and optimizer interval internals are not exposed through stable gateway contract assertions. |
| R8-05 | P1 | N/A | n/a | Per-rollout reward-dimension and optimizer interval internals are not exposed through stable gateway contract assertions. |
| R8-06 | P1 | Covered | tier_nightly_p1_runtime_matrix | Training status/rollouts endpoint coverage. |
| R8-07 | P1 | Covered | tier_nightly_p1_runtime_matrix | Training status/rollouts endpoint coverage. |
| R8-08 | P1 | N/A | n/a | Per-rollout reward-dimension and optimizer interval internals are not exposed through stable gateway contract assertions. |
| S11-01 | P0 | Covered | tier_pr_s11_safety_endpoint_matrix | Safety policy/rules/test endpoints and redaction/blocking behavior. |
| S11-02 | P0 | Covered | tier_pr_s11_safety_endpoint_matrix | Safety policy/rules/test endpoints and redaction/blocking behavior. |
| S11-03 | P0 | Covered | tier_pr_s11_safety_endpoint_matrix | Safety policy/rules/test endpoints and redaction/blocking behavior. |
| S11-04 | P0 | Covered | tier_pr_s11_safety_endpoint_matrix | Safety policy/rules/test endpoints and redaction/blocking behavior. |
| S11-05 | P0 | Covered | tier_pr_s11_safety_endpoint_matrix | Safety policy/rules/test endpoints and redaction/blocking behavior. |
| S11-06 | P0 | Covered | tier_pr_s11_safety_endpoint_matrix | Safety policy/rules/test endpoints and redaction/blocking behavior. |
| T4-01 | P0 | Covered | tier_pr_t4_tool_pipeline_executes_read_write_edit_sequence | Gateway->agent->tool pipeline for filesystem tools. |
| T4-02 | P0 | Covered | tier_pr_t4_tool_pipeline_executes_read_write_edit_sequence | Gateway->agent->tool pipeline for filesystem tools. |
| T4-03 | P0 | Covered | tier_pr_t4_tool_pipeline_executes_read_write_edit_sequence | Gateway->agent->tool pipeline for filesystem tools. |
| T4-04 | P0 | Covered | tier_pr_t4_memory_write_and_search_roundtrip | Memory write/search via tool loop. |
| T4-05 | P0 | Covered | tier_pr_t4_http_and_error_paths_continue_without_crash | HTTP tool path + error continuation. |
| T4-06 | P0 | Covered | tier_pr_t4_http_and_error_paths_continue_without_crash | HTTP tool path + error continuation. |
| T4-07 | P0 | Covered | tier_pr_t4_policy_and_protected_path_enforcement | Policy/protected-path enforcement in tool loop. |
| T4-08 | P0 | Covered | tier_pr_t4_tool_pipeline_executes_read_write_edit_sequence | Multi-tool sequence execution ordering. |
| T4-09 | P0 | Covered | tier_pr_t4_policy_and_protected_path_enforcement | Policy/protected-path enforcement in tool loop. |
| T4-10 | P0 | Covered | tier_pr_t4_jobs_create_and_status_roundtrip | Jobs create/status via tool loop. |
| X9-01 | P2 | Covered | tier_nightly_p2_observability_matrix | Cortex chat/status endpoint coverage. |
| X9-02 | P2 | Covered | tier_nightly_p2_observability_matrix | Cortex chat/status endpoint coverage. |
| X9-03 | P2 | Covered | integration_spec_2704_c01_c02_c05_cortex_status_endpoint_reports_tracked_runtime_events | Bulletin/event synthesis surfaced in existing cortex status integration coverage. |
| X9-04 | P2 | Covered | integration_spec_2717_c04_gateway_new_session_prompt_includes_latest_cortex_bulletin_snapshot | Bulletin injection into future session prompt covered. |
| X9-05 | P2 | Covered | integration_spec_2953_c01_c02_c04_cortex_chat_uses_llm_output_with_context_markers_and_stable_sse_order | Context composition markers covered. |
| X9-06 | P2 | Covered | regression_spec_2953_c03_c04_cortex_chat_provider_failure_uses_deterministic_fallback_and_reason_code | Fallback behavior covered. |
