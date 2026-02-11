pub use tau_extensions::*;

use anyhow::{anyhow, Result};

use crate::Cli;

pub(crate) fn execute_extension_list_command(cli: &Cli) -> Result<()> {
    if !cli.extension_list {
        return Ok(());
    }
    let report = list_extension_manifests(&cli.extension_list_root)?;
    println!("{}", render_extension_list_report(&report));
    Ok(())
}

pub(crate) fn execute_extension_exec_command(cli: &Cli) -> Result<()> {
    let Some(manifest_path) = cli.extension_exec_manifest.as_ref() else {
        return Ok(());
    };
    let hook = cli
        .extension_exec_hook
        .as_deref()
        .ok_or_else(|| anyhow!("--extension-exec-hook is required"))?;
    let payload_file = cli
        .extension_exec_payload_file
        .as_ref()
        .ok_or_else(|| anyhow!("--extension-exec-payload-file is required"))?;
    let payload = load_extension_exec_payload(payload_file)?;
    let summary = execute_extension_process_hook(manifest_path, hook, &payload)?;
    println!(
        "extension exec: path={} id={} version={} runtime={} hook={} timeout_ms={} duration_ms={} response_bytes={}",
        summary.manifest_path.display(),
        summary.id,
        summary.version,
        summary.runtime,
        summary.hook,
        summary.timeout_ms,
        summary.duration_ms,
        summary.response_bytes
    );
    println!("extension exec response: {}", summary.response);
    Ok(())
}

pub(crate) fn execute_extension_show_command(cli: &Cli) -> Result<()> {
    let Some(path) = cli.extension_show.as_ref() else {
        return Ok(());
    };
    let (manifest, summary) = load_and_validate_extension_manifest(path)?;
    println!("{}", render_extension_manifest_report(&summary, &manifest));
    Ok(())
}

pub(crate) fn execute_extension_validate_command(cli: &Cli) -> Result<()> {
    let Some(path) = cli.extension_validate.as_ref() else {
        return Ok(());
    };
    let summary = validate_extension_manifest(path)?;
    println!(
        "extension validate: path={} id={} version={} runtime={} entrypoint={} hooks={} permissions={} timeout_ms={}",
        summary.manifest_path.display(),
        summary.id,
        summary.version,
        summary.runtime,
        summary.entrypoint,
        summary.hook_count,
        summary.permission_count,
        summary.timeout_ms
    );
    Ok(())
}
