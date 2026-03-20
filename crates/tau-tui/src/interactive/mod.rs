//! Interactive full-screen TUI for the Tau coding agent.
//!
//! This module provides:
//! - Scrollable chat panel with message history
//! - Multi-line input editor with vim-like modes
//! - Status bar showing model, tokens, costs, and agent state
//! - Tool execution side panel
//! - Mouse-aware panel focus and scrolling

mod app;
mod app_commands;
mod app_keys;
mod app_mouse;
#[cfg(test)]
mod app_mouse_tests;
mod app_runtime;
mod chat;
mod input;
mod status;
mod tools;
mod ui;
mod ui_body;
mod ui_chat;
mod ui_chat_tool_lines;
mod ui_input;
mod ui_layout;
mod ui_overlays;
mod ui_status;
#[cfg(test)]
mod ui_build_status_tests;
#[cfg(test)]
mod ui_tool_visibility_tests;
mod ui_tools;

pub use app::{App, AppConfig};
pub use app_runtime::run_interactive;
