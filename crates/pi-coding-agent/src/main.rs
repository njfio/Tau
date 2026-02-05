mod session;
mod tools;

use std::{io::Write, path::PathBuf, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use pi_agent_core::{Agent, AgentConfig, AgentEvent};
use pi_ai::{
    AnthropicClient, AnthropicConfig, GoogleClient, GoogleConfig, LlmClient, Message, MessageRole,
    ModelRef, OpenAiClient, OpenAiConfig, Provider,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::session::SessionStore;

#[derive(Debug, Parser)]
#[command(
    name = "pi-rs",
    about = "Pure Rust coding agent inspired by pi-mono",
    version
)]
struct Cli {
    #[arg(
        long,
        env = "PI_MODEL",
        default_value = "openai/gpt-4o-mini",
        help = "Model in provider/model format. Supported providers: openai, anthropic, google."
    )]
    model: String,

    #[arg(
        long,
        env = "PI_API_BASE",
        default_value = "https://api.openai.com/v1",
        help = "Base URL for OpenAI-compatible APIs"
    )]
    api_base: String,

    #[arg(
        long,
        env = "PI_ANTHROPIC_API_BASE",
        default_value = "https://api.anthropic.com/v1",
        help = "Base URL for Anthropic Messages API"
    )]
    anthropic_api_base: String,

    #[arg(
        long,
        env = "PI_GOOGLE_API_BASE",
        default_value = "https://generativelanguage.googleapis.com/v1beta",
        help = "Base URL for Google Gemini API"
    )]
    google_api_base: String,

    #[arg(
        long,
        env = "PI_API_KEY",
        hide_env_values = true,
        help = "Generic API key fallback"
    )]
    api_key: Option<String>,

    #[arg(
        long,
        env = "OPENAI_API_KEY",
        hide_env_values = true,
        help = "API key for OpenAI-compatible APIs"
    )]
    openai_api_key: Option<String>,

    #[arg(
        long,
        env = "ANTHROPIC_API_KEY",
        hide_env_values = true,
        help = "API key for Anthropic"
    )]
    anthropic_api_key: Option<String>,

    #[arg(
        long,
        env = "GEMINI_API_KEY",
        hide_env_values = true,
        help = "API key for Google Gemini"
    )]
    google_api_key: Option<String>,

    #[arg(
        long,
        env = "PI_SYSTEM_PROMPT",
        default_value = "You are a focused coding assistant. Prefer concrete steps and safe edits.",
        help = "System prompt"
    )]
    system_prompt: String,

    #[arg(long, env = "PI_MAX_TURNS", default_value_t = 8)]
    max_turns: usize,

    #[arg(long, help = "Print agent lifecycle events as JSON")]
    json_events: bool,

    #[arg(long, help = "Run one prompt and exit")]
    prompt: Option<String>,

    #[arg(
        long,
        env = "PI_SESSION",
        default_value = ".pi/sessions/default.jsonl",
        help = "Session JSONL file"
    )]
    session: PathBuf,

    #[arg(long, help = "Disable session persistence")]
    no_session: bool,

    #[arg(long, help = "Start from a specific session entry id")]
    branch_from: Option<u64>,
}

#[derive(Debug)]
struct SessionRuntime {
    store: SessionStore,
    active_head: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandAction {
    Continue,
    Exit,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    if cli.no_session && cli.branch_from.is_some() {
        bail!("--branch-from cannot be used together with --no-session");
    }

    let model_ref = ModelRef::parse(&cli.model)
        .map_err(|error| anyhow!("failed to parse --model '{}': {error}", cli.model))?;

    let client = build_client(&cli, model_ref.provider)
        .with_context(|| format!("failed to create {} client", model_ref.provider))?;

    let mut agent = Agent::new(
        client,
        AgentConfig {
            model: model_ref.model,
            system_prompt: cli.system_prompt.clone(),
            max_turns: cli.max_turns,
            temperature: Some(0.0),
            max_tokens: None,
        },
    );

    tools::register_builtin_tools(&mut agent);

    let mut session_runtime = if cli.no_session {
        None
    } else {
        Some(initialize_session(&mut agent, &cli)?)
    };

    if cli.json_events {
        agent.subscribe(|event| {
            let value = event_to_json(event);
            println!("{value}");
        });
    }

    if let Some(prompt) = cli.prompt {
        run_prompt(&mut agent, &mut session_runtime, &prompt).await?;
        return Ok(());
    }

    run_interactive(agent, session_runtime).await
}

fn initialize_session(agent: &mut Agent, cli: &Cli) -> Result<SessionRuntime> {
    let mut store = SessionStore::load(&cli.session)?;

    let mut active_head = store.ensure_initialized(&cli.system_prompt)?;
    if let Some(branch_id) = cli.branch_from {
        if !store.contains(branch_id) {
            bail!(
                "session {} does not contain entry id {}",
                store.path().display(),
                branch_id
            );
        }
        active_head = Some(branch_id);
    }

    let lineage = store.lineage_messages(active_head)?;
    if !lineage.is_empty() {
        agent.replace_messages(lineage);
    }

    Ok(SessionRuntime { store, active_head })
}

async fn run_prompt(
    agent: &mut Agent,
    session_runtime: &mut Option<SessionRuntime>,
    prompt: &str,
) -> Result<()> {
    let new_messages = agent.prompt(prompt).await?;
    persist_messages(session_runtime, &new_messages)?;
    print_assistant_messages(&new_messages);
    Ok(())
}

async fn run_interactive(
    mut agent: Agent,
    mut session_runtime: Option<SessionRuntime>,
) -> Result<()> {
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    loop {
        print!("pi> ");
        std::io::stdout()
            .flush()
            .context("failed to flush stdout")?;

        let Some(line) = lines.next_line().await? else {
            break;
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('/') {
            if handle_command(trimmed, &mut agent, &mut session_runtime)? == CommandAction::Exit {
                break;
            }
            continue;
        }

        let new_messages = agent.prompt(trimmed).await?;
        persist_messages(&mut session_runtime, &new_messages)?;
        print_assistant_messages(&new_messages);
    }

    Ok(())
}

fn handle_command(
    command: &str,
    agent: &mut Agent,
    session_runtime: &mut Option<SessionRuntime>,
) -> Result<CommandAction> {
    if matches!(command, "/exit" | "/quit") {
        return Ok(CommandAction::Exit);
    }

    if command == "/session" {
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

    if command == "/resume" {
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        runtime.active_head = runtime.store.head_id();
        reload_agent_from_active_head(agent, runtime)?;
        println!(
            "resumed at head {}",
            runtime
                .active_head
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
        return Ok(CommandAction::Continue);
    }

    if command == "/branches" {
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

    if let Some(rest) = command.strip_prefix("/branch ") {
        let Some(runtime) = session_runtime.as_mut() else {
            println!("session is disabled");
            return Ok(CommandAction::Continue);
        };

        let target = rest
            .trim()
            .parse::<u64>()
            .map_err(|_| anyhow!("invalid branch id '{}'; expected an integer", rest.trim()))?;

        if !runtime.store.contains(target) {
            bail!("unknown session id {}", target);
        }

        runtime.active_head = Some(target);
        reload_agent_from_active_head(agent, runtime)?;
        println!("switched to branch id {target}");
        return Ok(CommandAction::Continue);
    }

    println!(
        "unknown command: {}\ncommands: /session, /branches, /branch <id>, /resume, /quit",
        command
    );
    Ok(CommandAction::Continue)
}

fn reload_agent_from_active_head(agent: &mut Agent, runtime: &SessionRuntime) -> Result<()> {
    let lineage = runtime.store.lineage_messages(runtime.active_head)?;
    agent.replace_messages(lineage);
    Ok(())
}

fn summarize_message(message: &Message) -> String {
    let text = message.text_content().replace('\n', " ");
    if text.trim().is_empty() {
        return format!(
            "{:?} (tool_calls={})",
            message.role,
            message.tool_calls().len()
        );
    }

    let max = 60;
    if text.chars().count() <= max {
        text
    } else {
        let summary = text.chars().take(max).collect::<String>();
        format!("{summary}...")
    }
}

fn persist_messages(
    session_runtime: &mut Option<SessionRuntime>,
    new_messages: &[Message],
) -> Result<()> {
    let Some(runtime) = session_runtime.as_mut() else {
        return Ok(());
    };

    runtime.active_head = runtime
        .store
        .append_messages(runtime.active_head, new_messages)?;
    Ok(())
}

fn print_assistant_messages(messages: &[Message]) {
    for message in messages {
        if message.role != MessageRole::Assistant {
            continue;
        }

        let text = message.text_content();
        if !text.trim().is_empty() {
            println!("\n{text}\n");
            continue;
        }

        let tool_calls = message.tool_calls();
        if !tool_calls.is_empty() {
            println!(
                "\n[assistant requested {} tool call(s)]\n",
                tool_calls.len()
            );
        }
    }
}

fn event_to_json(event: &AgentEvent) -> serde_json::Value {
    match event {
        AgentEvent::AgentStart => serde_json::json!({ "type": "agent_start" }),
        AgentEvent::AgentEnd { new_messages } => {
            serde_json::json!({ "type": "agent_end", "new_messages": new_messages })
        }
        AgentEvent::TurnStart { turn } => serde_json::json!({ "type": "turn_start", "turn": turn }),
        AgentEvent::TurnEnd { turn, tool_results } => {
            serde_json::json!({ "type": "turn_end", "turn": turn, "tool_results": tool_results })
        }
        AgentEvent::MessageAdded { message } => serde_json::json!({
            "type": "message_added",
            "role": format!("{:?}", message.role).to_lowercase(),
            "text": message.text_content(),
            "tool_calls": message.tool_calls().len(),
        }),
        AgentEvent::ToolExecutionStart {
            tool_call_id,
            tool_name,
            arguments,
        } => serde_json::json!({
            "type": "tool_execution_start",
            "tool_call_id": tool_call_id,
            "tool_name": tool_name,
            "arguments": arguments,
        }),
        AgentEvent::ToolExecutionEnd {
            tool_call_id,
            tool_name,
            result,
        } => serde_json::json!({
            "type": "tool_execution_end",
            "tool_call_id": tool_call_id,
            "tool_name": tool_name,
            "is_error": result.is_error,
            "content": result.content,
        }),
    }
}

fn build_client(cli: &Cli, provider: Provider) -> Result<Arc<dyn LlmClient>> {
    match provider {
        Provider::OpenAi => {
            let api_key = resolve_api_key(vec![
                cli.openai_api_key.clone(),
                cli.api_key.clone(),
                std::env::var("OPENAI_API_KEY").ok(),
                std::env::var("PI_API_KEY").ok(),
            ])
            .ok_or_else(|| {
                anyhow!(
                    "missing OpenAI API key. Set OPENAI_API_KEY, PI_API_KEY, --openai-api-key, or --api-key"
                )
            })?;

            let client = OpenAiClient::new(OpenAiConfig {
                api_base: cli.api_base.clone(),
                api_key,
                organization: None,
            })?;
            Ok(Arc::new(client))
        }
        Provider::Anthropic => {
            let api_key = resolve_api_key(vec![
                cli.anthropic_api_key.clone(),
                cli.api_key.clone(),
                std::env::var("ANTHROPIC_API_KEY").ok(),
                std::env::var("PI_API_KEY").ok(),
            ])
            .ok_or_else(|| {
                anyhow!(
                    "missing Anthropic API key. Set ANTHROPIC_API_KEY, PI_API_KEY, --anthropic-api-key, or --api-key"
                )
            })?;

            let client = AnthropicClient::new(AnthropicConfig {
                api_base: cli.anthropic_api_base.clone(),
                api_key,
            })?;
            Ok(Arc::new(client))
        }
        Provider::Google => {
            let api_key = resolve_api_key(vec![
                cli.google_api_key.clone(),
                cli.api_key.clone(),
                std::env::var("GEMINI_API_KEY").ok(),
                std::env::var("GOOGLE_API_KEY").ok(),
                std::env::var("PI_API_KEY").ok(),
            ])
            .ok_or_else(|| {
                anyhow!(
                    "missing Google API key. Set GEMINI_API_KEY, GOOGLE_API_KEY, PI_API_KEY, --google-api-key, or --api-key"
                )
            })?;

            let client = GoogleClient::new(GoogleConfig {
                api_base: cli.google_api_base.clone(),
                api_key,
            })?;
            Ok(Arc::new(client))
        }
    }
}

fn resolve_api_key(candidates: Vec<Option<String>>) -> Option<String> {
    candidates
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

fn init_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use async_trait::async_trait;
    use pi_agent_core::{Agent, AgentConfig};
    use pi_ai::PiAiError;
    use tempfile::tempdir;

    use super::{handle_command, CommandAction, SessionRuntime};
    use crate::resolve_api_key;
    use crate::session::SessionStore;

    struct NoopClient;

    #[async_trait]
    impl pi_ai::LlmClient for NoopClient {
        async fn complete(
            &self,
            _request: pi_ai::ChatRequest,
        ) -> Result<pi_ai::ChatResponse, PiAiError> {
            Err(PiAiError::InvalidResponse(
                "noop client should not be called".to_string(),
            ))
        }
    }

    #[test]
    fn resolve_api_key_uses_first_non_empty_candidate() {
        let key = resolve_api_key(vec![
            Some("".to_string()),
            Some("  ".to_string()),
            Some("abc".to_string()),
            Some("def".to_string()),
        ]);

        assert_eq!(key, Some("abc".to_string()));
    }

    #[test]
    fn resolve_api_key_returns_none_when_all_candidates_are_empty() {
        let key = resolve_api_key(vec![None, Some("".to_string())]);
        assert!(key.is_none());
    }

    #[test]
    fn pathbuf_from_cli_default_is_relative() {
        let path = PathBuf::from(".pi/sessions/default.jsonl");
        assert!(!path.is_absolute());
    }

    #[test]
    fn branch_and_resume_commands_reload_agent_messages() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");

        let mut store = SessionStore::load(&path).expect("load");
        let head = store
            .append_messages(None, &[pi_ai::Message::system("sys")])
            .expect("append");
        let head = store
            .append_messages(
                head,
                &[
                    pi_ai::Message::user("q1"),
                    pi_ai::Message::assistant_text("a1"),
                    pi_ai::Message::user("q2"),
                    pi_ai::Message::assistant_text("a2"),
                ],
            )
            .expect("append")
            .expect("head id");

        let branch_target = head - 2;

        let mut agent = Agent::new(Arc::new(NoopClient), AgentConfig::default());
        let lineage = store
            .lineage_messages(Some(head))
            .expect("lineage should resolve");
        agent.replace_messages(lineage);

        let mut runtime = Some(SessionRuntime {
            store,
            active_head: Some(head),
        });

        let action = handle_command(
            &format!("/branch {branch_target}"),
            &mut agent,
            &mut runtime,
        )
        .expect("branch command should succeed");
        assert_eq!(action, CommandAction::Continue);
        assert_eq!(
            runtime.as_ref().and_then(|runtime| runtime.active_head),
            Some(branch_target)
        );
        assert_eq!(agent.messages().len(), 3);

        let action = handle_command("/resume", &mut agent, &mut runtime)
            .expect("resume command should succeed");
        assert_eq!(action, CommandAction::Continue);
        assert_eq!(
            runtime.as_ref().and_then(|runtime| runtime.active_head),
            Some(head)
        );
        assert_eq!(agent.messages().len(), 5);
    }

    #[test]
    fn exit_commands_return_exit_action() {
        let mut agent = Agent::new(Arc::new(NoopClient), AgentConfig::default());
        let mut runtime = None;

        assert_eq!(
            handle_command("/quit", &mut agent, &mut runtime).expect("quit should succeed"),
            CommandAction::Exit
        );
        assert_eq!(
            handle_command("/exit", &mut agent, &mut runtime).expect("exit should succeed"),
            CommandAction::Exit
        );
    }
}
