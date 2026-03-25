//! Operator command runtime implementations for Tau.
//!
//! Contains command catalog and runtime handlers for canvas, daemon, macros,
//! project indexing, QA loops, channel store admin, and transport health ops.

mod canvas_commands;
mod channel_store_admin;
mod command_catalog;
mod daemon_runtime;
pub mod learn_commands;
mod macro_commands;
mod project_index;
mod qa_loop_commands;
pub mod script_gate;
pub mod self_modify_commands;
pub mod training_commands;
mod transport_health;
pub mod verification_gates;

pub use canvas_commands::*;
pub use channel_store_admin::*;
pub use command_catalog::*;
pub use daemon_runtime::*;
pub use learn_commands::*;
pub use macro_commands::*;
pub use project_index::*;
pub use qa_loop_commands::*;
pub use script_gate::*;
pub use self_modify_commands::*;
pub use training_commands::*;
pub use transport_health::*;

/// Generate a zsh completion script from the command catalog.
pub fn generate_completions_zsh() -> String {
    let mut output =
        String::from("#compdef tau\n\n_tau() {\n  local -a commands\n  commands=(\n");
    for name in command_catalog::COMMAND_NAMES {
        output.push_str(&format!("    '{}'\n", name.trim_start_matches('/')));
    }
    output.push_str("  )\n  _describe 'command' commands\n}\n\n_tau \"$@\"\n");
    output
}

/// Return the canonical test command for `/ops-dev test`.
pub fn ops_dev_test_command() -> String {
    "cargo test".to_string()
}

/// Return the canonical lint command for `/ops-dev lint`.
pub fn ops_dev_lint_command() -> String {
    "cargo fmt --check && cargo clippy --workspace".to_string()
}

/// Return the canonical clean command for `/ops-dev clean`.
pub fn ops_dev_clean_command() -> String {
    "cargo clean".to_string()
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn unit_generate_completions_zsh_valid_output() {
        let output = generate_completions_zsh();
        assert!(output.starts_with("#compdef tau"));
        assert!(output.contains("_tau"));
        assert!(output.contains("help"));
        assert!(output.contains("session"));
        assert!(output.contains("_describe 'command' commands"));
        // Should not contain leading slashes in completion entries
        assert!(!output.contains("    '/"));
    }

    #[test]
    fn unit_ops_dev_commands() {
        assert_eq!(ops_dev_test_command(), "cargo test");
        assert_eq!(
            ops_dev_lint_command(),
            "cargo fmt --check && cargo clippy --workspace"
        );
        assert_eq!(ops_dev_clean_command(), "cargo clean");
    }
}
