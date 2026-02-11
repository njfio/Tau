use super::*;
use tau_onboarding::startup_dispatch::{
    build_startup_runtime_dispatch_context, resolve_startup_model_runtime_from_cli,
    StartupModelRuntimeResolution, StartupRuntimeDispatchContext,
};

pub(crate) async fn run_cli(cli: Cli) -> Result<()> {
    if execute_startup_preflight(&cli)? {
        return Ok(());
    }

    let StartupModelRuntimeResolution {
        model_ref,
        fallback_model_refs,
        model_catalog,
        client,
    } = resolve_startup_model_runtime_from_cli(
        &cli,
        |cli| -> Result<(ModelRef, Vec<ModelRef>)> {
            let StartupModelResolution {
                model_ref,
                fallback_model_refs,
            } = resolve_startup_models(cli)?;
            Ok((model_ref, fallback_model_refs))
        },
        |cli| Box::pin(resolve_startup_model_catalog(cli)),
        |model_catalog, model_ref, fallback_model_refs: &Vec<ModelRef>| {
            validate_startup_model_catalog(model_catalog, model_ref, fallback_model_refs)
        },
        |cli, model_ref, fallback_model_refs: &Vec<ModelRef>| {
            build_client_with_fallbacks(cli, model_ref, fallback_model_refs)
        },
    )
    .await?;
    let skills_bootstrap = run_startup_skills_bootstrap(&cli).await?;
    let startup_package_activation = execute_package_activate_on_startup(&cli)?;
    let StartupRuntimeDispatchContext {
        effective_skills_dir,
        skills_lock_path,
        system_prompt,
        startup_policy,
    } = build_startup_runtime_dispatch_context(
        &cli,
        &skills_bootstrap.skills_lock_path,
        startup_package_activation.is_some(),
    )?;
    let StartupPolicyBundle {
        tool_policy,
        tool_policy_json,
    } = startup_policy;
    let render_options = RenderOptions::from_cli(&cli);
    if run_transport_mode_if_requested(
        &cli,
        &client,
        &model_ref,
        &system_prompt,
        &tool_policy,
        render_options,
    )
    .await?
    {
        return Ok(());
    }

    run_local_runtime(LocalRuntimeConfig {
        cli: &cli,
        client,
        model_ref: &model_ref,
        fallback_model_refs: &fallback_model_refs,
        model_catalog: &model_catalog,
        system_prompt: &system_prompt,
        tool_policy,
        tool_policy_json: &tool_policy_json,
        render_options,
        skills_dir: &effective_skills_dir,
        skills_lock_path: &skills_lock_path,
    })
    .await
}
