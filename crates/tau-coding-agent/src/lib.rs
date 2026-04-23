//! Library surface for the `tau-coding-agent` crate.
//!
//! The crate is primarily a binary (`src/main.rs`), but a few modules are
//! shared between that main binary, auxiliary binaries in `src/bin/`, and
//! external integration tests. Those modules are re-exported here so every
//! compilation unit resolves them via the same canonical path
//! (`tau_coding_agent::…`), avoiding `#[path]` hacks and duplicate
//! compilation.
//!
//! Only genuinely cross-binary surface belongs here. Binary-internal plumbing
//! stays in `src/main.rs` under `mod …;` to keep the library footprint small.

pub mod self_modification_pipeline;
pub mod self_modification_runtime;
pub mod self_modification_synthesis_tool;
pub mod self_modification_tool;

/// Test-only process-wide lock used by every test that mutates
/// `TAU_AUTONOMOUS_SELF_MOD`. Module-local mutexes do not serialize across
/// modules within the same test binary, and a torn env-var state produces
/// flaky failures like "execute_refuses_when_env_gate_is_off" observing "1".
/// Putting the lock on the shared lib surface guarantees a single instance.
#[cfg(test)]
pub(crate) static AUTONOMOUS_SELF_MOD_ENV_LOCK: std::sync::Mutex<()> =
    std::sync::Mutex::new(());
