use super::*;
use crate::extension_manifest::{
    dispatch_extension_registered_command, ExtensionRegisteredCommandAction,
};
#[cfg(test)]
use crate::runtime_types::{
    ProfileAuthDefaults, ProfileMcpDefaults, ProfilePolicyDefaults, ProfileSessionDefaults,
};
use tau_cli::{canonical_command_name, normalize_help_topic, parse_command, CommandFileReport};
pub(crate) use tau_ops::COMMAND_NAMES;
use tau_session::{
    execute_session_diff_command, execute_session_search_command, execute_session_stats_command,
    parse_session_diff_args, parse_session_stats_args,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandAction {
    Continue,
    Exit,
}

pub(crate) fn execute_command_file(
    path: &Path,
    mode: CliCommandFileErrorMode,
    agent: &mut Agent,
    session_runtime: &mut Option<SessionRuntime>,
    command_context: CommandExecutionContext<'_>,
) -> Result<CommandFileReport> {
    let entries = tau_cli::parse_command_file(path)?;
    let mut report = CommandFileReport {
        total: entries.len(),
        executed: 0,
        succeeded: 0,
        failed: 0,
        halted_early: false,
    };

    for entry in entries {
        report.executed += 1;

        if !entry.command.starts_with('/') {
            report.failed += 1;
            println!(
                "command file error: path={} line={} command={} error=command must start with '/'",
                path.display(),
                entry.line_number,
                entry.command
            );
            if mode == CliCommandFileErrorMode::FailFast {
                report.halted_early = true;
                break;
            }
            continue;
        }

        match handle_command_with_session_import_mode(
            &entry.command,
            agent,
            session_runtime,
            command_context.tool_policy_json,
            command_context.session_import_mode,
            command_context.profile_defaults,
            command_context.skills_command_config,
            command_context.auth_command_config,
            command_context.model_catalog,
            command_context.extension_commands,
        ) {
            Ok(CommandAction::Continue) => {
                report.succeeded += 1;
            }
            Ok(CommandAction::Exit) => {
                report.succeeded += 1;
                report.halted_early = true;
                println!(
                    "command file notice: path={} line={} command={} action=exit",
                    path.display(),
                    entry.line_number,
                    entry.command
                );
                break;
            }
            Err(error) => {
                report.failed += 1;
                println!(
                    "command file error: path={} line={} command={} error={error}",
                    path.display(),
                    entry.line_number,
                    entry.command
                );
                if mode == CliCommandFileErrorMode::FailFast {
                    report.halted_early = true;
                    break;
                }
            }
        }
    }

    println!(
        "command file summary: path={} mode={} total={} executed={} succeeded={} failed={} halted_early={}",
        path.display(),
        command_file_error_mode_label(mode),
        report.total,
        report.executed,
        report.succeeded,
        report.failed,
        report.halted_early
    );

    if mode == CliCommandFileErrorMode::FailFast && report.failed > 0 {
        bail!(
            "command file execution failed: path={} failed={} mode={}",
            path.display(),
            report.failed,
            command_file_error_mode_label(mode)
        );
    }

    Ok(report)
}

#[cfg(test)]
pub(crate) fn handle_command(
    command: &str,
    agent: &mut Agent,
    session_runtime: &mut Option<SessionRuntime>,
    tool_policy_json: &serde_json::Value,
) -> Result<CommandAction> {
    let skills_dir = PathBuf::from(".tau/skills");
    let skills_lock_path = default_skills_lock_path(&skills_dir);
    let skills_command_config = SkillsSyncCommandConfig {
        skills_dir,
        default_lock_path: skills_lock_path,
        default_trust_root_path: None,
        doctor_config: DoctorCommandConfig {
            model: "openai/gpt-4o-mini".to_string(),
            provider_keys: vec![DoctorProviderKeyStatus {
                provider_kind: Provider::OpenAi,
                provider: "openai".to_string(),
                key_env_var: "OPENAI_API_KEY".to_string(),
                present: true,
                auth_mode: ProviderAuthMethod::ApiKey,
                mode_supported: true,
                login_backend_enabled: false,
                login_backend_executable: None,
                login_backend_available: false,
            }],
            release_channel_path: PathBuf::from(".tau/release-channel.json"),
            release_lookup_cache_path: PathBuf::from(".tau/release-lookup-cache.json"),
            release_lookup_cache_ttl_ms: 900_000,
            browser_automation_playwright_cli: "playwright-cli".to_string(),
            session_enabled: true,
            session_path: PathBuf::from(".tau/sessions/default.jsonl"),
            skills_dir: PathBuf::from(".tau/skills"),
            skills_lock_path: PathBuf::from(".tau/skills/skills.lock.json"),
            trust_root_path: None,
            multi_channel_live_readiness: DoctorMultiChannelReadinessConfig::default(),
        },
    };
    let profile_defaults = ProfileDefaults {
        model: "openai/gpt-4o-mini".to_string(),
        fallback_models: Vec::new(),
        session: ProfileSessionDefaults {
            enabled: true,
            path: Some(".tau/sessions/default.jsonl".to_string()),
            import_mode: "merge".to_string(),
        },
        policy: ProfilePolicyDefaults {
            tool_policy_preset: "balanced".to_string(),
            bash_profile: "balanced".to_string(),
            bash_dry_run: false,
            os_sandbox_mode: "off".to_string(),
            enforce_regular_files: true,
            bash_timeout_ms: 500,
            max_command_length: 4096,
            max_tool_output_bytes: 1024,
            max_file_read_bytes: 2048,
            max_file_write_bytes: 2048,
            allow_command_newlines: true,
        },
        mcp: ProfileMcpDefaults::default(),
        auth: ProfileAuthDefaults::default(),
    };
    let auth_command_config = AuthCommandConfig {
        credential_store: PathBuf::from(".tau/credentials.json"),
        credential_store_key: None,
        credential_store_encryption: CredentialStoreEncryptionMode::None,
        api_key: None,
        openai_api_key: None,
        anthropic_api_key: None,
        google_api_key: None,
        openai_auth_mode: ProviderAuthMethod::ApiKey,
        anthropic_auth_mode: ProviderAuthMethod::ApiKey,
        google_auth_mode: ProviderAuthMethod::ApiKey,
        provider_subscription_strict: false,
        openai_codex_backend: true,
        openai_codex_cli: "codex".to_string(),
        anthropic_claude_backend: true,
        anthropic_claude_cli: "claude".to_string(),
        google_gemini_backend: true,
        google_gemini_cli: "gemini".to_string(),
        google_gcloud_cli: "gcloud".to_string(),
    };
    let model_catalog = ModelCatalog::built_in();
    handle_command_with_session_import_mode(
        command,
        agent,
        session_runtime,
        tool_policy_json,
        SessionImportMode::Merge,
        &profile_defaults,
        &skills_command_config,
        &auth_command_config,
        &model_catalog,
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_command_with_session_import_mode(
    command: &str,
    agent: &mut Agent,
    session_runtime: &mut Option<SessionRuntime>,
    tool_policy_json: &serde_json::Value,
    session_import_mode: SessionImportMode,
    profile_defaults: &ProfileDefaults,
    skills_command_config: &SkillsSyncCommandConfig,
    auth_command_config: &AuthCommandConfig,
    model_catalog: &ModelCatalog,
    extension_commands: &[crate::extension_manifest::ExtensionRegisteredCommand],
) -> Result<CommandAction> {
    let skills_dir = skills_command_config.skills_dir.as_path();
    let default_skills_lock_path = skills_command_config.default_lock_path.as_path();
    let default_trust_root_path = skills_command_config.default_trust_root_path.as_deref();

    let Some(parsed) = parse_command(command) else {
        println!("invalid command input: {command}");
        return Ok(CommandAction::Continue);
    };
    let command_name = canonical_command_name(parsed.name);
    let command_args = parsed.args;

    if command_name == "/quit" {
        return Ok(CommandAction::Exit);
    }

    if command_name == "/help" {
        if command_args.is_empty() {
            println!("{}", render_help_overview());
        } else {
            let topic = normalize_help_topic(command_args);
            match render_command_help(&topic) {
                Some(help) => println!("{help}"),
                None => println!("{}", unknown_help_topic_message(&topic)),
            }
        }
        return Ok(CommandAction::Continue);
    }

    if command_name == "/canvas" {
        let session_link = session_runtime
            .as_ref()
            .map(|runtime| CanvasSessionLinkContext {
                session_path: runtime.store.path().to_path_buf(),
                session_head_id: runtime.active_head,
            });
        println!(
            "{}",
            execute_canvas_command(
                command_args,
                &CanvasCommandConfig {
                    canvas_root: PathBuf::from(".tau/canvas"),
                    channel_store_root: PathBuf::from(".tau/channel-store"),
                    principal: resolve_local_principal(),
                    origin: CanvasEventOrigin::default(),
                    session_link,
                }
            )
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/rbac" {
        println!("{}", execute_rbac_command(command_args));
        return Ok(CommandAction::Continue);
    }

    let rbac_principal = resolve_local_principal();
    match authorize_command_for_principal(&rbac_principal, command_name) {
        Ok(RbacDecision::Allow { .. }) => {}
        Ok(RbacDecision::Deny {
            reason_code,
            matched_role,
            matched_pattern,
        }) => {
            println!(
                "rbac gate: status=denied principal={} action=command:{} reason_code={} matched_role={} matched_pattern={}",
                rbac_principal,
                command_name,
                reason_code,
                matched_role.as_deref().unwrap_or("none"),
                matched_pattern.as_deref().unwrap_or("none")
            );
            println!(
                "rbac gate hint: run '/rbac check command:{} --principal {}' for diagnostics",
                command_name, rbac_principal
            );
            return Ok(CommandAction::Continue);
        }
        Err(error) => {
            println!(
                "rbac gate error: principal={} action=command:{} error={error}",
                rbac_principal, command_name
            );
            return Ok(CommandAction::Continue);
        }
    }

    if command_name == "/approvals" {
        println!("{}", execute_approvals_command(command_args));
        return Ok(CommandAction::Continue);
    }

    match evaluate_approval_gate(&ApprovalAction::Command {
        name: command_name.to_string(),
        args: command_args.to_string(),
    }) {
        Ok(ApprovalGateResult::Allowed) => {}
        Ok(ApprovalGateResult::Denied {
            request_id,
            rule_id,
            reason_code,
            message,
        }) => {
            println!(
                "approval gate: status=denied command={} request_id={} rule_id={} reason_code={} message={}",
                command_name, request_id, rule_id, reason_code, message
            );
            println!(
                "approval gate hint: run '/approvals list' then '/approvals approve {}' to continue",
                request_id
            );
            return Ok(CommandAction::Continue);
        }
        Err(error) => {
            println!(
                "approval gate error: command={} error={error}",
                command_name
            );
            return Ok(CommandAction::Continue);
        }
    }

    if command_name == "/session" {
        if !command_args.is_empty() {
            println!("usage: /session");
            return Ok(CommandAction::Continue);
        }
        match session_runtime.as_ref() {
            Some(runtime) => {
                println!(
                    "session: path={} entries={} active_head={}",
                    runtime.store.path().display(),
                    runtime.store.entries().len(),
                    runtime
                        .active_head
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "none".to_string())
                );
            }
            None => println!("session: disabled"),
        }
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-search" {
        let Some(runtime) = session_runtime.as_ref() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };
        if command_args.trim().is_empty() {
            println!("usage: /session-search <query> [--role <role>] [--limit <n>]");
            return Ok(CommandAction::Continue);
        }

        println!("{}", execute_session_search_command(runtime, command_args));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-stats" {
        let Some(runtime) = session_runtime.as_ref() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };
        let format = match parse_session_stats_args(command_args) {
            Ok(format) => format,
            Err(_) => {
                println!("usage: /session-stats [--json]");
                return Ok(CommandAction::Continue);
            }
        };

        println!("{}", execute_session_stats_command(runtime, format));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-diff" {
        let Some(runtime) = session_runtime.as_ref() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };
        let heads = match parse_session_diff_args(command_args) {
            Ok(heads) => heads,
            Err(_) => {
                println!("usage: /session-diff [<left-id> <right-id>]");
                return Ok(CommandAction::Continue);
            }
        };

        println!("{}", execute_session_diff_command(runtime, heads));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/qa-loop" {
        println!("{}", execute_qa_loop_cli_command(command_args));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/doctor" {
        println!(
            "{}",
            execute_doctor_cli_command(&skills_command_config.doctor_config, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-graph-export" {
        let Some(runtime) = session_runtime.as_ref() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };
        if command_args.trim().is_empty() {
            println!("usage: /session-graph-export <path>");
            return Ok(CommandAction::Continue);
        }

        println!(
            "{}",
            execute_session_graph_export_command(runtime, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-export" {
        let Some(runtime) = session_runtime.as_ref() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };
        if command_args.is_empty() {
            println!("usage: /session-export <path>");
            return Ok(CommandAction::Continue);
        }

        let destination = PathBuf::from(command_args);
        let exported = runtime
            .store
            .export_lineage(runtime.active_head, &destination)?;
        println!(
            "session export complete: path={} entries={} head={}",
            destination.display(),
            exported,
            runtime
                .active_head
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-import" {
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };
        if command_args.is_empty() {
            println!("usage: /session-import <path>");
            return Ok(CommandAction::Continue);
        }

        let source = PathBuf::from(command_args);
        let report = runtime
            .store
            .import_snapshot(&source, session_import_mode)?;
        runtime.active_head = report.active_head;
        agent.replace_messages(session_lineage_messages(runtime)?);
        println!(
            "session import complete: path={} mode={} imported_entries={} remapped_entries={} remapped_ids={} replaced_entries={} total_entries={} head={}",
            source.display(),
            session_import_mode_label(session_import_mode),
            report.imported_entries,
            report.remapped_entries,
            format_remap_ids(&report.remapped_ids),
            report.replaced_entries,
            report.resulting_entries,
            runtime
                .active_head
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/policy" {
        match execute_policy_command(command_args, tool_policy_json) {
            Ok(output) => println!("{output}"),
            Err(_) => println!("usage: /policy"),
        }
        return Ok(CommandAction::Continue);
    }

    if command_name == "/audit-summary" {
        println!("{}", execute_audit_summary_command(command_args));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/models-list" {
        match parse_models_list_args(command_args) {
            Ok(args) => println!("{}", render_models_list(model_catalog, &args)),
            Err(error) => {
                println!("models list error: {error}");
                println!("usage: {MODELS_LIST_USAGE}");
            }
        }
        return Ok(CommandAction::Continue);
    }

    if command_name == "/model-show" {
        if command_args.is_empty() {
            println!("usage: {MODEL_SHOW_USAGE}");
            return Ok(CommandAction::Continue);
        }
        match render_model_show(model_catalog, command_args) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                println!("model show error: {error}");
                println!("usage: {MODEL_SHOW_USAGE}");
            }
        }
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-search" {
        if command_args.is_empty() {
            println!("usage: /skills-search <query> [max_results]");
            return Ok(CommandAction::Continue);
        }
        println!(
            "{}",
            execute_skills_search_command(skills_dir, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-show" {
        if command_args.is_empty() {
            println!("usage: /skills-show <name>");
            return Ok(CommandAction::Continue);
        }
        println!("{}", execute_skills_show_command(skills_dir, command_args));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-list" {
        if !command_args.is_empty() {
            println!("usage: /skills-list");
            return Ok(CommandAction::Continue);
        }
        println!("{}", execute_skills_list_command(skills_dir));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-lock-diff" {
        println!(
            "{}",
            execute_skills_lock_diff_command(skills_dir, default_skills_lock_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-prune" {
        println!(
            "{}",
            execute_skills_prune_command(skills_dir, default_skills_lock_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-trust-list" {
        println!(
            "{}",
            execute_skills_trust_list_command(default_trust_root_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-trust-add" {
        println!(
            "{}",
            execute_skills_trust_add_command(default_trust_root_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-trust-revoke" {
        println!(
            "{}",
            execute_skills_trust_revoke_command(default_trust_root_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-trust-rotate" {
        println!(
            "{}",
            execute_skills_trust_rotate_command(default_trust_root_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-lock-write" {
        println!(
            "{}",
            execute_skills_lock_write_command(skills_dir, default_skills_lock_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-sync" {
        println!(
            "{}",
            execute_skills_sync_command(skills_dir, default_skills_lock_path, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/skills-verify" {
        println!(
            "{}",
            execute_skills_verify_command(
                skills_dir,
                default_skills_lock_path,
                default_trust_root_path,
                command_args
            )
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/resume" {
        if !command_args.is_empty() {
            println!("usage: /resume");
            return Ok(CommandAction::Continue);
        }
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        runtime.active_head = runtime.store.head_id();
        agent.replace_messages(session_lineage_messages(runtime)?);
        println!(
            "resumed at head {}",
            runtime
                .active_head
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/branches" {
        if !command_args.is_empty() {
            println!("usage: /branches");
            return Ok(CommandAction::Continue);
        }
        let Some(runtime) = session_runtime.as_ref() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        let tips = runtime.store.branch_tips();
        if tips.is_empty() {
            println!("no branches");
        } else {
            for tip in tips {
                println!(
                    "id={} parent={} text={}",
                    tip.id,
                    tip.parent_id
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    summarize_message(&tip.message)
                );
            }
        }

        return Ok(CommandAction::Continue);
    }

    if command_name == "/macro" {
        let macro_path = match default_macro_config_path() {
            Ok(path) => path,
            Err(error) => {
                println!("macro error: path=unknown error={error}");
                return Ok(CommandAction::Continue);
            }
        };
        println!(
            "{}",
            execute_macro_command(
                command_args,
                &macro_path,
                agent,
                session_runtime,
                CommandExecutionContext {
                    tool_policy_json,
                    session_import_mode,
                    profile_defaults,
                    skills_command_config,
                    auth_command_config,
                    model_catalog,
                    extension_commands,
                }
            )
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/profile" {
        let profile_path = match default_profile_store_path() {
            Ok(path) => path,
            Err(error) => {
                println!("profile error: path=unknown error={error}");
                return Ok(CommandAction::Continue);
            }
        };
        println!(
            "{}",
            execute_profile_command(command_args, &profile_path, profile_defaults)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/release-channel" {
        let release_channel_path = match default_release_channel_path() {
            Ok(path) => path,
            Err(error) => {
                println!("release channel error: path=unknown error={error}");
                return Ok(CommandAction::Continue);
            }
        };
        println!(
            "{}",
            execute_release_channel_command(command_args, &release_channel_path)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/auth" {
        println!(
            "{}",
            execute_auth_command(auth_command_config, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/integration-auth" {
        println!(
            "{}",
            execute_integration_auth_command(auth_command_config, command_args)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/pair" {
        println!("{}", execute_pair_command(command_args, "local"));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/unpair" {
        println!("{}", execute_unpair_command(command_args));
        return Ok(CommandAction::Continue);
    }

    if command_name == "/branch-alias" {
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        let outcome = execute_branch_alias_command(command_args, runtime);
        if outcome.reload_active_head {
            let lineage = session_lineage_messages(runtime)?;
            agent.replace_messages(lineage);
        }
        println!("{}", outcome.message);
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-bookmark" {
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        let outcome = execute_session_bookmark_command(command_args, runtime);
        if outcome.reload_active_head {
            let lineage = session_lineage_messages(runtime)?;
            agent.replace_messages(lineage);
        }
        println!("{}", outcome.message);
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-repair" {
        if !command_args.is_empty() {
            println!("usage: /session-repair");
            return Ok(CommandAction::Continue);
        }
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        let report = runtime.store.repair()?;
        runtime.active_head = runtime
            .active_head
            .filter(|head| runtime.store.contains(*head))
            .or_else(|| runtime.store.head_id());
        agent.replace_messages(session_lineage_messages(runtime)?);

        println!(
            "repair complete: removed_duplicates={} duplicate_ids={} removed_invalid_parent={} invalid_parent_ids={} removed_cycles={} cycle_ids={}",
            report.removed_duplicates,
            format_id_list(&report.duplicate_ids),
            report.removed_invalid_parent,
            format_id_list(&report.invalid_parent_ids),
            report.removed_cycles,
            format_id_list(&report.cycle_ids)
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/session-compact" {
        if !command_args.is_empty() {
            println!("usage: /session-compact");
            return Ok(CommandAction::Continue);
        }
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        let report = runtime.store.compact_to_lineage(runtime.active_head)?;
        runtime.active_head = report
            .head_id
            .filter(|head| runtime.store.contains(*head))
            .or_else(|| runtime.store.head_id());
        agent.replace_messages(session_lineage_messages(runtime)?);

        println!(
            "compact complete: removed_entries={} retained_entries={} head={}",
            report.removed_entries,
            report.retained_entries,
            runtime
                .active_head
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
        return Ok(CommandAction::Continue);
    }

    if command_name == "/branch" {
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };
        if command_args.is_empty() {
            println!("usage: /branch <id>");
            return Ok(CommandAction::Continue);
        }

        let target = command_args
            .parse::<u64>()
            .map_err(|_| anyhow!("invalid branch id '{}'; expected an integer", command_args))?;

        if !runtime.store.contains(target) {
            bail!("unknown session id {}", target);
        }

        runtime.active_head = Some(target);
        agent.replace_messages(session_lineage_messages(runtime)?);
        println!("switched to branch id {target}");
        return Ok(CommandAction::Continue);
    }

    match dispatch_extension_registered_command(extension_commands, command_name, command_args) {
        Ok(Some(dispatch_result)) => {
            if let Some(output) = dispatch_result.output {
                println!("{output}");
            }
            return Ok(match dispatch_result.action {
                ExtensionRegisteredCommandAction::Continue => CommandAction::Continue,
                ExtensionRegisteredCommandAction::Exit => CommandAction::Exit,
            });
        }
        Ok(None) => {}
        Err(error) => {
            println!("extension command error: command={command_name} error={error}");
            return Ok(CommandAction::Continue);
        }
    }

    println!("{}", unknown_command_message(parsed.name));
    Ok(CommandAction::Continue)
}

pub(crate) fn session_import_mode_label(mode: SessionImportMode) -> &'static str {
    match mode {
        SessionImportMode::Merge => "merge",
        SessionImportMode::Replace => "replace",
    }
}

pub(crate) fn render_help_overview() -> String {
    tau_ops::render_help_overview()
}

pub(crate) fn render_command_help(topic: &str) -> Option<String> {
    tau_ops::render_command_help(topic)
}

pub(crate) fn unknown_help_topic_message(topic: &str) -> String {
    tau_ops::unknown_help_topic_message(topic)
}

pub(crate) fn unknown_command_message(command: &str) -> String {
    tau_ops::unknown_command_message(command)
}
