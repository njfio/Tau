---
title: Tau TUI command palette placeholder jump editor
category: patterns
date: '2026-04-27'
tags:
  - tau-tui
  - command-palette
  - placeholder-jump
  - tui
  - rust-tests
related:
  - specs/3691/spec.md
  - crates/tau-tui/src/interactive/app_commands.rs
  - crates/tau-tui/src/interactive/app_command_palette_tests.rs
---

# Tau TUI command palette placeholder jump editor
## Problem
The tau-tui command palette was append-only, so argument-heavy command templates with placeholders were awkward to edit and could not support predictable placeholder jumping.
## Root cause
The command palette stored input as a plain String without a cursor position, while focused key handling only appended characters or popped the last character before submitting the command.
## Solution
Add a command-palette cursor column to App, route focused palette keys through local single-line editing helpers, use Tab to jump to the next placeholder span written as <name> or {name}, and render the cursor at the tracked palette column. Focused tests named placeholder_jump cover interior insertion/backspace, first placeholder movement, multi-placeholder wraparound, and existing command submission behavior.
## Prevention

Keep command-palette editing behavior covered by narrow placeholder_jump tests whenever adding autocomplete or templates, and preserve command execution through the existing execute_command path so editing ergonomics do not alter command semantics.
