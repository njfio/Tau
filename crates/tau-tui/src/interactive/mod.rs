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
mod chat;
mod gateway;
mod gateway_runtime;
mod input;
mod status;
mod tools;
mod ui;
#[cfg(test)]
mod gateway_runtime_tests;
#[cfg(test)]
mod gateway_tests;

pub use app::{App, AppConfig, run_interactive};
pub use gateway::GatewayInteractiveConfig;
