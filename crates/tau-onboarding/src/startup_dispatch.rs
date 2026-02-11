use std::path::{Path, PathBuf};

use anyhow::Result;
use tau_cli::Cli;
use tau_skills::default_skills_lock_path;

use crate::startup_policy::{resolve_startup_policy, StartupPolicyBundle};
use crate::startup_prompt_composition::compose_startup_system_prompt;

pub struct StartupRuntimeDispatchContext {
    pub effective_skills_dir: PathBuf,
    pub skills_lock_path: PathBuf,
    pub system_prompt: String,
    pub startup_policy: StartupPolicyBundle,
}

pub fn build_startup_runtime_dispatch_context(
    cli: &Cli,
    bootstrap_lock_path: &Path,
    activation_applied: bool,
) -> Result<StartupRuntimeDispatchContext> {
    let effective_skills_dir = resolve_runtime_skills_dir(cli, activation_applied);
    let skills_lock_path =
        resolve_runtime_skills_lock_path(cli, bootstrap_lock_path, &effective_skills_dir);
    let system_prompt = compose_startup_system_prompt(cli, &effective_skills_dir)?;
    let startup_policy = resolve_startup_policy(cli)?;
    Ok(StartupRuntimeDispatchContext {
        effective_skills_dir,
        skills_lock_path,
        system_prompt,
        startup_policy,
    })
}

pub fn resolve_runtime_skills_dir(cli: &Cli, activation_applied: bool) -> PathBuf {
    if !activation_applied {
        return cli.skills_dir.clone();
    }
    let activated_skills_dir = cli.package_activate_destination.join("skills");
    if activated_skills_dir.is_dir() {
        return activated_skills_dir;
    }
    cli.skills_dir.clone()
}

pub fn resolve_runtime_skills_lock_path(
    cli: &Cli,
    bootstrap_lock_path: &Path,
    effective_skills_dir: &Path,
) -> PathBuf {
    if effective_skills_dir == cli.skills_dir {
        bootstrap_lock_path.to_path_buf()
    } else {
        default_skills_lock_path(effective_skills_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_startup_runtime_dispatch_context, resolve_runtime_skills_dir,
        resolve_runtime_skills_lock_path,
    };
    use clap::Parser;
    use tau_cli::Cli;
    use tau_skills::default_skills_lock_path;
    use tempfile::tempdir;

    fn parse_cli_with_stack() -> Cli {
        std::thread::Builder::new()
            .name("tau-cli-parse".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| Cli::parse_from(["tau-rs"]))
            .expect("spawn cli parse thread")
            .join()
            .expect("join cli parse thread")
    }

    #[test]
    fn unit_resolve_runtime_skills_lock_path_prefers_bootstrap_lock_for_default_skills_dir() {
        let mut cli = parse_cli_with_stack();
        let workspace = tempdir().expect("tempdir");
        let skills_dir = workspace.path().join(".tau/skills");
        cli.skills_dir = skills_dir.clone();

        let bootstrap_lock_path = workspace.path().join(".tau/skills.lock.json");
        let resolved = resolve_runtime_skills_lock_path(&cli, &bootstrap_lock_path, &skills_dir);
        assert_eq!(resolved, bootstrap_lock_path);
    }

    #[test]
    fn functional_resolve_runtime_skills_dir_prefers_activated_directory_when_present() {
        let mut cli = parse_cli_with_stack();
        let workspace = tempdir().expect("tempdir");
        cli.skills_dir = workspace.path().join(".tau/skills");
        cli.package_activate_destination = workspace.path().join("packages-active");

        let activated_skills_dir = cli.package_activate_destination.join("skills");
        std::fs::create_dir_all(&activated_skills_dir).expect("create activated skills dir");

        let resolved = resolve_runtime_skills_dir(&cli, true);
        assert_eq!(resolved, activated_skills_dir);
    }

    #[test]
    fn regression_resolve_runtime_skills_dir_falls_back_when_activation_output_missing() {
        let mut cli = parse_cli_with_stack();
        let workspace = tempdir().expect("tempdir");
        let base_skills_dir = workspace.path().join(".tau/skills");
        cli.skills_dir = base_skills_dir.clone();
        cli.package_activate_destination = workspace.path().join("packages-active");

        let resolved = resolve_runtime_skills_dir(&cli, true);
        assert_eq!(resolved, base_skills_dir);
    }

    #[test]
    fn regression_resolve_runtime_skills_lock_path_uses_effective_directory_when_switched() {
        let mut cli = parse_cli_with_stack();
        let workspace = tempdir().expect("tempdir");
        cli.skills_dir = workspace.path().join(".tau/skills");
        let bootstrap_lock_path = workspace.path().join(".tau/skills.lock.json");

        let activated_skills_dir = workspace.path().join("packages-active/skills");
        let resolved =
            resolve_runtime_skills_lock_path(&cli, &bootstrap_lock_path, &activated_skills_dir);

        assert_eq!(resolved, default_skills_lock_path(&activated_skills_dir));
        assert_ne!(resolved, bootstrap_lock_path);
    }

    #[test]
    fn functional_build_startup_runtime_dispatch_context_prefers_activated_runtime_paths() {
        let mut cli = parse_cli_with_stack();
        let workspace = tempdir().expect("tempdir");
        cli.system_prompt = "You are Tau.".to_string();
        cli.skills_dir = workspace.path().join(".tau/skills");
        cli.package_activate_destination = workspace.path().join("packages-active");
        std::fs::create_dir_all(&cli.skills_dir).expect("create skills dir");
        let activated_skills_dir = cli.package_activate_destination.join("skills");
        std::fs::create_dir_all(&activated_skills_dir).expect("create activated skills dir");

        let bootstrap_lock_path = workspace.path().join(".tau/skills.lock.json");
        let context =
            build_startup_runtime_dispatch_context(&cli, &bootstrap_lock_path, true).expect("ok");

        assert_eq!(context.effective_skills_dir, activated_skills_dir);
        assert_eq!(
            context.skills_lock_path,
            default_skills_lock_path(&context.effective_skills_dir)
        );
        assert!(context.system_prompt.contains("You are Tau."));
        assert!(context.startup_policy.tool_policy_json.is_object());
    }

    #[test]
    fn integration_build_startup_runtime_dispatch_context_honors_system_prompt_file() {
        let mut cli = parse_cli_with_stack();
        let workspace = tempdir().expect("tempdir");
        cli.skills_dir = workspace.path().join(".tau/skills");
        std::fs::create_dir_all(&cli.skills_dir).expect("create skills dir");
        let prompt_path = workspace.path().join("system_prompt.txt");
        std::fs::write(&prompt_path, "System prompt from file.").expect("write system prompt");
        cli.system_prompt_file = Some(prompt_path);
        let bootstrap_lock_path = workspace.path().join(".tau/skills.lock.json");

        let context =
            build_startup_runtime_dispatch_context(&cli, &bootstrap_lock_path, false).expect("ok");

        assert!(context.system_prompt.contains("System prompt from file."));
        assert_eq!(context.skills_lock_path, bootstrap_lock_path);
    }

    #[test]
    fn regression_build_startup_runtime_dispatch_context_uses_bootstrap_lock_without_switch() {
        let mut cli = parse_cli_with_stack();
        let workspace = tempdir().expect("tempdir");
        cli.system_prompt = "Tau system prompt".to_string();
        cli.skills_dir = workspace.path().join(".tau/skills");
        std::fs::create_dir_all(&cli.skills_dir).expect("create skills dir");
        let bootstrap_lock_path = workspace.path().join(".tau/skills.lock.json");

        let context =
            build_startup_runtime_dispatch_context(&cli, &bootstrap_lock_path, false).expect("ok");

        assert_eq!(context.effective_skills_dir, cli.skills_dir);
        assert_eq!(context.skills_lock_path, bootstrap_lock_path);
        assert!(context.system_prompt.contains("Tau system prompt"));
    }
}
