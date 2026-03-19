//! Interactive terminal UI for Tau, inspired by OpenCode/OpenClaw.
//!
//! Provides a full-screen ratatui-based interface with:
//! - Scrollable chat panel with message history
//! - Main-shell tool activity summary plus side-panel recent tool calls
//! - Status bar with model, tokens, cost, and circuit breaker state
//! - Multi-line input editor with vim-like keybindings
//! - Keyboard-driven navigation and command palette

mod app;
mod app_mouse;
mod chat;
mod input;
mod status;
mod tools;
mod ui;
mod ui_body;
mod ui_chat;
mod ui_chat_tool_lines;
mod ui_input;
mod ui_overlays;
mod ui_status;
#[cfg(test)]
mod app_mouse_tests;
#[cfg(test)]
mod ui_tool_visibility_tests;
mod ui_tools;

pub use app::{run_interactive, App, AppConfig};
