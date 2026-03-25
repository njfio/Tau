//! Custom-command contract and runtime support for Tau.
//!
//! Hosts fixture contracts and execution runtime logic for user-defined custom
//! command workflows in the operator control plane.
//!
//! # Deprecation Notice
//!
//! **This crate is deprecated as of 0.2.0.** The unified skills surface
//! (`tau-skills`) with tool, command, and hook support is now the primary
//! mechanism for user-defined commands. Migrate to skill manifests with
//! `commands` fields. See `skill_runtime.rs` for the replacement API.

pub mod custom_command_contract;
pub mod custom_command_policy;
pub mod custom_command_runtime;
