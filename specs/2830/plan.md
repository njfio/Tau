# Plan: Issue #2830 - Chat message send and transcript visibility contracts

## Approach
1. Add/extend RED conformance tests for `/ops/chat` send-form markers and message visibility contracts in both UI and gateway layers.
2. Extend `tau-dashboard-ui` shell tests to assert deterministic chat form/transcript markers.
3. In `tau-gateway`, hydrate chat snapshot rows from active session lineage using query controls (`session`/`session_key`).
4. Add `POST /ops/chat/send` to append user messages and redirect to `/ops/chat` with preserved `theme`/`sidebar`/`session` controls.
5. Distinguish total session entries from rendered transcript rows when system entries are hidden from the operator transcript.
6. Add a form submit-event guard and backend `chat_status=empty-message` rejection path so blank sends are visibly rejected without creating or mutating session state.
7. Keep active compose controls above new-session creation and historical session selection so secondary navigation cannot bury the current chat action.
8. Group new-session creation and session history in a compact collapsed-by-default session manager while preserving the underlying contracts.
9. Move session summary and session navigation after the active compose/status controls so the primary chat action is first.
10. Run targeted regressions for existing ops shell slices and validate crate gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_controls.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks & Mitigations
- Risk: chat transcript hydration leaks system prompt rows and clutters transcript contracts.
  - Mitigation: filter system rows and blank message payloads from rendered chat rows.
- Risk: send endpoint drops operator shell controls after redirect.
  - Mitigation: deterministic redirect builder carries validated `theme`, `sidebar`, and sanitized `session` query tokens.
- Risk: empty button clicks bypass the Enter-key guard and appear to submit successfully.
  - Mitigation: install a submit-event guard in the form and preserve a backend empty-message status marker in the redirect.
- Risk: session history growth pushes the current send controls below the first screen.
  - Mitigation: assert and render the send form/status before the historical session selector.
- Risk: new-session creation is visually treated as the primary chat action.
  - Mitigation: assert and render active message compose/status before new-session creation controls.
- Risk: session controls still flood the first view even after ordering fixes.
  - Mitigation: place new-session and history controls inside a secondary `<details>` manager with a deterministic summary count.
- Risk: session metadata and navigation links still make the first chat control secondary.
  - Mitigation: assert and render the send form/status before the session summary and session action links.
- Risk: control query expansion (`session`/`session_key`) regresses existing route behavior.
  - Mitigation: add unit coverage for control parsing + keep existing default behavior unchanged.

## Interfaces / Contracts
- New gateway route: `POST /ops/chat/send` (form payload: `session_key`, `message`, `theme`, `sidebar`).
- Existing route update: `GET /ops/chat` reads `session`/`session_key` query token and maps active session transcript rows.
- UI shell contracts:
  - `id="tau-ops-chat-send-form"` with deterministic `action`, `method`, and session/theme/sidebar hidden inputs.
  - `data-empty-message-submit-guard="true"` plus submit-event JavaScript that prevents whitespace-only form submissions.
  - `id="tau-ops-chat-send-status"` with `data-chat-send-status`, including `empty-message` after backend rejection.
  - `tau-ops-chat-send-form` and `tau-ops-chat-send-status` appear before `tau-ops-chat-session-selector`.
  - `tau-ops-chat-send-form` and `tau-ops-chat-send-status` appear before `tau-ops-chat-new-session-form`.
  - `tau-ops-chat-send-form` and `tau-ops-chat-send-status` appear before `tau-ops-chat-session-summary` and `tau-ops-chat-session-actions`.
  - `id="tau-ops-chat-session-manager"` wraps new-session and selector controls with `data-collapsed-by-default="true"` and `data-session-option-count`.
  - `id="tau-ops-chat-transcript"` with deterministic `data-message-count` and row markers.
  - `id="tau-ops-chat-session-summary"` with `data-entry-count`, `data-transcript-message-count`, and `data-hidden-entry-count` so hidden system entries are not mistaken for missing transcript rows.

## ADR
No ADR required: no new dependency, protocol schema, or architecture boundary change.
