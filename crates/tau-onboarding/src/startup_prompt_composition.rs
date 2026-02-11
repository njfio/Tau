use std::path::Path;

use anyhow::{Context, Result};
use tau_cli::Cli;
use tau_skills::{augment_system_prompt, load_catalog, resolve_selected_skills};

use crate::startup_resolution::resolve_system_prompt;

pub fn compose_startup_system_prompt(cli: &Cli, skills_dir: &Path) -> Result<String> {
    let base_system_prompt = resolve_system_prompt(cli)?;
    let catalog = load_catalog(skills_dir)
        .with_context(|| format!("failed to load skills from {}", skills_dir.display()))?;
    let selected_skills = resolve_selected_skills(&catalog, &cli.skills)?;
    Ok(augment_system_prompt(&base_system_prompt, &selected_skills))
}
