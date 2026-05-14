# Tasks: Issue #2905 - ops memory search relevant result contracts

1. [x] T1 (RED): add failing `functional_spec_2905_*` UI tests for memory panel/form/query/result/empty-state markers.
2. [x] T2 (RED): add failing `integration_spec_2905_*` gateway tests seeding persisted memory entries and asserting relevant `/ops/memory` rows.
3. [x] T3 (GREEN): implement minimal memory search snapshot plumbing + memory panel rendering markers.
4. [x] T4 (GREEN): add Memory Scope and empty-state graph node/edge availability markers for zero-result pages.
5. [x] T5 (GREEN): add bounded graph-backed node preview with summaries, navigable relation context, preserved graph-detail return anchors, selected returned-row markers, out-of-preview recovery, and not-in-scope return markers on zero-result Memory Explorer pages.
6. [x] T6 (REGRESSION): rerun selected chat/session/detail/ops suites (`spec_2802`, `spec_2830`, `spec_2834`, `spec_2838`, `spec_2842`, `spec_2846`, `spec_2885`, `spec_2889`, `spec_2893`, `spec_2897`, `spec_2901`).
7. [x] T7 (VERIFY): run fmt/clippy/scoped tests/mutation + sanitized live validation.
8. [x] T8 (GREEN): preserve unresolved Memory Graph `detail_memory_id` requests as explicit not-found detail state instead of rendering a generic empty detail panel.
