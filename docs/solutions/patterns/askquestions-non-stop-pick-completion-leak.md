---
title: "askQuestions non-Stop pick triggered GYRE-TURN-COMPLETE — premium-request leak"
category: patterns
date: 2026-04-20
tags: ["gyre", "copilot", "askquestions", "billing", "premium-request", "completion-promise", "phase-4"]
related: ["patterns/askquestions-replaces-custom-picker.md"]
---
# askQuestions non-Stop pick triggered GYRE-TURN-COMPLETE — premium-request leak

## Problem

After a Gyre session restart + "continue" prompt, the agent ran Phase 4,
presented `askQuestions` with 3 next-stage options plus a "Stop here —
emit GYRE-TURN-COMPLETE" option, and the user picked the **Harden journal
integration** option. Instead of calling `mcp_gyre_backlog_replace` with
deliverables for that stage, the agent wrote:

```
Q: Which stage next?
A: Harden journal integration - Verify appendJournal preserves earlier entries…
GYRE-TURN-COMPLETE: Cycled through property tests, versioned the release to
0.2.3, simplified the unit test runner, and closed with 107 unit+property
tests plus 13 integration tests passing…
```

The turn ended. The user's pick — which was a clear continuation instruction
— was converted into a summary and a stop marker. Next action from the user
would have cost a new premium request. This is the primary billing-leak
failure mode the whole agent prompt is built to prevent.

## Root cause

The agent prompt had two related weaknesses:

1. **Template 3's trigger condition was ambiguous about tool-results vs.
   chat messages.** The phrasing "If the user's most recent message contains
   an explicit stop signal" could be read to include an `askQuestions`
   tool-result as a "message," especially when the picked option label
   sounded conservative (Harden / Deepen / Simplify). The agent rationalized
   a "Harden" pick as a natural wrap-up because it read Q/A + completed
   exchange as a closing beat.

2. **The post-`askQuestions` behavior was documented inside Template 2's
   body paragraph, not as an inviolable top-level rule.** The prompt did
   say "for any other pick, IMMEDIATELY call `mcp_gyre_backlog_replace`"
   but it was buried mid-paragraph and had no self-check anchor.

Root-cause framing: the prompt treated the POST-ASKQUESTIONS path as a
soft rule inside Template 2 rather than a hard self-check that runs
*before* any response is emitted. The self-check list started with "Does
it END with a tool call?" but had no "Did you just receive an askQuestions
result? Was it Stop?" step — which is the ONE check that would have
caught this.

## Solution

Three prompt edits in `.github/agents/gyre.agent.md`:

1. **New top-level "POST-ASKQUESTIONS RULE" section** placed immediately
   after Template 2, labeled as "the single most common failure mode."
   Spells out: the pick is structured input that instructs what to do
   next, it is NOT a chat message for purposes of the completion-promise
   trigger, and the ONLY Stop trigger is the literal substring `Stop
   here — emit GYRE-TURN-COMPLETE` in the pick text.

2. **Tightened Template 3 trigger.** Now explicitly requires EITHER a
   real chat-message stop signal OR a literal-Stop-substring pick in the
   most recent `askQuestions` tool-result. Adds an anti-pattern example
   quoting the exact failure mode ("feels like a natural wrap-up" →
   forbidden).

3. **Self-check step 1 rewritten** to run the POST-ASKQUESTIONS check
   *first*, before any other ending check. If the last tool-result was
   from `askQuestions` and the pick wasn't the literal Stop option, the
   next tool call MUST be `mcp_gyre_backlog_replace`. If it isn't,
   rewrite.

4. **Phase 4's forbidden endings** now explicitly list "emitting
   `GYRE-TURN-COMPLETE:` after the user picks a non-Stop option" as
   forbidden, with examples ("Harden", "Deepen", "Refactor", "Simplify"
   are all continuations, not stops).

5. **Completion-promise section rewritten** to enumerate non-stop
   signals explicitly, including the `askQuestions` case.

## Prevention

- **Prompt rules that gate behavior on tool-result content must be
  structured as a self-check step, not a paragraph claim.** The
  self-check list is what the agent scans before emitting; rules not
  anchored there can be overridden by rationalization.
- **Never overload "user's most recent message" to cover tool-results.**
  A tool-result is structured input from a tool, even if the user's
  selection drove it. The completion-promise trigger must reference
  "chat message" explicitly to avoid conflation.
- **When adding a stop option to `askQuestions` (or any similar blocking
  picker), the stop detection must be a literal-substring match, not a
  semantic judgment.** "Harden" sounds conservative; the agent will
  over-generalize unless the Stop signal is a unique literal string.
- **Instrumentation / detection idea for next iteration:** the MCP server
  or a hook could record the last `askQuestions` response, and a
  post-response hook could fail the turn if it detects
  `GYRE-TURN-COMPLETE:` in the agent's output without a preceding
  literal-Stop pick. Belt-and-suspenders for the prompt rule.

## Signs the fix is working

After applying: send a task, drive to Phase 4, let `askQuestions` fire,
pick any non-Stop option. Expected behavior: the agent's next tool call
is `mcp_gyre_backlog_replace` with deliverables matching the picked
option. Expected non-behavior: the agent does NOT emit
`GYRE-TURN-COMPLETE:` and does NOT write a summary paragraph between
the tool-result and the `backlog_replace` call.

If the Gyre session proceeds through multiple `askQuestions` cycles
inside a single Copilot premium request (check the Copilot usage
dashboard at github.com/settings/copilot), the billing invariant is
preserved.
