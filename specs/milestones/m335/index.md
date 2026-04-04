# M335 - World-class TUI REPL

Status: Active

## Context
Tau's interactive TUI has the core pieces of a usable operator shell: chat,
tool streaming, mission controls, status surfaces, and slash commands. It does
not yet behave like a world-class REPL. The current shell still makes common
interactive tasks harder than they should be: editing and reusing prompts,
inspecting streaming work, copying useful output, navigating session history,
and controlling the runtime from the keyboard without friction.

This milestone raises `tau-tui` from "working terminal UI" to "high-leverage
operator REPL" by focusing on ergonomic, transparent, keyboard-first
interaction. The goal is not a cosmetic reskin. The goal is a shell that feels
fast, stateful, inspectable, and powerful during real long-running Tau work.

## Issue Hierarchy
- Story: [#3677](https://github.com/njfio/Tau/issues/3677) Raise `tau-tui` to a
  world-class operator REPL
- Task: [#3678](https://github.com/njfio/Tau/issues/3678) Add prompt history
  and editor ergonomics to `tau-tui` REPL
- Task: [#3679](https://github.com/njfio/Tau/issues/3679) Add transcript
  search, copy, and stronger scrollback to `tau-tui` REPL
- Task: [#3680](https://github.com/njfio/Tau/issues/3680) Add local session
  persistence to `tau-tui` REPL
- Task: [#3681](https://github.com/njfio/Tau/issues/3681) Add local transcript
  persistence to `tau-tui` REPL
- Task: [#3682](https://github.com/njfio/Tau/issues/3682) Add transcript
  export commands to `tau-tui` REPL
- Task: [#3683](https://github.com/njfio/Tau/issues/3683) Add command palette
  autocomplete and history to `tau-tui` REPL
- Task: [#3684](https://github.com/njfio/Tau/issues/3684) Add command palette
  aliases and richer feedback to `tau-tui` REPL
- Task: [#3685](https://github.com/njfio/Tau/issues/3685) Add command palette
  paging and full browsing to `tau-tui` REPL
- Task: [#3686](https://github.com/njfio/Tau/issues/3686) Add selected-command
  preview details to `tau-tui` command palette
- Task: [#3687](https://github.com/njfio/Tau/issues/3687) Group `tau-tui`
  command palette commands into operator sections
- Task: [#3688](https://github.com/njfio/Tau/issues/3688) Add inline argument
  scaffolding to `tau-tui` command palette
- Task: [#3689](https://github.com/njfio/Tau/issues/3689) Add fuzzy command
  filtering to `tau-tui` command palette
- Task: [#3690](https://github.com/njfio/Tau/issues/3690) Add placeholder
  guardrails to `tau-tui` command palette scaffolds
- Task: [#3692](https://github.com/njfio/Tau/issues/3692) Add direct scaffold
  placeholder editing to `tau-tui` command palette
- Task: [#3693](https://github.com/njfio/Tau/issues/3693) Add reverse
  placeholder cycling and active-placeholder feedback to `tau-tui` command
  palette
- Task: [#3694](https://github.com/njfio/Tau/issues/3694) Auto-focus the first
  scaffold placeholder after parameterized command autocomplete in `tau-tui`
- Task: [#3695](https://github.com/njfio/Tau/issues/3695) Add shell-style
  `Ctrl+A`/`Ctrl+E`/`Ctrl+U`/`Ctrl+K` editing to `tau-tui` command palette
- Task: [#3696](https://github.com/njfio/Tau/issues/3696) Add word-wise
  `Alt+B`/`Alt+F` cursor movement to `tau-tui` command palette
- Task: [#3697](https://github.com/njfio/Tau/issues/3697) Add previous-word
  deletion via `Ctrl+W` to `tau-tui` command palette
- Task: [#3698](https://github.com/njfio/Tau/issues/3698) Add forward-word
  deletion via `Alt+D` to `tau-tui` command palette
- Task: [#3699](https://github.com/njfio/Tau/issues/3699) Add `Left`/`Right`
  cursor movement and `Delete` to `tau-tui` command palette
- Task: [#3700](https://github.com/njfio/Tau/issues/3700) Make active scaffold
  placeholders atomic for `Left`/`Right` in `tau-tui` command palette
- Task: [#3701](https://github.com/njfio/Tau/issues/3701) Make active scaffold
  placeholders atomic for `Alt+B`/`Alt+F` in `tau-tui` command palette
- Task: [#3702](https://github.com/njfio/Tau/issues/3702) Collapse separator
  whitespace when `Alt+D`/`Ctrl+W` delete active placeholders in `tau-tui`
  command palette
- Task: [#3703](https://github.com/njfio/Tau/issues/3703) Collapse separator
  whitespace when `Backspace`/`Delete` remove active placeholders in `tau-tui`
  command palette
- Task: [#3704](https://github.com/njfio/Tau/issues/3704) Preserve active
  placeholder focus across `Ctrl+U`/`Ctrl+K` in `tau-tui` command palette
- Task: [#3705](https://github.com/njfio/Tau/issues/3705) Make `Esc` clear
  active placeholder focus before closing `tau-tui` command palette
- Task: [#3706](https://github.com/njfio/Tau/issues/3706) Make active scaffold
  placeholders atomic for `Ctrl+A`/`Ctrl+E` in `tau-tui` command palette
- Task: [#3707](https://github.com/njfio/Tau/issues/3707) Match `tau-tui`
  command palette queries against usage and scaffold text
- Task: [#3708](https://github.com/njfio/Tau/issues/3708) Normalize separator
  punctuation in `tau-tui` command palette queries
- Task: [#3709](https://github.com/njfio/Tau/issues/3709) Match multi-token
  `tau-tui` command palette queries across command metadata
- Task: [#3710](https://github.com/njfio/Tau/issues/3710) Match tau-tui
  command palette queries against section labels
- Task: [#3711](https://github.com/njfio/Tau/issues/3711) Surface section and
  scaffold details in tau-tui command palette preview
- Task: [#3712](https://github.com/njfio/Tau/issues/3712) Surface scaffold
  placeholder summary in tau-tui command palette preview
- Task: [#3713](https://github.com/njfio/Tau/issues/3713) Surface Enter
  execution target in tau-tui command palette preview
- Task: [#3714](https://github.com/njfio/Tau/issues/3714) Surface alias-match
  provenance in tau-tui command palette preview
- Task: [#3715](https://github.com/njfio/Tau/issues/3715) Surface scaffold
  placeholder-match provenance in tau-tui command palette preview
- Task: [#3716](https://github.com/njfio/Tau/issues/3716) Surface section-match
  provenance in tau-tui command palette preview
- Task: [#3717](https://github.com/njfio/Tau/issues/3717) Surface summary-match
  provenance in tau-tui command palette preview
- Task: [#3718](https://github.com/njfio/Tau/issues/3718) Surface literal
  scaffold-token provenance in tau-tui command palette preview
- Task: [#3719](https://github.com/njfio/Tau/issues/3719) Surface unresolved-
  placeholder Enter blocking in the tau-tui command-palette preview
- Task: [#3720](https://github.com/njfio/Tau/issues/3720) Reuse real Enter
  preview guidance in the empty-match tau-tui command-palette state
- Task: [#3721](https://github.com/njfio/Tau/issues/3721) Normalize
  slash-prefixed explicit command-palette submissions in tau-tui
- Task: [#3722](https://github.com/njfio/Tau/issues/3722) Canonicalize
  slash-prefixed typed Enter preview guidance in tau-tui command palette
- Task: [#3723](https://github.com/njfio/Tau/issues/3723) Surface
  slash-prefixed typed command matches in tau-tui command palette
- Task: [#3724](https://github.com/njfio/Tau/issues/3724) Canonicalize
  slash-prefixed alias typed Enter guidance in tau-tui command palette

## Scope
- Improve the interactive TUI as an operator-grade REPL rather than a basic
  transcript + input box
- Focus on keyboard-first workflows, history/navigation, copy/export,
  streaming visibility, command execution ergonomics, and session control
- Preserve gateway-backed Ralph-loop behavior while making it easier to drive,
  inspect, and recover from within the shell

## Exit Criteria
- Operators can efficiently edit, resend, search, inspect, and copy prior work
  without leaving the TUI
- The shell exposes clear session/runtime state and current turn progress while
  staying responsive during long-running work
- The first REPL improvement slice is captured in a governed story with
  explicit requirements, bounded scope, and verification gates

## Delivery Notes
- Favor durable operator ergonomics over decorative UI changes
- Prefer staged REPL upgrades over a monolithic shell rewrite
- Treat input editing, history, observability, and recovery controls as core
  REPL capabilities
