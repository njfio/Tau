#![cfg_attr(test, allow(unused_imports))]
//! Tau coding-agent binary entrypoint and module wiring.
//!
//! This crate root binds startup, command, runtime, and transport modules into a
//! single executable. Startup preflight and dispatch boundaries are delegated to
//! dedicated modules to keep runtime behavior explicit and testable.

mod auth_commands;
mod bootstrap_helpers;
mod canvas;
mod channel_adapters;
mod channel_lifecycle;
mod channel_send;
mod channel_store;
mod channel_store_admin;
mod commands;
mod deployment_wasm;
mod events;
mod live_rl_runtime;
mod macro_profile_commands;
mod mcp_client;
mod mcp_server;
mod model_catalog;
mod multi_agent_router;
mod observability_loggers;
mod orchestrator_bridge;
mod project_index;
mod qa_loop_commands;
mod release_channel_commands;
mod rpc_capabilities;
mod rpc_protocol;
mod runtime_loop;
mod runtime_output;
mod runtime_profile_policy_bridge;
mod runtime_prompt_template_bridge;
mod runtime_types;
mod self_modification_runtime;
mod startup_dispatch;
mod startup_local_runtime;
mod startup_model_catalog;
mod startup_preflight;
mod startup_transport_modes;
#[cfg(test)]
mod tool_policy_config;
mod tools;
mod training_proxy_runtime;
mod training_runtime;
#[cfg(test)]
mod transport_conformance;
#[cfg(test)]
mod transport_health;

use anyhow::Result;
use clap::Parser;
pub(crate) use tau_extensions as extension_manifest;

pub(crate) use crate::bootstrap_helpers::init_tracing;
use crate::startup_dispatch::run_cli;
pub(crate) use tau_cli::normalize_legacy_training_aliases;
pub(crate) use tau_cli::Cli;

// Test-only crate aliases (must live at crate root so that `crate::package_manifest`,
// `crate::skills`, and `crate::skills_commands` resolve for test_harness re-exports).
#[cfg(test)]
pub(crate) use tau_skills as package_manifest;
#[cfg(test)]
pub(crate) use tau_skills as skills;
#[cfg(test)]
pub(crate) use tau_skills as skills_commands;

// Centralised test re-exports – keeps the cfg(test) surface in this file minimal.
#[cfg(test)]
mod test_harness;
#[cfg(test)]
pub(crate) use test_harness::*;

pub(crate) fn normalize_daemon_subcommand_args(args: Vec<String>) -> Vec<String> {
    if args.len() < 3 || args[1] != "daemon" {
        return args;
    }

    let action_flag = match args[2].as_str() {
        "install" => "--daemon-install",
        "uninstall" => "--daemon-uninstall",
        "start" => "--daemon-start",
        "stop" => "--daemon-stop",
        "status" => "--daemon-status",
        _ => return args,
    };

    let mut normalized = Vec::with_capacity(args.len());
    normalized.push(args[0].clone());
    normalized.push(action_flag.to_string());
    for argument in args.into_iter().skip(3) {
        match argument.as_str() {
            "--profile" => normalized.push("--daemon-profile".to_string()),
            "--state-dir" => normalized.push("--daemon-state-dir".to_string()),
            "--reason" => normalized.push("--daemon-stop-reason".to_string()),
            "--json" => normalized.push("--daemon-status-json".to_string()),
            other => normalized.push(other.to_string()),
        }
    }
    normalized
}

pub(crate) fn normalize_startup_cli_args(args: Vec<String>) -> (Vec<String>, Vec<String>) {
    let daemon_normalized = normalize_daemon_subcommand_args(args);
    normalize_legacy_training_aliases(daemon_normalized)
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let (normalized_args, compatibility_warnings) =
        normalize_startup_cli_args(std::env::args().collect());
    for warning in compatibility_warnings {
        eprintln!("{warning}");
    }
    let cli = Cli::parse_from(normalized_args);
    run_cli(cli).await
}

#[cfg(test)]
mod tests;
