use super::*;

struct TauStartupPreflightActions;

impl tau_startup::StartupPreflightActions for TauStartupPreflightActions {
    fn execute_onboarding_command(&self, cli: &Cli) -> Result<()> {
        execute_onboarding_command(cli)
    }

    fn execute_multi_channel_send_command(&self, cli: &Cli) -> Result<()> {
        crate::channel_send::execute_multi_channel_send_command(cli)
    }

    fn execute_multi_channel_channel_lifecycle_command(&self, cli: &Cli) -> Result<()> {
        crate::channel_lifecycle::execute_multi_channel_channel_lifecycle_command(cli)
    }

    fn execute_deployment_wasm_package_command(&self, cli: &Cli) -> Result<()> {
        crate::deployment_wasm::execute_deployment_wasm_package_command(cli)
    }

    fn execute_deployment_wasm_inspect_command(&self, cli: &Cli) -> Result<()> {
        crate::deployment_wasm::execute_deployment_wasm_inspect_command(cli)
    }

    fn execute_project_index_command(&self, cli: &Cli) -> Result<()> {
        execute_project_index_command(cli)
    }

    fn execute_channel_store_admin_command(&self, cli: &Cli) -> Result<()> {
        execute_channel_store_admin_command(cli)
    }

    fn execute_multi_channel_live_readiness_preflight_command(&self, cli: &Cli) -> Result<()> {
        execute_multi_channel_live_readiness_preflight_command(cli)
    }

    fn execute_browser_automation_preflight_command(&self, cli: &Cli) -> Result<()> {
        execute_browser_automation_preflight_command(cli)
    }

    fn execute_extension_exec_command(&self, cli: &Cli) -> Result<()> {
        execute_extension_exec_command(cli)
    }

    fn execute_extension_list_command(&self, cli: &Cli) -> Result<()> {
        execute_extension_list_command(cli)
    }

    fn execute_extension_show_command(&self, cli: &Cli) -> Result<()> {
        execute_extension_show_command(cli)
    }

    fn execute_extension_validate_command(&self, cli: &Cli) -> Result<()> {
        execute_extension_validate_command(cli)
    }

    fn execute_package_validate_command(&self, cli: &Cli) -> Result<()> {
        execute_package_validate_command(cli)
    }

    fn execute_package_show_command(&self, cli: &Cli) -> Result<()> {
        execute_package_show_command(cli)
    }

    fn execute_package_install_command(&self, cli: &Cli) -> Result<()> {
        execute_package_install_command(cli)
    }

    fn execute_package_update_command(&self, cli: &Cli) -> Result<()> {
        execute_package_update_command(cli)
    }

    fn execute_package_list_command(&self, cli: &Cli) -> Result<()> {
        execute_package_list_command(cli)
    }

    fn execute_package_remove_command(&self, cli: &Cli) -> Result<()> {
        execute_package_remove_command(cli)
    }

    fn execute_package_rollback_command(&self, cli: &Cli) -> Result<()> {
        execute_package_rollback_command(cli)
    }

    fn execute_package_conflicts_command(&self, cli: &Cli) -> Result<()> {
        execute_package_conflicts_command(cli)
    }

    fn execute_package_activate_command(&self, cli: &Cli) -> Result<()> {
        execute_package_activate_command(cli)
    }

    fn execute_qa_loop_preflight_command(&self, cli: &Cli) -> Result<()> {
        execute_qa_loop_preflight_command(cli)
    }

    fn execute_mcp_server_command(&self, cli: &Cli) -> Result<()> {
        execute_mcp_server_command(cli)
    }

    fn execute_rpc_capabilities_command(&self, cli: &Cli) -> Result<()> {
        execute_rpc_capabilities_command(cli)
    }

    fn execute_rpc_validate_frame_command(&self, cli: &Cli) -> Result<()> {
        execute_rpc_validate_frame_command(cli)
    }

    fn execute_rpc_dispatch_frame_command(&self, cli: &Cli) -> Result<()> {
        execute_rpc_dispatch_frame_command(cli)
    }

    fn execute_rpc_dispatch_ndjson_command(&self, cli: &Cli) -> Result<()> {
        execute_rpc_dispatch_ndjson_command(cli)
    }

    fn execute_rpc_serve_ndjson_command(&self, cli: &Cli) -> Result<()> {
        execute_rpc_serve_ndjson_command(cli)
    }

    fn execute_events_inspect_command(&self, cli: &Cli) -> Result<()> {
        execute_events_inspect_command(cli)
    }

    fn execute_events_validate_command(&self, cli: &Cli) -> Result<()> {
        execute_events_validate_command(cli)
    }

    fn execute_events_simulate_command(&self, cli: &Cli) -> Result<()> {
        execute_events_simulate_command(cli)
    }

    fn execute_events_dry_run_command(&self, cli: &Cli) -> Result<()> {
        execute_events_dry_run_command(cli)
    }

    fn execute_events_template_write_command(&self, cli: &Cli) -> Result<()> {
        execute_events_template_write_command(cli)
    }

    fn resolve_secret_from_cli_or_store_id(
        &self,
        cli: &Cli,
        direct_secret: Option<&str>,
        secret_id: Option<&str>,
        secret_id_flag: &str,
    ) -> Result<Option<String>> {
        resolve_secret_from_cli_or_store_id(cli, direct_secret, secret_id, secret_id_flag)
    }

    fn handle_daemon_commands(&self, cli: &Cli) -> Result<bool> {
        tau_onboarding::startup_daemon_preflight::handle_daemon_commands(cli)
    }
}

pub(crate) fn execute_startup_preflight(cli: &Cli) -> Result<bool> {
    tau_startup::execute_startup_preflight(cli, &TauStartupPreflightActions)
}
