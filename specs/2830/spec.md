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
Then the user message and assistant reply are appended to the session store and the response redirects to `/ops/chat` preserving theme/sidebar/session controls plus the latest rendered message-row anchor.

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
Then the verbose session summary is grouped inside a collapsed-by-default session-details manager so it remains available without competing with the active composer.

### AC-12 Verbose latest-turn proof is grouped behind a compact latest-turn manager
Given `/ops/chat` renders latest user/assistant proof content,
When the operator lands on the active chat page,
Then the verbose latest-turn proof remains available inside a collapsed-by-default latest-turn manager so the transcript and active compose flow are not flooded by a long assistant turn.

### AC-13 Idle send status does not consume operator attention
Given `/ops/chat` renders without a send result,
When the operator lands on the active chat page,
Then the idle send-status marker remains present for contracts but is hidden from the visible operator surface, while real send results such as `empty-message` render visibly.

### AC-14 Current session navigation stays visible
Given `/ops/chat` renders active session metadata and a long transcript,
When the operator lands on the active chat page,
Then the open-session-detail and jump-to-latest actions are visible before collapsed secondary metadata so the current transcript and detail view are reachable without expanding a panel.

### AC-15 Non-empty send shows pending state while provider work runs
Given an operator submits a non-empty `/ops/chat` message,
When the browser waits for the provider-backed response and redirect,
Then the form exposes a deterministic submitting state, disables the send button, keeps the message payload submitted, and announces that Tau is sending the message.

### AC-16 Collapsed session metadata summarizes hidden transcript rows honestly
Given `/ops/chat` renders an active session with stored entries that do not all appear in the transcript,
When the collapsed session-details summary renders,
Then the summary identifies shown transcript rows, hidden rows, and total stored entries without requiring the operator to expand the metadata panel.

### AC-17 Collapsed latest-turn summary is operator-readable
Given `/ops/chat` renders latest user/assistant proof content,
When the collapsed latest-turn summary renders,
Then the visible summary describes the latest turn state in operator terms while preserving deterministic indexes and latest-message role markers as data attributes.

### AC-18 Collapsed latest-turn proof has truthful accessibility state
Given `/ops/chat` renders the latest-turn proof article inside a collapsed details manager,
When the page loads or the operator toggles the latest-turn manager,
Then the proof article is `aria-hidden` while collapsed and becomes exposed only when the manager is open and latest-turn content exists.

### AC-19 Collapsed session panels have truthful accessibility state
Given `/ops/chat` renders session metadata and session management controls inside collapsed details managers,
When the page loads or the operator toggles those managers,
Then the hidden session-summary, new-session form, new-session status, and session selector are `aria-hidden` while collapsed and become exposed only when their manager is open.

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
- Compact secondary session-details grouping around verbose session metadata.
- Visible current-session navigation actions for open-session-detail and jump-to-latest.
- Compact latest-turn proof grouping around the verbose user/assistant latest-turn preview.
- Hidden idle send-status marker with visible non-idle send-result states.
- Visible submitting-state markers for non-empty chat sends while provider work is pending.
- Honest collapsed session-details summary counts for shown transcript rows, hidden rows, and total entries.
- Operator-readable collapsed latest-turn state with deterministic latest row indexes retained as attributes.
- Synchronized latest-turn proof accessibility state for collapsed and expanded details states.
- Synchronized session-details and session-manager accessibility state for collapsed and expanded details states.
- Targeted regression validation for existing ops shell slices.

### Out of Scope
- Streaming chat responses in `/ops/chat`.
- Auth session cookie model changes.
- New message editing/deletion UI controls.

## Conformance Cases
- C-01 (functional): `/ops/chat` SSR includes send-form and transcript marker contracts.
- C-02 (integration): `/ops/chat` transcript markers reflect persisted active-session messages.
- C-03 (integration): `POST /ops/chat/send` appends user and assistant messages, redirects to `/ops/chat` with the latest message-row anchor, and the messages appear in transcript markers.
- C-04 (regression): existing ops shell suites (auth/nav/theme/control/timeline/alerts/connectors) remain green.
- C-05 (functional): `/ops/chat` summary markers distinguish total stored entries from rendered transcript rows and hidden system entries.
- C-06 (functional/integration): empty or whitespace-only chat sends are blocked by the form contract or redirected with `chat_status=empty-message` without creating or mutating a session store.
- C-07 (functional): `/ops/chat` places the send form and send-status marker before the historical session selector.
- C-08 (functional): `/ops/chat` places the send form and send-status marker before the new-session form and status controls.
- C-09 (functional): `/ops/chat` groups new-session and session selector controls inside a collapsed secondary session manager with a summary count.
- C-10 (functional): `/ops/chat` places the send form and send-status marker before the session summary and session navigation links.
- C-11 (functional): `/ops/chat` groups the verbose session summary inside a collapsed secondary session-details manager with an active-session summary.
- C-12 (functional): `/ops/chat` groups verbose latest-turn proof content inside a collapsed secondary latest-turn manager while preserving latest-turn markers and indexes.
- C-13 (functional): `/ops/chat` hides idle send-status copy while rendering non-idle send-result states visibly.
- C-14 (functional): `/ops/chat` renders open-session-detail and jump-to-latest actions before collapsed secondary metadata.
- C-19 (functional): `/ops/chat` marks hidden session-summary/session-manager children as `aria-hidden` while collapsed and syncs them when toggled open.
- C-15 (functional): `/ops/chat` compose script marks non-empty sends as submitting with visible status copy and a disabled pending send button without disabling the message textarea.
- C-16 (functional): `/ops/chat` collapsed session-details summary renders shown/hidden/total entry counts and exposes those counts on deterministic manager attributes.
- C-17 (functional): `/ops/chat` collapsed latest-turn summary renders operator-readable state such as assistant reply shown or waiting for assistant reply while exposing latest indexes and role on deterministic attributes.
- C-18 (functional/browser): `/ops/chat` latest-turn proof article starts `aria-hidden` while its details manager is collapsed, and the details-toggle sync script exposes it only when opened.

## Success Metrics / Observable Signals
- `cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passes.
- `cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passes.
- `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1` passes.
- `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1` passes.

## Approval Gate
P1 multi-module slice proceeds with spec marked `Reviewed` per AGENTS.md self-acceptance rule. Human review required in PR.
