//! Interactive terminal UI for Tau, inspired by OpenCode/OpenClaw.
//!
//! Provides a full-screen ratatui-based interface with:
//! - Scrollable chat panel with message history
//! - Tool execution panel showing active/recent tool calls
//! - Status bar with model, tokens, cost, and circuit breaker state
//! - Multi-line input editor with vim-like keybindings
//! - Keyboard-driven navigation and command palette

mod app;
mod app_gateway;
mod app_input;
mod app_submit;
mod chat;
mod gateway;
mod gateway_runtime;
#[cfg(test)]
mod gateway_runtime_tests;
#[cfg(test)]
mod gateway_tests;
mod input;
mod status;
mod terminal;
mod tools;
mod ui;

pub use app::{App, AppConfig};
pub use gateway::{parse_sse_frames, GatewayInteractiveConfig, GatewayUiEvent, OperatorStateEvent};
pub use gateway_runtime::GatewayRuntime;
pub use terminal::run_interactive;
