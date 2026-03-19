//! Interactive terminal UI for Tau, inspired by OpenCode/OpenClaw.
//!
//! Provides a full-screen ratatui-based interface with:
//! - Scrollable chat panel with message history
//! - Tool execution panel showing active/recent tool calls
//! - Status bar with model, tokens, cost, and circuit breaker state
//! - Multi-line input editor with vim-like keybindings
//! - Keyboard-driven navigation and command palette

mod app;
mod chat;
mod input;
mod status;
mod tools;
mod ui;
#[cfg(test)]
mod ui_tool_visibility_tests;

pub use app::{run_interactive, App, AppConfig};
