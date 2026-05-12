# Spec: Issue #2830 - Chat message send and transcript visibility contracts

Status: Implemented

## Problem Statement
Tau Ops chat route scaffolding exists, but there is no live message-send pathway that appends operator input and re-renders transcript state deterministically in SSR output. PRD phase 1M requires end-to-end contracts for `Chat -> Message sends and appears in chat`.

## Acceptance Criteria

### AC-1 `/ops/chat` exposes deterministic send-form and transcript SSR markers
Given the Tau Ops chat shell render,
When operators inspect `/ops/chat` HTML,
Then deterministic marker contracts exist for chat form action/method/session and transcript rows.

### AC-2 Chat transcript markers map from active session state
Given an active session key for chat,
When `/ops/chat` renders,
Then transcript markers map role/content rows from persisted session lineage for that session.

### AC-3 `POST /ops/chat/send` appends user messages and redirects back to chat
Given a valid message submission to `/ops/chat/send`,
When the request is processed,
Then the message is appended to the session store and the response redirects to `/ops/chat` preserving theme/sidebar/session controls.

### AC-4 Existing Tau Ops shell contracts remain stable
Given existing phase 1A..1L ops shell suites,
When chat send/transcript integration lands,
Then prior suites remain green.

### AC-5 Chat summary distinguishes stored entries from rendered transcript rows
Given an active chat session that contains hidden system entries,
When `/ops/chat` renders the operator summary and transcript,
Then the summary exposes total entries, rendered transcript rows, and hidden rows separately, and the transcript message count excludes empty-state placeholders.

### AC-6 Empty chat sends are rejected visibly without mutating session state
Given an operator submits an empty or whitespace-only message through `/ops/chat/send`,
When the request is processed by the UI submit guard or backend handler,
Then the send is blocked or redirected back to `/ops/chat` with `chat_status=empty-message`, no session entry is appended, and the chat page renders a visible send-status marker explaining the rejection.

### AC-7 Active chat composition stays above historical session navigation
Given `/ops/chat` renders with multiple discovered sessions,
When the operator arrives at the active chat page,
Then the send form and send-status controls appear before the historical session selector so the primary compose action remains reachable without scanning past old sessions.

### AC-8 Active chat composition stays above secondary session creation
Given `/ops/chat` renders for an active session,
When the operator arrives at the active chat page,
Then the message composer appears before new-session creation controls so the primary path is replying in the current session.

### AC-9 Secondary session controls are grouped behind a compact session manager
Given `/ops/chat` renders session creation and session history controls,
When the operator lands on the active chat page,
Then the new-session form and session selector are grouped inside a collapsed-by-default session manager summary so they do not flood the first view.

### AC-10 Active chat composition appears before session metadata navigation
Given `/ops/chat` renders active session metadata and session navigation links,
When the operator lands on the active chat page,
Then the send form and send status appear before the session summary, open-session-detail link, and jump-to-latest link so the primary task remains composing in the current session.

### AC-11 Secondary session metadata is grouped behind a compact session-details manager
Given `/ops/chat` renders active session metadata and session navigation links,
When the operator lands on the active chat page,
Then the session summary, open-session-detail link, and jump-to-latest link are grouped inside a collapsed-by-default session-details manager so they remain available without competing with the active composer.

## Scope

### In Scope
- `tau-dashboard-ui` chat panel form/transcript SSR contract coverage.
- `tau-gateway` ops chat transcript hydration from `SessionStore`.
- `tau-gateway` `POST /ops/chat/send` append + redirect behavior.
- Empty-message submit guard and visible backend rejection status.
- Composer-first ordering before historical session selection.
- Composer-first ordering before secondary new-session creation controls.
- Compact secondary session manager grouping around new-session and session-history controls.
- Composer-first ordering before session metadata and navigation actions.
- Compact secondary session-details grouping around session metadata and navigation actions.
- Targeted regression validation for existing ops shell slices.

### Out of Scope
- Streaming chat responses in `/ops/chat`.
- Auth session cookie model changes.
- New message editing/deletion UI controls.

## Conformance Cases
- C-01 (functional): `/ops/chat` SSR includes send-form and transcript marker contracts.
- C-02 (integration): `/ops/chat` transcript markers reflect persisted active-session messages.
- C-03 (integration): `POST /ops/chat/send` appends user message, redirects to `/ops/chat`, and message appears in transcript markers.
- C-04 (regression): existing ops shell suites (auth/nav/theme/control/timeline/alerts/connectors) remain green.
- C-05 (functional): `/ops/chat` summary markers distinguish total stored entries from rendered transcript rows and hidden system entries.
- C-06 (functional/integration): empty or whitespace-only chat sends are blocked by the form contract or redirected with `chat_status=empty-message` without creating or mutating a session store.
- C-07 (functional): `/ops/chat` places the send form and send-status marker before the historical session selector.
- C-08 (functional): `/ops/chat` places the send form and send-status marker before the new-session form and status controls.
- C-09 (functional): `/ops/chat` groups new-session and session selector controls inside a collapsed secondary session manager with a summary count.
- C-10 (functional): `/ops/chat` places the send form and send-status marker before the session summary and session navigation links.
- C-11 (functional): `/ops/chat` groups the session summary and session navigation links inside a collapsed secondary session-details manager with an active-session summary.

## Success Metrics / Observable Signals
- `cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passes.
- `cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1` passes.

## Approval Gate
P1 multi-module slice proceeds with spec marked `Reviewed` per AGENTS.md self-acceptance rule. Human review required in PR.
