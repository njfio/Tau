//! Memory contract fixtures and runtime execution for Tau.
//!
//! Contains memory-specific contract replay and runtime plumbing used by
//! channel memory workflows and validation suites.

pub mod memory_contract;
pub mod memory_runtime;

pub use tau_memory_backend::{
    normalize_workspace_id, JsonlLiveMemoryBackend, LiveMemoryBackend, LiveMemoryMessage,
    LiveMemoryRole,
};
