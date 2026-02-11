use anyhow::{Context, Result};

pub use tau_deployment::deployment_wasm::{
    inspect_deployment_wasm_deliverable, package_deployment_wasm_artifact,
    render_deployment_wasm_inspect_report, render_deployment_wasm_package_report,
    DeploymentWasmPackageConfig,
};

use crate::Cli;

pub(crate) fn execute_deployment_wasm_package_command(cli: &Cli) -> Result<()> {
    let Some(module_path) = cli.deployment_wasm_package_module.clone() else {
        return Ok(());
    };
    let report = package_deployment_wasm_artifact(&DeploymentWasmPackageConfig {
        module_path,
        blueprint_id: cli.deployment_wasm_package_blueprint_id.clone(),
        runtime_profile: cli
            .deployment_wasm_package_runtime_profile
            .as_str()
            .to_string(),
        output_dir: cli.deployment_wasm_package_output_dir.clone(),
        state_dir: cli.deployment_state_dir.clone(),
    })?;
    if cli.deployment_wasm_package_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .context("failed to render deployment wasm package report json")?
        );
    } else {
        println!("{}", render_deployment_wasm_package_report(&report));
    }
    Ok(())
}

pub(crate) fn execute_deployment_wasm_inspect_command(cli: &Cli) -> Result<()> {
    let Some(manifest_path) = cli.deployment_wasm_inspect_manifest.clone() else {
        return Ok(());
    };
    let report = inspect_deployment_wasm_deliverable(&manifest_path)?;
    if cli.deployment_wasm_inspect_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .context("failed to render deployment wasm inspect report json")?
        );
    } else {
        println!("{}", render_deployment_wasm_inspect_report(&report));
    }
    Ok(())
}
