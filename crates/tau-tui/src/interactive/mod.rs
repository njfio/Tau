//! Interactive full-screen TUI for the Tau coding agent.
//!
//! This module provides:
//! - Scrollable chat panel with message history
//! - Multi-line input editor with vim-like modes
//! - Status bar showing model, tokens, costs, and agent state
//! - Tool execution side panel
//! - Mouse-aware panel focus and scrolling

mod app;
#[cfg(test)]
mod app_command_palette_tests;
mod app_commands;
mod app_copy_target;
#[cfg(test)]
mod app_gateway_tests;
mod app_keys;
mod app_mouse;
#[cfg(test)]
mod app_mouse_tests;
mod app_runtime;
mod build_status;
mod chat;
mod gateway_client;
mod input;
pub mod operator_state;
#[cfg(test)]
mod operator_state_tests;
mod status;
mod tools;
mod ui;
mod ui_body;
#[cfg(test)]
mod ui_build_status_tests;
mod ui_chat;
mod ui_chat_mutating_progress;
mod ui_chat_tool_lines;
mod ui_input;
mod ui_layout;
#[cfg(test)]
mod ui_mutating_tool_progress_tests;
#[cfg(test)]
mod ui_mutating_transcript_target_tests;
mod ui_overlays;
mod ui_status;
#[cfg(test)]
mod ui_tool_visibility_tests;
mod ui_tools;
#[cfg(test)]
mod ui_transcript_first_layout_tests;

pub use app::{App, AppConfig};
pub use app_runtime::run_interactive;
pub use gateway_client::GatewayRuntimeConfig;
