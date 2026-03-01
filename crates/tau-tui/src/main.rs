use std::{
    collections::VecDeque,
    env, fs,
    io::{IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        block::BorderType, Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Terminal,
};

use tau_tui::{
    apply_overlay, render_operator_shell_frame, Component, DiffRenderer, EditorBuffer, EditorView,
    LumaImage, OperatorShellFrame, Text, Theme, ThemeRole,
};

const DEFAULT_GATEWAY_SESSION_KEY: &str = "default";

const HELP: &str = "\
tau-tui operator terminal

Usage:
  cargo run -p tau-tui -- [demo] [--frames N] [--width N] [--sleep-ms N] [--no-color]
  cargo run -p tau-tui -- shell [--width N] [--profile NAME] [--no-color]
  cargo run -p tau-tui -- shell-live [--state-dir PATH] [--width N] [--profile NAME] [--watch] [--iterations N] [--interval-ms N] [--no-color]
  cargo run -p tau-tui -- agent [--dashboard-state-dir PATH] [--gateway-state-dir PATH] [--model ID] [--max-turns N] [--request-timeout-ms N] [--agent-request-max-retries N] [--codex-reasoning-effort LEVEL] [--interactive-timeline-verbose] [--mcp-client] [--mcp-external-server-config PATH] [--memory-state-dir PATH] [--skills-dir PATH] [--skill NAME]... [--prompt-optimization-config PATH] [--prompt-optimization-store-sqlite PATH] [--prompt-optimization-json] [--agent-arg ARG]... [--agent-binary PATH] [--width N] [--profile NAME] [--passthrough] [--ratatui|--no-ratatui] [--dry-run] [--no-color]

Options:
  demo          Animated rendering demo mode (default command)
  shell         Operator shell mode with status/auth/training panels
  shell-live    State-backed operator shell mode from dashboard artifacts
  agent         Operator shell mode that launches interactive tau-coding-agent runtime
  --frames N    Demo: number of frames to render (default: 3, min: 1)
  --width N     Demo/Shell: render width in characters (demo default: 72, shell default: 88)
  --sleep-ms N  Demo: delay between frames in milliseconds (default: 120)
  --profile N   Shell: operator profile label (default: local-dev)
  --state-dir P Shell-live: dashboard state directory (default: .tau/dashboard)
  --dashboard-state-dir P Agent: dashboard state directory (default: .tau/dashboard)
  --gateway-state-dir P Agent: gateway state directory (default: .tau/gateway)
  --model ID    Agent: model id for interactive runtime (default: openai/gpt-5.2)
  --max-turns N Agent: max model/tool turns allowed for a single prompt (min: 1)
  --request-timeout-ms N Agent: request timeout in milliseconds forwarded to tau-coding-agent
  --agent-request-max-retries N Agent: max model request retries forwarded to tau-coding-agent
  --codex-reasoning-effort LEVEL Agent: codex reasoning effort override (minimal|low|medium|high|xhigh)
  --interactive-timeline-verbose Agent: disable compact rolling timeline and print all timeline updates
  --mcp-client Agent: enable MCP client tool discovery/registration for interactive runtime
  --mcp-external-server-config P Agent: MCP external server/client config path
  --memory-state-dir P Agent: memory state directory override
  --skills-dir P Agent: skills directory override
  --skill NAME Agent: selected skill id/name (repeatable)
  --prompt-optimization-config P Agent: RL/prompt-optimization rollout config path
  --prompt-optimization-store-sqlite P Agent: RL/prompt-optimization sqlite state path
  --prompt-optimization-json Agent: emit RL/prompt-optimization summaries as JSON
  --agent mode defaults to gateway/webchat session bridging unless --session/--no-session is provided
  --agent-arg ARG Agent: additional raw tau-coding-agent argument token (repeatable)
  --agent-binary PATH Agent: launch tau-coding-agent from this binary path (fallback: cargo run)
  --passthrough Agent: disable app-style panel renderer and stream raw runtime output
  --ratatui    Agent: force ratatui pane renderer (default)
  --no-ratatui Agent: force legacy text renderer
  --dry-run     Agent: print interactive launch command without executing it
  --watch       Shell-live: enable watch mode across multiple refresh cycles
  --iterations N Shell-live watch: number of render cycles (default: 3, min: 1)
  --interval-ms N Shell-live watch: delay between cycles in milliseconds (default: 1000)
  --no-color    Disable ANSI color output for CI/smoke runs
  --help, -h    Show this help message
";

#[derive(Debug, Clone, PartialEq, Eq)]
struct DemoArgs {
    frames: usize,
    width: usize,
    sleep_ms: u64,
    color: bool,
}

impl Default for DemoArgs {
    fn default() -> Self {
        Self {
            frames: 3,
            width: 72,
            sleep_ms: 120,
            color: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellArgs {
    width: usize,
    profile: String,
    color: bool,
}

impl Default for ShellArgs {
    fn default() -> Self {
        Self {
            width: 88,
            profile: "local-dev".to_string(),
            color: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiveShellArgs {
    width: usize,
    profile: String,
    state_dir: String,
    watch: bool,
    iterations: usize,
    interval_ms: u64,
    color: bool,
}

impl Default for LiveShellArgs {
    fn default() -> Self {
        Self {
            width: 88,
            profile: "local-dev".to_string(),
            state_dir: ".tau/dashboard".to_string(),
            watch: false,
            iterations: 3,
            interval_ms: 1000,
            color: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentArgs {
    width: usize,
    profile: String,
    dashboard_state_dir: String,
    gateway_state_dir: String,
    model: String,
    max_turns: Option<usize>,
    request_timeout_ms: Option<u64>,
    agent_request_max_retries: Option<usize>,
    codex_reasoning_effort: Option<String>,
    interactive_timeline_verbose: bool,
    mcp_client: bool,
    mcp_external_server_config: Option<String>,
    memory_state_dir: Option<String>,
    skills_dir: Option<String>,
    skills: Vec<String>,
    prompt_optimization_config: Option<String>,
    prompt_optimization_store_sqlite: Option<String>,
    prompt_optimization_json: bool,
    agent_args: Vec<String>,
    agent_binary: Option<String>,
    passthrough: bool,
    ratatui: bool,
    dry_run: bool,
    color: bool,
}

impl Default for AgentArgs {
    fn default() -> Self {
        Self {
            width: 88,
            profile: "local-dev".to_string(),
            dashboard_state_dir: ".tau/dashboard".to_string(),
            gateway_state_dir: ".tau/gateway".to_string(),
            model: "openai/gpt-5.2".to_string(),
            max_turns: None,
            request_timeout_ms: None,
            agent_request_max_retries: None,
            codex_reasoning_effort: None,
            interactive_timeline_verbose: false,
            mcp_client: false,
            mcp_external_server_config: None,
            memory_state_dir: None,
            skills_dir: None,
            skills: Vec::new(),
            prompt_optimization_config: None,
            prompt_optimization_store_sqlite: None,
            prompt_optimization_json: false,
            agent_args: Vec::new(),
            agent_binary: None,
            passthrough: false,
            ratatui: true,
            dry_run: false,
            color: true,
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum ParseAction {
    RunDemo(DemoArgs),
    RunShell(ShellArgs),
    RunShellLive(LiveShellArgs),
    RunAgent(AgentArgs),
    Help,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<ParseAction, String> {
    let mut values = args.into_iter();
    let _ = values.next();
    let mut values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(ParseAction::RunDemo(DemoArgs::default()));
    }

    match values.first().map(String::as_str) {
        Some("--help") | Some("-h") => Ok(ParseAction::Help),
        Some("demo") => {
            values.remove(0);
            parse_demo_args(values)
        }
        Some("shell") => {
            values.remove(0);
            parse_shell_args(values)
        }
        Some("shell-live") => {
            values.remove(0);
            parse_shell_live_args(values)
        }
        Some("agent") => {
            values.remove(0);
            parse_agent_args(values)
        }
        _ => parse_demo_args(values),
    }
}

fn parse_demo_args(args: Vec<String>) -> Result<ParseAction, String> {
    let mut parsed = DemoArgs::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(ParseAction::Help),
            "--no-color" => parsed.color = false,
            "--frames" => {
                let raw = it.next().ok_or("missing value for --frames")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --frames: {raw}"))?;
                if value == 0 {
                    return Err("--frames must be >= 1".to_string());
                }
                parsed.frames = value;
            }
            "--width" => {
                let raw = it.next().ok_or("missing value for --width")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --width: {raw}"))?;
                if value < 20 {
                    return Err("--width must be >= 20".to_string());
                }
                parsed.width = value;
            }
            "--sleep-ms" => {
                let raw = it.next().ok_or("missing value for --sleep-ms")?;
                let value = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid u64 for --sleep-ms: {raw}"))?;
                parsed.sleep_ms = value;
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(ParseAction::RunDemo(parsed))
}

fn parse_shell_args(args: Vec<String>) -> Result<ParseAction, String> {
    let mut parsed = ShellArgs::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(ParseAction::Help),
            "--no-color" => parsed.color = false,
            "--width" => {
                let raw = it.next().ok_or("missing value for --width")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --width: {raw}"))?;
                if value < 40 {
                    return Err("--width must be >= 40".to_string());
                }
                parsed.width = value;
            }
            "--profile" => {
                let raw = it.next().ok_or("missing value for --profile")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--profile must not be empty".to_string());
                }
                parsed.profile = value.to_string();
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(ParseAction::RunShell(parsed))
}

fn parse_shell_live_args(args: Vec<String>) -> Result<ParseAction, String> {
    let mut parsed = LiveShellArgs::default();
    let mut saw_iterations = false;
    let mut saw_interval = false;
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(ParseAction::Help),
            "--no-color" => parsed.color = false,
            "--watch" => parsed.watch = true,
            "--width" => {
                let raw = it.next().ok_or("missing value for --width")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --width: {raw}"))?;
                if value < 40 {
                    return Err("--width must be >= 40".to_string());
                }
                parsed.width = value;
            }
            "--profile" => {
                let raw = it.next().ok_or("missing value for --profile")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--profile must not be empty".to_string());
                }
                parsed.profile = value.to_string();
            }
            "--state-dir" => {
                let raw = it.next().ok_or("missing value for --state-dir")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--state-dir must not be empty".to_string());
                }
                parsed.state_dir = value.to_string();
            }
            "--iterations" => {
                let raw = it.next().ok_or("missing value for --iterations")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --iterations: {raw}"))?;
                if value == 0 {
                    return Err("--iterations must be >= 1".to_string());
                }
                parsed.iterations = value;
                saw_iterations = true;
            }
            "--interval-ms" => {
                let raw = it.next().ok_or("missing value for --interval-ms")?;
                let value = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid u64 for --interval-ms: {raw}"))?;
                parsed.interval_ms = value;
                saw_interval = true;
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    if !parsed.watch && (saw_iterations || saw_interval) {
        return Err("--iterations/--interval-ms require --watch".to_string());
    }

    Ok(ParseAction::RunShellLive(parsed))
}

fn parse_agent_args(args: Vec<String>) -> Result<ParseAction, String> {
    let mut parsed = AgentArgs::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(ParseAction::Help),
            "--no-color" => parsed.color = false,
            "--dry-run" => parsed.dry_run = true,
            "--passthrough" => parsed.passthrough = true,
            "--ratatui" => parsed.ratatui = true,
            "--no-ratatui" => parsed.ratatui = false,
            "--width" => {
                let raw = it.next().ok_or("missing value for --width")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --width: {raw}"))?;
                if value < 40 {
                    return Err("--width must be >= 40".to_string());
                }
                parsed.width = value;
            }
            "--profile" => {
                let raw = it.next().ok_or("missing value for --profile")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--profile must not be empty".to_string());
                }
                parsed.profile = value.to_string();
            }
            "--state-dir" | "--dashboard-state-dir" => {
                let raw = it.next().ok_or("missing value for --dashboard-state-dir")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--dashboard-state-dir must not be empty".to_string());
                }
                parsed.dashboard_state_dir = value.to_string();
            }
            "--gateway-state-dir" => {
                let raw = it.next().ok_or("missing value for --gateway-state-dir")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--gateway-state-dir must not be empty".to_string());
                }
                parsed.gateway_state_dir = value.to_string();
            }
            "--model" => {
                let raw = it.next().ok_or("missing value for --model")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--model must not be empty".to_string());
                }
                parsed.model = value.to_string();
            }
            "--max-turns" => {
                let raw = it.next().ok_or("missing value for --max-turns")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --max-turns: {raw}"))?;
                if value == 0 {
                    return Err("--max-turns must be >= 1".to_string());
                }
                parsed.max_turns = Some(value);
            }
            "--request-timeout-ms" => {
                let raw = it.next().ok_or("missing value for --request-timeout-ms")?;
                let value = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid u64 for --request-timeout-ms: {raw}"))?;
                if value == 0 {
                    return Err("--request-timeout-ms must be >= 1".to_string());
                }
                parsed.request_timeout_ms = Some(value);
            }
            "--agent-request-max-retries" => {
                let raw = it
                    .next()
                    .ok_or("missing value for --agent-request-max-retries")?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid usize for --agent-request-max-retries: {raw}"))?;
                parsed.agent_request_max_retries = Some(value);
            }
            "--codex-reasoning-effort" => {
                let raw = it
                    .next()
                    .ok_or("missing value for --codex-reasoning-effort")?;
                let value = raw.trim().to_ascii_lowercase();
                match value.as_str() {
                    "minimal" | "low" | "medium" | "high" | "xhigh" => {
                        parsed.codex_reasoning_effort = Some(value);
                    }
                    _ => {
                        return Err(format!(
                            "invalid value for --codex-reasoning-effort: {raw} (expected minimal|low|medium|high|xhigh)"
                        ));
                    }
                }
            }
            "--interactive-timeline-verbose" => {
                parsed.interactive_timeline_verbose = true;
            }
            "--mcp-client" => {
                parsed.mcp_client = true;
            }
            "--mcp-external-server-config" => {
                let raw = it
                    .next()
                    .ok_or("missing value for --mcp-external-server-config")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--mcp-external-server-config must not be empty".to_string());
                }
                parsed.mcp_external_server_config = Some(value.to_string());
            }
            "--memory-state-dir" => {
                let raw = it.next().ok_or("missing value for --memory-state-dir")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--memory-state-dir must not be empty".to_string());
                }
                parsed.memory_state_dir = Some(value.to_string());
            }
            "--skills-dir" => {
                let raw = it.next().ok_or("missing value for --skills-dir")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--skills-dir must not be empty".to_string());
                }
                parsed.skills_dir = Some(value.to_string());
            }
            "--skill" => {
                let raw = it.next().ok_or("missing value for --skill")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--skill must not be empty".to_string());
                }
                parsed.skills.push(value.to_string());
            }
            "--prompt-optimization-config" => {
                let raw = it
                    .next()
                    .ok_or("missing value for --prompt-optimization-config")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--prompt-optimization-config must not be empty".to_string());
                }
                parsed.prompt_optimization_config = Some(value.to_string());
            }
            "--prompt-optimization-store-sqlite" => {
                let raw = it
                    .next()
                    .ok_or("missing value for --prompt-optimization-store-sqlite")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--prompt-optimization-store-sqlite must not be empty".to_string());
                }
                parsed.prompt_optimization_store_sqlite = Some(value.to_string());
            }
            "--prompt-optimization-json" => {
                parsed.prompt_optimization_json = true;
            }
            "--agent-arg" => {
                let raw = it.next().ok_or("missing value for --agent-arg")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--agent-arg must not be empty".to_string());
                }
                parsed.agent_args.push(value.to_string());
            }
            "--agent-binary" => {
                let raw = it.next().ok_or("missing value for --agent-binary")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--agent-binary must not be empty".to_string());
                }
                parsed.agent_binary = Some(value.to_string());
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(ParseAction::RunAgent(parsed))
}

fn paint(theme: &Theme, role: ThemeRole, text: impl Into<String>, color: bool) -> String {
    let text = text.into();
    if color {
        theme.paint(role, &text)
    } else {
        text
    }
}

fn compose_frame(
    buffer: &EditorBuffer,
    image: &LumaImage,
    args: &DemoArgs,
    frame: usize,
) -> Vec<String> {
    let viewport_top = buffer.lines().len().saturating_sub(6);
    let editor_lines = EditorView::new(buffer)
        .with_viewport(viewport_top, 6)
        .with_line_numbers(true)
        .with_cursor(true)
        .render(args.width);

    let mut base = Text::new("live editor view").render(args.width);
    base.extend(editor_lines);
    base.push(String::new());
    base.push("ascii preview".to_string());
    base.extend(image.render_fit(args.width.min(24)));

    let overlay = vec![format!(
        "frame={}/{} width={} sleep_ms={}",
        frame + 1,
        args.frames,
        args.width,
        args.sleep_ms
    )];
    apply_overlay(&base, &overlay, 0, 0)
}

fn advance_buffer(buffer: &mut EditorBuffer, frame: usize) {
    if frame == 0 {
        buffer.insert_text("fn tau_demo_loop(frame: usize) {\n    let status = \"ready\";\n");
        buffer.insert_text("    println!(\"frame={frame} status={status}\");\n}");
        return;
    }
    buffer.insert_newline();
    buffer.insert_text(&format!(
        "// frame {} checkpoint: render diff + overlay",
        frame + 1
    ));
}

fn run_demo(args: DemoArgs) -> Result<(), String> {
    let theme = Theme::default();
    let image = LumaImage::from_luma(
        8,
        4,
        vec![
            0, 24, 56, 88, 120, 152, 184, 216, 16, 40, 72, 104, 136, 168, 200, 232, 32, 64, 96,
            128, 160, 192, 224, 255, 24, 56, 88, 120, 152, 184, 216, 248,
        ],
    )
    .map_err(|err| format!("failed to construct demo image: {err}"))?;
    let mut buffer = EditorBuffer::new();
    let mut diff = DiffRenderer::new();

    for frame in 0..args.frames {
        advance_buffer(&mut buffer, frame);
        let rendered = compose_frame(&buffer, &image, &args, frame);
        let operations = diff.diff(rendered.clone());

        let header = paint(
            &theme,
            ThemeRole::Accent,
            format!(
                "Tau TUI Demo - frame {}/{} (ops={})",
                frame + 1,
                args.frames,
                operations.len()
            ),
            args.color,
        );
        println!("{header}");
        for operation in operations {
            let line = paint(
                &theme,
                ThemeRole::Muted,
                format!("op:{operation}"),
                args.color,
            );
            println!("{line}");
        }
        for line in rendered {
            println!("{}", paint(&theme, ThemeRole::Primary, line, args.color));
        }
        println!();

        if frame + 1 < args.frames && args.sleep_ms > 0 {
            thread::sleep(Duration::from_millis(args.sleep_ms));
        }
    }
    Ok(())
}

fn run_shell(args: ShellArgs) {
    let theme = Theme::default();
    let frame = OperatorShellFrame::deterministic_fixture(args.profile.clone());
    let rendered = render_operator_shell_frame(&frame, args.width);
    let header = paint(
        &theme,
        ThemeRole::Accent,
        format!(
            "Tau Operator Shell - profile={} env={}",
            frame.profile, frame.environment
        ),
        args.color,
    );
    println!("{header}");
    for line in rendered {
        println!("{}", paint(&theme, ThemeRole::Primary, line, args.color));
    }
}

fn format_live_watch_marker(
    cycle: usize,
    total: usize,
    interval_ms: u64,
    diff_ops: usize,
) -> String {
    format!("watch.cycle={cycle}/{total} watch.interval_ms={interval_ms} watch.diff_ops={diff_ops}")
}

fn run_shell_live(args: LiveShellArgs) {
    let theme = Theme::default();
    let cycles = if args.watch { args.iterations } else { 1 };
    let mut diff = DiffRenderer::new();
    for cycle in 0..cycles {
        let frame = OperatorShellFrame::from_dashboard_state_dir(
            args.profile.clone(),
            Path::new(args.state_dir.as_str()),
        );
        let rendered = render_operator_shell_frame(&frame, args.width);
        let diff_ops = if args.watch {
            diff.diff(rendered.clone()).len()
        } else {
            rendered.len()
        };
        let header = if args.watch {
            paint(
                &theme,
                ThemeRole::Accent,
                format!(
                    "Tau Operator Shell (live-watch) - profile={} env={} state_dir={} cycle={}/{}",
                    frame.profile,
                    frame.environment,
                    args.state_dir,
                    cycle + 1,
                    cycles
                ),
                args.color,
            )
        } else {
            paint(
                &theme,
                ThemeRole::Accent,
                format!(
                    "Tau Operator Shell (live) - profile={} env={} state_dir={}",
                    frame.profile, frame.environment, args.state_dir
                ),
                args.color,
            )
        };
        println!("{header}");
        if args.watch {
            println!(
                "{}",
                paint(
                    &theme,
                    ThemeRole::Muted,
                    format_live_watch_marker(cycle + 1, cycles, args.interval_ms, diff_ops),
                    args.color
                )
            );
        }
        for line in rendered {
            println!("{}", paint(&theme, ThemeRole::Primary, line, args.color));
        }
        if args.watch && cycle + 1 < cycles && args.interval_ms > 0 {
            thread::sleep(Duration::from_millis(args.interval_ms));
        }
    }
}

fn build_agent_runtime_command(args: &AgentArgs) -> Vec<String> {
    let program = args.agent_binary.clone().or_else(|| {
        let env_binary = env::var("TAU_TUI_AGENT_BINARY").ok();
        let env_binary = env_binary
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if env_binary.is_some() {
            return env_binary;
        }
        let default_binary = Path::new("target/debug/tau-coding-agent");
        if default_binary.exists() {
            Some(default_binary.display().to_string())
        } else {
            None
        }
    });

    let mut command = if let Some(program) = program {
        vec![program]
    } else {
        vec![
            "cargo".to_string(),
            "run".to_string(),
            "-p".to_string(),
            "tau-coding-agent".to_string(),
            "--".to_string(),
        ]
    };
    command.extend_from_slice(&[
        "--model".to_string(),
        args.model.clone(),
        "--gateway-state-dir".to_string(),
        args.gateway_state_dir.clone(),
        "--dashboard-state-dir".to_string(),
        args.dashboard_state_dir.clone(),
        "--interactive-launch-surface".to_string(),
        "tui".to_string(),
    ]);
    if let Some(max_turns) = args.max_turns {
        command.push("--max-turns".to_string());
        command.push(max_turns.to_string());
    }
    if let Some(timeout_ms) = args.request_timeout_ms {
        command.push("--request-timeout-ms".to_string());
        command.push(timeout_ms.to_string());
    }
    if let Some(max_retries) = args.agent_request_max_retries {
        command.push("--agent-request-max-retries".to_string());
        command.push(max_retries.to_string());
    }
    if let Some(reasoning_effort) = args.codex_reasoning_effort.as_deref() {
        command.push(format!(
            "--openai-codex-args=-c,model_reasoning_effort=\"{reasoning_effort}\""
        ));
    }
    if args.interactive_timeline_verbose {
        command.push("--interactive-timeline-verbose".to_string());
    }
    if args.mcp_client {
        command.push("--mcp-client".to_string());
    }
    if let Some(config_path) = args.mcp_external_server_config.as_deref() {
        command.push("--mcp-external-server-config".to_string());
        command.push(config_path.to_string());
    }
    if let Some(memory_state_dir) = args.memory_state_dir.as_deref() {
        command.push("--memory-state-dir".to_string());
        command.push(memory_state_dir.to_string());
    }
    if let Some(skills_dir) = args.skills_dir.as_deref() {
        command.push("--skills-dir".to_string());
        command.push(skills_dir.to_string());
    }
    for skill in &args.skills {
        command.push("--skill".to_string());
        command.push(skill.clone());
    }
    if let Some(rl_config) = args.prompt_optimization_config.as_deref() {
        command.push("--prompt-optimization-config".to_string());
        command.push(rl_config.to_string());
    }
    if let Some(rl_store_sqlite) = args.prompt_optimization_store_sqlite.as_deref() {
        command.push("--prompt-optimization-store-sqlite".to_string());
        command.push(rl_store_sqlite.to_string());
    }
    if args.prompt_optimization_json {
        command.push("--prompt-optimization-json".to_string());
    }
    if let Some(gateway_session_path) = resolve_gateway_session_bridge_path(args) {
        command.push("--session".to_string());
        command.push(gateway_session_path);
    }
    command.extend(args.agent_args.clone());
    command
}

fn resolve_gateway_session_bridge_path(args: &AgentArgs) -> Option<String> {
    if agent_runtime_has_explicit_session_override(&args.agent_args)
        || env::var("TAU_SESSION")
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
    {
        return None;
    }
    let session_key = env::var("TAU_TUI_GATEWAY_SESSION_KEY")
        .ok()
        .map(|value| sanitize_session_key(value.as_str()))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_GATEWAY_SESSION_KEY.to_string());
    let gateway_root = args.gateway_state_dir.trim_end_matches('/');
    if gateway_root.is_empty() {
        return None;
    }
    Some(format!(
        "{gateway_root}/openresponses/sessions/{session_key}.jsonl"
    ))
}

fn agent_runtime_has_explicit_session_override(agent_args: &[String]) -> bool {
    let mut expect_session_value = false;
    for token in agent_args {
        let value = token.trim();
        if value.is_empty() {
            continue;
        }
        if expect_session_value {
            return true;
        }
        if value == "--session" {
            expect_session_value = true;
            continue;
        }
        if value == "--no-session"
            || value.starts_with("--session=")
            || value.starts_with("--no-session=")
        {
            return true;
        }
    }
    false
}

fn sanitize_session_key(raw: &str) -> String {
    let mut normalized = String::new();
    for ch in raw.chars() {
        let keep = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.';
        normalized.push(if keep { ch } else { '-' });
    }
    let trimmed = normalized.trim_matches('-').trim();
    if trimmed.is_empty() {
        DEFAULT_GATEWAY_SESSION_KEY.to_string()
    } else {
        trimmed.to_string()
    }
}

fn format_shell_command(tokens: &[String]) -> String {
    tokens.join(" ")
}

fn format_interactive_controls_marker() -> &'static str {
    "controls: Ctrl+C cancel turn | /quit exit"
}

const AGENT_PANEL_HISTORY_LIMIT: usize = 200;
const AGENT_PANEL_SCROLL_STEP: usize = 4;
const AGENT_PANEL_PAGE_SCROLL_STEP: usize = 12;
const AGENT_PANEL_SCROLL_FOLLOW_TAIL: usize = 0;
const AGENT_PANEL_SCROLL_TO_HEAD: usize = usize::MAX / 4;
const AGENT_LOCAL_QUIT_GRACE_MS: u64 = 1500;
const GATEWAY_SYNC_INTERVAL_MS: u64 = 10000;
const GATEWAY_SYNC_BACKOFF_INTERVAL_MS: u64 = 30000;
const GATEWAY_SYNC_BACKOFF_MAX_INTERVAL_MS: u64 = 120000;
const GATEWAY_SYNC_FULL_REFRESH_EVERY_LIGHT_CYCLES: u64 = 6;
const GATEWAY_SYNC_CURL_TIMEOUT_SECONDS: u64 = 2;
const DEFAULT_TUI_GATEWAY_BASE_URL: &str = "http://127.0.0.1:8791";
const DEFAULT_TUI_PREFS_PATH: &str = ".tau/tui-state.json";
const AGENT_MEMORY_ACTIVITY_LIMIT: usize = 24;
const TUI_PREFS_VERSION: u64 = 2;
const LEGACY_DEFAULT_SPLIT_LEFT_PERCENT: u8 = 72;
const LEGACY_DEFAULT_SPLIT_TOP_PERCENT: u8 = 68;
const DEFAULT_SPLIT_LEFT_PERCENT: u8 = 66;
const DEFAULT_SPLIT_TOP_PERCENT: u8 = 72;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentOutputSource {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgentRuntimeEvent {
    Output(AgentOutputSource, String),
    InputByte(u8),
    GatewaySync(GatewaySyncSnapshot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatewaySyncCommand {
    RefreshNow,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GatewaySyncSnapshot {
    status_line: String,
    dashboard_lines: Vec<String>,
    tools_lines: Vec<String>,
    routines_lines: Vec<String>,
    cortex_lines: Vec<String>,
    memory_lines: Vec<String>,
    event_line: String,
    success: bool,
    rate_limited: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatewaySyncFetchMode {
    Light,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalTuiCommand {
    Help,
    Status,
    Dashboard,
    Tools,
    Routines,
    Cortex,
    Memory,
    Sync,
    Colors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentPanel {
    Session,
    Turn,
    Assistant,
    Timeline,
    Tools,
    Events,
}

impl AgentPanel {
    const ORDER: [Self; 6] = [
        Self::Session,
        Self::Turn,
        Self::Assistant,
        Self::Timeline,
        Self::Tools,
        Self::Events,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Session => "Session",
            Self::Turn => "Turn",
            Self::Assistant => "Assistant",
            Self::Timeline => "Timeline",
            Self::Tools => "Tools",
            Self::Events => "Events",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Session => 0,
            Self::Turn => 1,
            Self::Assistant => 2,
            Self::Timeline => 3,
            Self::Tools => 4,
            Self::Events => 5,
        }
    }

    fn as_prefs_key(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Turn => "turn",
            Self::Assistant => "assistant",
            Self::Timeline => "timeline",
            Self::Tools => "tools",
            Self::Events => "events",
        }
    }

    fn from_prefs_key(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "session" => Some(Self::Session),
            "turn" => Some(Self::Turn),
            "assistant" => Some(Self::Assistant),
            "timeline" => Some(Self::Timeline),
            "tools" => Some(Self::Tools),
            "events" => Some(Self::Events),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TurnPhase {
    Idle,
    Queued,
    Model,
    Tool,
    PostTool,
    Done,
    Failed,
    Cancelled,
}

impl TurnPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Queued => "queued",
            Self::Model => "model",
            Self::Tool => "tool",
            Self::PostTool => "post-tool",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToneMode {
    Semantic,
    Minimal,
}

impl ToneMode {
    fn label(self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::Minimal => "minimal",
        }
    }

    fn legend(self) -> &'static str {
        match self {
            Self::Semantic => "green=ok yellow=warn red=bad",
            Self::Minimal => "minimal palette (semantic highlights off)",
        }
    }

    fn from_label(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "semantic" => Some(Self::Semantic),
            "minimal" => Some(Self::Minimal),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct AgentAppState {
    turn_status: String,
    progress_status: String,
    turn_phase: TurnPhase,
    turn_in_progress: bool,
    spinner_phase: usize,
    heartbeat_phase: usize,
    turn_started_at: Option<Instant>,
    assistant_output_seen_in_turn: bool,
    last_submitted_prompt: String,
    runtime_event_count: u64,
    submitted_turn_count: u64,
    last_runtime_event_at: Option<Instant>,
    runtime_event_count_at_turn_start: u64,
    default_request_budget_ms: Option<u64>,
    turn_request_budget_ms: Option<u64>,
    active_tool_name: Option<String>,
    gateway_sync_ok: bool,
    gateway_sync_status: String,
    gateway_sync_last_at: Option<Instant>,
    dashboard_lines: Vec<String>,
    integration_tools_lines: Vec<String>,
    integration_routines_lines: Vec<String>,
    integration_cortex_lines: Vec<String>,
    integration_memory_lines: Vec<String>,
    timeline_lines: VecDeque<String>,
    assistant_lines: VecDeque<String>,
    tool_lines: VecDeque<String>,
    event_lines: VecDeque<String>,
    memory_activity_lines: VecDeque<String>,
    last_memory_activity_summary: String,
    prompt_line: String,
    auth_hint_emitted: bool,
    focused_panel: AgentPanel,
    expanded_panel: Option<AgentPanel>,
    panel_scroll_offsets: [usize; AgentPanel::ORDER.len()],
    show_shortcuts: bool,
    split_left_percent: u8,
    split_top_percent: u8,
    tone_mode: ToneMode,
    ui_prefs_dirty: bool,
}

impl Default for AgentAppState {
    fn default() -> Self {
        Self {
            turn_status: String::new(),
            progress_status: String::new(),
            turn_phase: TurnPhase::Idle,
            turn_in_progress: false,
            spinner_phase: 0,
            heartbeat_phase: 0,
            turn_started_at: None,
            assistant_output_seen_in_turn: false,
            last_submitted_prompt: String::new(),
            runtime_event_count: 0,
            submitted_turn_count: 0,
            last_runtime_event_at: None,
            runtime_event_count_at_turn_start: 0,
            default_request_budget_ms: None,
            turn_request_budget_ms: None,
            active_tool_name: None,
            gateway_sync_ok: false,
            gateway_sync_status: "gateway sync: pending".to_string(),
            gateway_sync_last_at: None,
            dashboard_lines: vec!["dashboard sync pending (/sync)".to_string()],
            integration_tools_lines: vec!["tools sync pending (/sync)".to_string()],
            integration_routines_lines: vec!["routines sync pending (/sync)".to_string()],
            integration_cortex_lines: vec!["cortex sync pending (/sync)".to_string()],
            integration_memory_lines: vec!["memory sync pending (/sync)".to_string()],
            timeline_lines: VecDeque::new(),
            assistant_lines: VecDeque::new(),
            tool_lines: VecDeque::new(),
            event_lines: VecDeque::new(),
            memory_activity_lines: VecDeque::new(),
            last_memory_activity_summary: String::new(),
            prompt_line: String::new(),
            auth_hint_emitted: false,
            focused_panel: AgentPanel::Assistant,
            expanded_panel: None,
            panel_scroll_offsets: [AGENT_PANEL_SCROLL_FOLLOW_TAIL; AgentPanel::ORDER.len()],
            show_shortcuts: false,
            split_left_percent: DEFAULT_SPLIT_LEFT_PERCENT,
            split_top_percent: DEFAULT_SPLIT_TOP_PERCENT,
            tone_mode: ToneMode::Semantic,
            ui_prefs_dirty: false,
        }
    }
}

impl AgentAppState {
    fn panel_title(&self, panel: AgentPanel) -> String {
        panel.title().to_string()
    }

    fn panel_is_focused(&self, panel: AgentPanel) -> bool {
        self.focused_panel == panel
    }

    fn panel_is_expanded(&self, panel: AgentPanel) -> bool {
        self.expanded_panel == Some(panel)
    }

    fn focus_next_panel(&mut self) {
        let current_index = self.focused_panel.index();
        let next_index = (current_index + 1) % AgentPanel::ORDER.len();
        self.focused_panel = AgentPanel::ORDER[next_index];
        self.ui_prefs_dirty = true;
    }

    fn focus_previous_panel(&mut self) {
        let current_index = self.focused_panel.index();
        let next_index = if current_index == 0 {
            AgentPanel::ORDER.len() - 1
        } else {
            current_index - 1
        };
        self.focused_panel = AgentPanel::ORDER[next_index];
        self.ui_prefs_dirty = true;
    }

    fn toggle_expand_focused_panel(&mut self) {
        if matches!(self.focused_panel, AgentPanel::Session | AgentPanel::Turn) {
            return;
        }
        self.expanded_panel = if self.expanded_panel == Some(self.focused_panel) {
            None
        } else {
            Some(self.focused_panel)
        };
        self.ui_prefs_dirty = true;
    }

    fn panel_offset(&self, panel: AgentPanel) -> usize {
        self.panel_scroll_offsets[panel.index()]
    }

    fn set_panel_offset(&mut self, panel: AgentPanel, offset: usize) {
        self.panel_scroll_offsets[panel.index()] = offset;
        self.ui_prefs_dirty = true;
    }

    fn scroll_focused_panel_up(&mut self) {
        let panel = self.focused_panel;
        let current = self.panel_offset(panel);
        let next = current.saturating_add(AGENT_PANEL_SCROLL_STEP);
        self.set_panel_offset(panel, next);
    }

    fn scroll_focused_panel_down(&mut self) {
        let panel = self.focused_panel;
        let current = self.panel_offset(panel);
        self.set_panel_offset(panel, current.saturating_sub(AGENT_PANEL_SCROLL_STEP));
    }

    fn page_scroll_focused_panel_up(&mut self) {
        let panel = self.focused_panel;
        let current = self.panel_offset(panel);
        let next = current.saturating_add(AGENT_PANEL_PAGE_SCROLL_STEP);
        self.set_panel_offset(panel, next);
    }

    fn page_scroll_focused_panel_down(&mut self) {
        let panel = self.focused_panel;
        let current = self.panel_offset(panel);
        self.set_panel_offset(panel, current.saturating_sub(AGENT_PANEL_PAGE_SCROLL_STEP));
    }

    fn scroll_focused_panel_to_oldest(&mut self) {
        let panel = self.focused_panel;
        self.set_panel_offset(panel, AGENT_PANEL_SCROLL_TO_HEAD);
    }

    fn scroll_focused_panel_to_latest(&mut self) {
        let panel = self.focused_panel;
        self.set_panel_offset(panel, AGENT_PANEL_SCROLL_FOLLOW_TAIL);
    }

    fn adjust_split_left_percent(&mut self, delta: i8) {
        let next = (self.split_left_percent as i16 + delta as i16).clamp(35, 80) as u8;
        self.split_left_percent = next;
        self.ui_prefs_dirty = true;
    }

    fn adjust_split_top_percent(&mut self, delta: i8) {
        let next = (self.split_top_percent as i16 + delta as i16).clamp(55, 85) as u8;
        self.split_top_percent = next;
        self.ui_prefs_dirty = true;
    }

    fn toggle_tone_mode(&mut self) {
        self.tone_mode = match self.tone_mode {
            ToneMode::Semantic => ToneMode::Minimal,
            ToneMode::Minimal => ToneMode::Semantic,
        };
        self.ui_prefs_dirty = true;
    }

    fn mark_turn_started(&mut self) {
        if !self.turn_in_progress {
            self.turn_started_at = Some(Instant::now());
            self.assistant_output_seen_in_turn = false;
            self.spinner_phase = 0;
            self.runtime_event_count_at_turn_start = self.runtime_event_count;
            self.active_tool_name = None;
            if self.turn_request_budget_ms.is_none() {
                self.turn_request_budget_ms = self.default_request_budget_ms;
            }
        }
        self.turn_in_progress = true;
        if matches!(
            self.turn_phase,
            TurnPhase::Idle | TurnPhase::Done | TurnPhase::Failed | TurnPhase::Cancelled
        ) {
            self.turn_phase = TurnPhase::Model;
        }
    }

    fn mark_turn_finished(&mut self) {
        self.turn_in_progress = false;
        self.turn_started_at = None;
        self.active_tool_name = None;
        self.turn_request_budget_ms = None;
        if matches!(
            self.turn_phase,
            TurnPhase::Queued | TurnPhase::Model | TurnPhase::Tool | TurnPhase::PostTool
        ) {
            self.turn_phase = TurnPhase::Done;
        }
    }

    fn mark_assistant_output_seen(&mut self) {
        if self.turn_in_progress {
            self.assistant_output_seen_in_turn = true;
        }
    }

    fn turn_elapsed(&self) -> Option<Duration> {
        self.turn_started_at.map(|started| started.elapsed())
    }

    fn last_runtime_event_age(&self) -> Option<Duration> {
        self.last_runtime_event_at.map(|last| last.elapsed())
    }

    fn gateway_sync_age(&self) -> Option<Duration> {
        self.gateway_sync_last_at.map(|last| last.elapsed())
    }

    fn note_runtime_event(&mut self) {
        self.runtime_event_count = self.runtime_event_count.saturating_add(1);
        self.last_runtime_event_at = Some(Instant::now());
    }

    fn note_turn_submitted(&mut self) {
        self.submitted_turn_count = self.submitted_turn_count.saturating_add(1);
    }

    fn push_memory_activity(&mut self, line: String) {
        if self
            .memory_activity_lines
            .back()
            .map(|existing| existing == &line)
            .unwrap_or(false)
        {
            return;
        }
        self.memory_activity_lines.push_back(line);
        while self.memory_activity_lines.len() > AGENT_MEMORY_ACTIVITY_LIMIT {
            self.memory_activity_lines.pop_front();
        }
    }

    fn runtime_events_seen_in_turn(&self) -> u64 {
        self.runtime_event_count
            .saturating_sub(self.runtime_event_count_at_turn_start)
    }

    fn tick_spinner(&mut self) {
        self.heartbeat_phase = (self.heartbeat_phase + 1) % 4;
        if self.turn_in_progress {
            self.spinner_phase = (self.spinner_phase + 1) % 4;
        }
    }

    fn spinner_glyph(&self) -> &'static str {
        match self.spinner_phase % 4 {
            0 => "|",
            1 => "/",
            2 => "-",
            _ => "\\",
        }
    }

    fn heartbeat_glyph(&self) -> &'static str {
        match self.heartbeat_phase % 4 {
            0 => ".",
            1 => "o",
            2 => "O",
            _ => "o",
        }
    }
}

fn push_history_line(history: &mut VecDeque<String>, line: String) {
    history.push_back(line);
    while history.len() > AGENT_PANEL_HISTORY_LIMIT {
        history.pop_front();
    }
}

fn strip_ansi_sequences(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn parse_stream_line_for_app(line: &str) -> String {
    strip_ansi_sequences(line).trim().to_string()
}

fn parse_legacy_kv(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=");
    line.split_whitespace()
        .find_map(|token| token.strip_prefix(&needle).map(str::to_string))
}

fn parse_ms_field(value: &str) -> Option<u64> {
    value
        .trim()
        .trim_end_matches("ms")
        .trim_end_matches([',', ';', ')'])
        .parse::<u64>()
        .ok()
}

fn extract_request_budget_ms_from_line(line: &str) -> Option<u64> {
    parse_legacy_kv(line, "request_budget")
        .and_then(|value| parse_ms_field(value.as_str()))
        .or_else(|| {
            parse_legacy_kv(line, "request_timeout_ms")
                .and_then(|value| parse_ms_field(value.as_str()))
        })
        .or_else(|| {
            parse_legacy_kv(line, "turn_timeout_ms")
                .and_then(|value| parse_ms_field(value.as_str()))
        })
}

fn resolve_tui_prefs_path() -> PathBuf {
    env::var("TAU_TUI_PREFS_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_TUI_PREFS_PATH))
}

fn load_tui_prefs(path: &Path, state: &mut AgentAppState) -> Result<(), String> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "unable to read tui prefs {}: {error}",
                path.display()
            ))
        }
    };
    let parsed = serde_json::from_str::<serde_json::Value>(raw.as_str())
        .map_err(|error| format!("invalid tui prefs json {}: {error}", path.display()))?;
    let object = parsed.as_object().ok_or_else(|| {
        format!(
            "invalid tui prefs payload at {}: expected object",
            path.display()
        )
    })?;

    let prefs_version = object
        .get("prefs_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    let mut loaded_split_left_percent: Option<u8> = None;
    let mut loaded_split_top_percent: Option<u8> = None;

    if let Some(value) = object
        .get("split_left_percent")
        .and_then(serde_json::Value::as_u64)
    {
        let loaded = value.clamp(35, 80) as u8;
        state.split_left_percent = loaded;
        loaded_split_left_percent = Some(loaded);
    }
    if let Some(value) = object
        .get("split_top_percent")
        .and_then(serde_json::Value::as_u64)
    {
        let loaded = value.clamp(55, 85) as u8;
        state.split_top_percent = loaded;
        loaded_split_top_percent = Some(loaded);
    }
    if let Some(value) = object
        .get("show_shortcuts")
        .and_then(serde_json::Value::as_bool)
    {
        state.show_shortcuts = value;
    }
    if let Some(value) = object
        .get("focused_panel")
        .and_then(serde_json::Value::as_str)
        .and_then(AgentPanel::from_prefs_key)
    {
        state.focused_panel = value;
    }
    if let Some(value) = object
        .get("expanded_panel")
        .and_then(serde_json::Value::as_str)
        .and_then(AgentPanel::from_prefs_key)
    {
        state.expanded_panel = Some(value);
    } else if object
        .get("expanded_panel")
        .map(|value| value.is_null())
        .unwrap_or(false)
    {
        state.expanded_panel = None;
    }
    if let Some(value) = object
        .get("tone_mode")
        .and_then(serde_json::Value::as_str)
        .and_then(ToneMode::from_label)
    {
        state.tone_mode = value;
    }
    if let Some(entries) = object
        .get("panel_scroll_offsets")
        .and_then(serde_json::Value::as_array)
    {
        for panel in AgentPanel::ORDER {
            if let Some(offset) = entries
                .get(panel.index())
                .and_then(serde_json::Value::as_u64)
            {
                state.panel_scroll_offsets[panel.index()] = offset as usize;
            }
        }
    }

    // One-time migration: preserve user custom splits, but upgrade legacy defaults
    // to the newer balanced layout when prefs were authored before versioning.
    let mut migrated = false;
    if prefs_version < TUI_PREFS_VERSION {
        if loaded_split_left_percent == Some(LEGACY_DEFAULT_SPLIT_LEFT_PERCENT) {
            state.split_left_percent = DEFAULT_SPLIT_LEFT_PERCENT;
            migrated = true;
        }
        if loaded_split_top_percent == Some(LEGACY_DEFAULT_SPLIT_TOP_PERCENT) {
            state.split_top_percent = DEFAULT_SPLIT_TOP_PERCENT;
            migrated = true;
        }
    }

    state.ui_prefs_dirty = migrated;
    Ok(())
}

fn persist_tui_prefs(path: &Path, state: &AgentAppState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "unable to create tui prefs directory {}: {error}",
                    parent.display()
                )
            })?;
        }
    }
    let mut object = serde_json::Map::new();
    object.insert(
        "prefs_version".to_string(),
        serde_json::Value::from(TUI_PREFS_VERSION),
    );
    object.insert(
        "split_left_percent".to_string(),
        serde_json::Value::from(state.split_left_percent as u64),
    );
    object.insert(
        "split_top_percent".to_string(),
        serde_json::Value::from(state.split_top_percent as u64),
    );
    object.insert(
        "show_shortcuts".to_string(),
        serde_json::Value::from(state.show_shortcuts),
    );
    object.insert(
        "focused_panel".to_string(),
        serde_json::Value::from(state.focused_panel.as_prefs_key()),
    );
    object.insert(
        "expanded_panel".to_string(),
        state
            .expanded_panel
            .map(|panel| serde_json::Value::from(panel.as_prefs_key()))
            .unwrap_or(serde_json::Value::Null),
    );
    object.insert(
        "tone_mode".to_string(),
        serde_json::Value::from(state.tone_mode.label()),
    );
    let panel_offsets = AgentPanel::ORDER
        .iter()
        .map(|panel| serde_json::Value::from(state.panel_offset(*panel) as u64))
        .collect::<Vec<_>>();
    object.insert(
        "panel_scroll_offsets".to_string(),
        serde_json::Value::Array(panel_offsets),
    );
    let payload = serde_json::to_string_pretty(&serde_json::Value::Object(object))
        .map_err(|error| format!("failed to encode tui prefs {}: {error}", path.display()))?;
    fs::write(path, payload)
        .map_err(|error| format!("failed to persist tui prefs {}: {error}", path.display()))
}

fn persist_tui_prefs_if_dirty(path: &Path, state: &mut AgentAppState) {
    if !state.ui_prefs_dirty {
        return;
    }
    match persist_tui_prefs(path, state) {
        Ok(()) => {
            state.ui_prefs_dirty = false;
        }
        Err(error) => {
            push_history_line(
                &mut state.event_lines,
                format!("prefs warning: {}", compact_ui_snippet(error.as_str(), 140)),
            );
            state.ui_prefs_dirty = false;
        }
    }
}

fn format_ms_as_s(ms_value: &str) -> Option<String> {
    ms_value.parse::<u64>().ok().map(|ms| {
        let whole = ms / 1000;
        let tenths = (ms % 1000) / 100;
        format!("{whole}.{tenths}s")
    })
}

fn format_duration_as_s(duration: Duration) -> String {
    let whole = duration.as_secs();
    let tenths = duration.subsec_millis() / 100;
    format!("{whole}.{tenths}s")
}

fn format_runtime_age(duration: Option<Duration>) -> String {
    duration
        .map(format_duration_as_s)
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_turn_activity_line(state: &AgentAppState) -> String {
    if state.turn_in_progress {
        let elapsed = format_runtime_age(state.turn_elapsed());
        let last_runtime_event = format_runtime_age(state.last_runtime_event_age());
        let events_in_turn = state.runtime_events_seen_in_turn();
        let mode = if events_in_turn == 0 {
            "awaiting-first-event"
        } else {
            "streaming"
        };
        return format!(
            "activity: {} {mode} phase={} elapsed={} last_runtime_event={} events_in_turn={events_in_turn}",
            state.spinner_glyph(),
            state.turn_phase.label(),
            elapsed,
            last_runtime_event
        );
    }
    format!(
        "activity: {} idle phase={} last_runtime_event={} last_sync={}",
        state.heartbeat_glyph(),
        state.turn_phase.label(),
        format_runtime_age(state.last_runtime_event_age()),
        format_runtime_age(state.gateway_sync_age())
    )
}

fn format_turn_budget(state: &AgentAppState) -> String {
    match (state.turn_request_budget_ms, state.turn_elapsed()) {
        (Some(total_ms), Some(elapsed)) if state.turn_in_progress => {
            let elapsed_ms = elapsed.as_millis() as u64;
            if elapsed_ms >= total_ms {
                format!("0.0s/{:.1}s", total_ms as f64 / 1000.0)
            } else {
                let remaining = Duration::from_millis(total_ms.saturating_sub(elapsed_ms));
                format!(
                    "{}/{}",
                    format_duration_as_s(remaining),
                    format_duration_as_s(Duration::from_millis(total_ms))
                )
            }
        }
        (Some(total_ms), _) => format!("-/{:.1}s", total_ms as f64 / 1000.0),
        (None, _) => "n/a".to_string(),
    }
}

fn format_live_status_bar_line(state: &AgentAppState) -> String {
    let tool = state.active_tool_name.as_deref().unwrap_or("-");
    format!(
        "status: {} phase={} tool={} budget={} last_evt={} mem={}",
        if state.turn_in_progress {
            state.spinner_glyph()
        } else {
            state.heartbeat_glyph()
        },
        state.turn_phase.label(),
        tool,
        format_turn_budget(state),
        format_runtime_age(state.last_runtime_event_age()),
        state.memory_activity_lines.len()
    )
}

fn recent_memory_activity_lines(state: &AgentAppState, max_items: usize) -> Vec<String> {
    let mut items = state
        .memory_activity_lines
        .iter()
        .rev()
        .take(max_items)
        .cloned()
        .collect::<Vec<_>>();
    items.reverse();
    items
}

fn turn_silence_duration(state: &AgentAppState) -> Option<Duration> {
    if !state.turn_in_progress {
        return None;
    }
    if state.runtime_events_seen_in_turn() > 0 {
        state.last_runtime_event_age()
    } else {
        state.turn_elapsed()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LegacyTurnLineNormalization {
    is_turn_line: bool,
    normalized: String,
    assistant_fragment: Option<String>,
}

fn extract_legacy_running_assistant_fragment(line: &str) -> Option<String> {
    if !line.starts_with("interactive.turn=running") {
        return None;
    }
    let candidate = line
        .rfind(')')
        .and_then(|idx| line.get(idx + 1..))
        .map(str::trim)
        .unwrap_or_default();
    if candidate.is_empty() {
        return None;
    }
    let looks_like_key_values = candidate
        .split_whitespace()
        .all(|token| token.contains('=') || token.ends_with(':'));
    if looks_like_key_values {
        return None;
    }
    if !candidate.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    Some(candidate.to_string())
}

fn normalize_legacy_turn_line(line: &str) -> Option<LegacyTurnLineNormalization> {
    if !line.starts_with("interactive.turn=") {
        return None;
    }
    if line.starts_with("interactive.turn=running") {
        let elapsed = parse_legacy_kv(line, "elapsed_ms")
            .and_then(|value| format_ms_as_s(&value))
            .unwrap_or_else(|| "?s".to_string());
        let remaining = parse_legacy_kv(line, "remaining_request_ms")
            .and_then(|value| format_ms_as_s(&value))
            .unwrap_or_else(|| "?s".to_string());
        let phase = parse_legacy_kv(line, "phase").unwrap_or_else(|| "model_request".to_string());
        let status = parse_legacy_kv(line, "status")
            .or_else(|| parse_legacy_kv(line, "detail"))
            .unwrap_or_else(|| phase.clone());
        let turn = parse_legacy_kv(line, "turn")
            .map(|value| format!(" (turn {value})"))
            .unwrap_or_default();
        return Some(LegacyTurnLineNormalization {
            is_turn_line: false,
            normalized: format!(
                "| {status} [{phase}{turn}] | elapsed={elapsed} | remaining={remaining}"
            ),
            assistant_fragment: extract_legacy_running_assistant_fragment(line),
        });
    }

    if line.starts_with("interactive.turn=start") {
        let timeout = parse_legacy_kv(line, "request_timeout_ms")
            .or_else(|| parse_legacy_kv(line, "turn_timeout_ms"))
            .unwrap_or_else(|| "?".to_string());
        return Some(LegacyTurnLineNormalization {
            is_turn_line: true,
            normalized: format!("turn.start timeout={timeout}ms request_budget={timeout}ms"),
            assistant_fragment: None,
        });
    }

    if line.starts_with("interactive.turn=end") {
        let status = parse_legacy_kv(line, "status").unwrap_or_else(|| "unknown".to_string());
        let elapsed = parse_legacy_kv(line, "elapsed_ms").unwrap_or_else(|| "?".to_string());
        return Some(LegacyTurnLineNormalization {
            is_turn_line: true,
            normalized: format!("turn.end status={status} elapsed={elapsed}ms"),
            assistant_fragment: None,
        });
    }

    if line.starts_with("interactive.turn=final") {
        let remaining =
            parse_legacy_kv(line, "remaining_request_ms").unwrap_or_else(|| "?".to_string());
        let phase = parse_legacy_kv(line, "phase").unwrap_or_else(|| "model_response".to_string());
        return Some(LegacyTurnLineNormalization {
            is_turn_line: true,
            normalized: format!("turn.final remaining={remaining}ms phase={phase}"),
            assistant_fragment: None,
        });
    }

    None
}

fn parse_tau_runtime_event(
    line: &str,
) -> Option<(String, serde_json::Map<String, serde_json::Value>)> {
    fn parse_tau_runtime_event_payload_value(
        parsed: &serde_json::Value,
    ) -> Option<(String, serde_json::Map<String, serde_json::Value>)> {
        let object = parsed.as_object()?;
        let event_type = object
            .get("event_type")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_string();
        let fields = object
            .get("fields")
            .and_then(serde_json::Value::as_object)
            .cloned()
            .unwrap_or_default();
        Some((event_type, fields))
    }

    fn parse_tau_runtime_event_payload_str(
        payload: &str,
    ) -> Option<(String, serde_json::Map<String, serde_json::Value>)> {
        let parsed = serde_json::from_str::<serde_json::Value>(payload).ok()?;
        parse_tau_runtime_event_payload_value(&parsed)
    }

    let trimmed = line.trim();
    if let Some(payload) = trimmed.strip_prefix("tau.event ") {
        return parse_tau_runtime_event_payload_str(payload);
    }
    if let Some(payload) = trimmed.strip_prefix("data:") {
        if let Some(parsed) = parse_tau_runtime_event_payload_str(payload.trim()) {
            return Some(parsed);
        }
    }

    let parsed = serde_json::from_str::<serde_json::Value>(trimmed).ok()?;
    if let Some(parsed_event) = parse_tau_runtime_event_payload_value(&parsed) {
        return Some(parsed_event);
    }
    let object = parsed.as_object()?;
    let event_name = object
        .get("event")
        .and_then(serde_json::Value::as_str)
        .or_else(|| object.get("event_name").and_then(serde_json::Value::as_str))
        .or_else(|| object.get("eventName").and_then(serde_json::Value::as_str))
        .map(str::trim);
    if !matches!(event_name, Some("tau.event")) {
        return None;
    }
    if let Some(payload) = object.get("payload") {
        if let Some(parsed_event) = parse_tau_runtime_event_payload_value(payload) {
            return Some(parsed_event);
        }
    }
    if let Some(data) = object.get("data") {
        if let Some(parsed_event) = parse_tau_runtime_event_payload_value(data) {
            return Some(parsed_event);
        }
        if let Some(data_text) = data.as_str() {
            return parse_tau_runtime_event_payload_str(data_text);
        }
    }
    None
}

fn tau_event_field_str(
    fields: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    fields
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn tau_event_field_u64(
    fields: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<u64> {
    fields.get(key).and_then(serde_json::Value::as_u64)
}

fn compact_tau_event_value(raw: &str, max_chars: usize) -> String {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        return collapsed;
    }
    let truncated = collapsed.chars().take(max_chars).collect::<String>();
    format!("{truncated}...")
}

fn parse_turn_phase_label(label: &str) -> Option<TurnPhase> {
    match label.trim().to_ascii_lowercase().as_str() {
        "queued" => Some(TurnPhase::Queued),
        "model" => Some(TurnPhase::Model),
        "tool" => Some(TurnPhase::Tool),
        "post_tool" | "post-tool" => Some(TurnPhase::PostTool),
        "done" | "completed" => Some(TurnPhase::Done),
        "failed" | "timed-out" | "timed_out" => Some(TurnPhase::Failed),
        "cancelled" | "canceled" => Some(TurnPhase::Cancelled),
        "idle" => Some(TurnPhase::Idle),
        _ => None,
    }
}

fn format_tau_runtime_event_line(
    event_type: &str,
    fields: &serde_json::Map<String, serde_json::Value>,
) -> String {
    let mut tokens = vec![event_type.to_string()];
    for key in [
        "phase",
        "status",
        "tool_name",
        "tool_call_id",
        "prompt_chars",
        "assistant_chars",
        "event_type",
        "error",
    ] {
        if let Some(value) = tau_event_field_str(fields, key) {
            tokens.push(format!(
                "{key}={}",
                compact_tau_event_value(value.as_str(), 72)
            ));
            continue;
        }
        if let Some(value) = tau_event_field_u64(fields, key) {
            tokens.push(format!("{key}={value}"));
        }
    }
    format!("tau.event {}", tokens.join(" "))
}

fn apply_tau_runtime_event(
    state: &mut AgentAppState,
    event_type: &str,
    fields: &serde_json::Map<String, serde_json::Value>,
) {
    fn event_budget_ms(fields: &serde_json::Map<String, serde_json::Value>) -> Option<u64> {
        tau_event_field_u64(fields, "request_budget_ms")
            .or_else(|| tau_event_field_u64(fields, "request_timeout_ms"))
            .or_else(|| tau_event_field_u64(fields, "turn_timeout_ms"))
    }

    let rendered = format_tau_runtime_event_line(event_type, fields);
    push_history_line(&mut state.event_lines, rendered.clone());

    match event_type {
        "turn.submitted" => {
            state.mark_turn_started();
            state.turn_phase = TurnPhase::Queued;
            state.turn_status = "turn.start submitted to runtime".to_string();
            state.turn_request_budget_ms =
                event_budget_ms(fields).or(state.default_request_budget_ms);
            if let Some(prompt_chars) = tau_event_field_u64(fields, "prompt_chars") {
                state.progress_status = format!("queued prompt_chars={prompt_chars}");
            }
            push_history_line(&mut state.timeline_lines, rendered);
        }
        "turn.started" => {
            state.mark_turn_started();
            if let Some(budget_ms) = event_budget_ms(fields) {
                state.turn_request_budget_ms = Some(budget_ms);
            }
            if let Some(phase) = tau_event_field_str(fields, "phase")
                .and_then(|value| parse_turn_phase_label(value.as_str()))
            {
                state.turn_phase = phase;
            } else {
                state.turn_phase = TurnPhase::Model;
            }
            push_history_line(&mut state.timeline_lines, rendered);
        }
        "turn.phase" => {
            state.mark_turn_started();
            if let Some(phase) = tau_event_field_str(fields, "phase")
                .and_then(|value| parse_turn_phase_label(value.as_str()))
            {
                state.turn_phase = phase;
            }
            if let Some(phase) = tau_event_field_str(fields, "phase") {
                state.progress_status = format!("phase={phase}");
            }
            push_history_line(&mut state.timeline_lines, rendered);
        }
        "turn.completed" | "turn.finished" => {
            state.mark_turn_finished();
            state.turn_phase = match tau_event_field_str(fields, "status")
                .map(|value| value.to_ascii_lowercase())
                .as_deref()
            {
                Some("cancelled") | Some("canceled") => TurnPhase::Cancelled,
                Some("failed") | Some("timed-out") | Some("timed_out") => TurnPhase::Failed,
                _ => TurnPhase::Done,
            };
            push_history_line(&mut state.timeline_lines, rendered);
        }
        "turn.failed" => {
            state.mark_turn_finished();
            state.turn_phase = TurnPhase::Failed;
            if let Some(error) = tau_event_field_str(fields, "error") {
                let compact = compact_tau_event_value(&error, 96);
                state.progress_status = format!("failed: {compact}");
                push_failed_turn_assistant_line(state, error.as_str());
            }
            push_history_line(&mut state.timeline_lines, rendered);
        }
        "tool.started" => {
            state.mark_turn_started();
            state.turn_phase = TurnPhase::Tool;
            state.active_tool_name = tau_event_field_str(fields, "tool_name")
                .or_else(|| tau_event_field_str(fields, "tool_call_id"));
            push_history_line(&mut state.tool_lines, rendered);
        }
        "tool.finished" => {
            state.mark_turn_started();
            state.turn_phase = TurnPhase::PostTool;
            state.active_tool_name = None;
            push_history_line(&mut state.tool_lines, rendered);
        }
        "memory.write" | "memory.delete" => {
            push_history_line(&mut state.timeline_lines, rendered.clone());
            state.push_memory_activity(format!(
                "{} via {}",
                event_type,
                tau_event_field_str(fields, "tool_name")
                    .unwrap_or_else(|| "memory tool".to_string())
            ));
            push_history_line(
                &mut state.assistant_lines,
                format!(
                    "memory updated ({})",
                    tau_event_field_str(fields, "tool_name")
                        .unwrap_or_else(|| "memory tool".to_string())
                ),
            );
        }
        "assistant.delta" => {
            if let Some(chars) = tau_event_field_u64(fields, "assistant_chars") {
                state.mark_turn_started();
                state.turn_phase = TurnPhase::Model;
                state.progress_status = format!("assistant drafting response ({chars} chars)");
            }
        }
        _ => {}
    }
}

fn push_failed_turn_assistant_line(state: &mut AgentAppState, error: &str) {
    if state.assistant_output_seen_in_turn {
        return;
    }
    push_history_line(
        &mut state.assistant_lines,
        format!("assistant error: {error}"),
    );
}

fn is_progress_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    (trimmed.starts_with("| ") || trimmed.starts_with("/ ") || trimmed.starts_with("\\ "))
        && trimmed.contains("elapsed=")
}

fn is_tool_line(line: &str) -> bool {
    line.starts_with("{\"tool_call\"")
        || line.starts_with("{\"tool_calls\"")
        || line.starts_with("[assistant requested ")
        || is_cortex_observer_tool_line(line)
}

fn is_cortex_observer_line(line: &str) -> bool {
    line.starts_with("cortex.observer ")
}

fn cortex_observer_event_type(line: &str) -> Option<&str> {
    line.split_whitespace()
        .find_map(|token| token.strip_prefix("event_type="))
}

fn is_cortex_observer_tool_line(line: &str) -> bool {
    if !is_cortex_observer_line(line) {
        return false;
    }
    matches!(
        cortex_observer_event_type(line),
        Some("local.tool.start" | "local.tool.end")
    )
}

fn is_cortex_observer_timeline_line(line: &str) -> bool {
    if !is_cortex_observer_line(line) {
        return false;
    }
    matches!(
        cortex_observer_event_type(line),
        Some(
            "local.turn.submitted"
                | "local.turn.completed"
                | "local.turn.failed"
                | "session.append"
        )
    )
}

fn extract_progress_assistant_fragment(line: &str) -> Option<String> {
    if !is_progress_line(line) {
        return None;
    }
    let mut candidate = line
        .rfind(')')
        .and_then(|idx| line.get(idx + 1..))
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if let Some((head, _)) = candidate.split_once(" | elapsed=") {
        candidate = head.trim().to_string();
    }
    candidate = candidate
        .trim_start_matches(|ch: char| ch == ']' || ch == ':' || ch == '-' || ch.is_whitespace())
        .to_string();
    if candidate.is_empty() || candidate.starts_with('|') {
        return None;
    }
    if candidate.contains("remaining=")
        || candidate.contains("phase=")
        || candidate.contains("status=")
    {
        return None;
    }
    if !candidate.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    Some(candidate)
}

fn push_assistant_message_lines(state: &mut AgentAppState, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    for line in trimmed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        push_history_line(&mut state.assistant_lines, line.to_string());
    }
    state.mark_assistant_output_seen();
}

fn apply_structured_runtime_json_line(state: &mut AgentAppState, line: &str) -> bool {
    let parsed = match serde_json::from_str::<serde_json::Value>(line) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let object = match parsed.as_object() {
        Some(object) => object,
        None => return false,
    };
    let event_type = match object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(event_type) => event_type,
        None => return false,
    };

    match event_type {
        "message_added" => {
            let role = object
                .get("role")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("unknown");
            let text = object
                .get("text")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .unwrap_or_default();
            let tool_calls = object
                .get("tool_calls")
                .and_then(|value| {
                    value
                        .as_u64()
                        .or_else(|| value.as_array().map(|calls| calls.len() as u64))
                })
                .unwrap_or_default();
            match role {
                "assistant" => {
                    if !text.is_empty() {
                        state.mark_turn_started();
                        state.turn_phase = TurnPhase::Model;
                        state.progress_status = "assistant message received".to_string();
                        push_assistant_message_lines(state, text);
                    }
                    if tool_calls > 0 {
                        state.mark_turn_started();
                        state.turn_phase = TurnPhase::PostTool;
                        push_history_line(
                            &mut state.timeline_lines,
                            format!(
                                "assistant.timeline tool_plan: requested tool_calls={tool_calls}"
                            ),
                        );
                    }
                    true
                }
                "tool" => {
                    let preview = if text.is_empty() {
                        "tool output received".to_string()
                    } else {
                        format!("tool output: {}", compact_ui_snippet(text, 120))
                    };
                    push_history_line(&mut state.tool_lines, preview);
                    true
                }
                "user" => {
                    if !text.is_empty() {
                        push_history_line(
                            &mut state.timeline_lines,
                            format!("user.input {}", compact_ui_snippet(text, 120)),
                        );
                    }
                    true
                }
                _ => {
                    push_history_line(
                        &mut state.event_lines,
                        format!(
                            "runtime.message_added role={role} chars={}",
                            text.chars().count()
                        ),
                    );
                    true
                }
            }
        }
        "agent_start" => {
            push_history_line(&mut state.event_lines, "agent.start".to_string());
            true
        }
        "turn_start" => {
            state.mark_turn_started();
            state.turn_request_budget_ms = object
                .get("request_timeout_ms")
                .and_then(serde_json::Value::as_u64)
                .or_else(|| {
                    object
                        .get("turn_timeout_ms")
                        .and_then(serde_json::Value::as_u64)
                })
                .or(state.default_request_budget_ms);
            state.turn_phase = TurnPhase::Model;
            let turn = object
                .get("turn")
                .and_then(serde_json::Value::as_u64)
                .map(|value| format!(" turn={value}"))
                .unwrap_or_default();
            push_history_line(
                &mut state.timeline_lines,
                format!("turn.started{turn} (runtime json event)"),
            );
            true
        }
        "turn_end" => {
            let status = object
                .get("status")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("completed");
            finish_turn_with_event_fallback(state, status);
            state.active_tool_name = None;
            state.turn_phase = parse_turn_phase_label(status).unwrap_or(TurnPhase::Done);
            push_history_line(
                &mut state.timeline_lines,
                format!("turn.completed status={status} (runtime json event)"),
            );
            true
        }
        _ => {
            push_history_line(
                &mut state.event_lines,
                format!("runtime.event type={event_type}"),
            );
            true
        }
    }
}

fn finish_turn_with_event_fallback(state: &mut AgentAppState, reason: &str) {
    let turn_had_no_assistant_output =
        state.turn_in_progress && !state.assistant_output_seen_in_turn;
    state.mark_turn_finished();
    if turn_had_no_assistant_output {
        push_history_line(
            &mut state.event_lines,
            format!("turn.note: completed ({reason}) with no assistant text"),
        );
    }
}

fn update_agent_app_state(state: &mut AgentAppState, source: AgentOutputSource, raw_line: &str) {
    let line = parse_stream_line_for_app(raw_line);
    if line.is_empty() {
        return;
    }
    state.note_runtime_event();

    let invalid_api_key = line.contains("invalid_api_key")
        || line.contains("Incorrect API key provided")
        || line.contains("provider returned non-success status 401");
    if invalid_api_key && !state.auth_hint_emitted {
        push_history_line(
            &mut state.event_lines,
            "auth hint: OpenAI request used API-key auth and key was rejected".to_string(),
        );
        push_history_line(
            &mut state.event_lines,
            "auth fix: unset OPENAI_API_KEY TAU_API_KEY; export TAU_OPENAI_AUTH_MODE=oauth-token; export TAU_PROVIDER_SUBSCRIPTION_STRICT=true; codex login".to_string(),
        );
        state.auth_hint_emitted = true;
    }

    if line == "event: tau.event" {
        return;
    }
    if line == "event: done" || line == "data: [DONE]" || line == "[DONE]" {
        finish_turn_with_event_fallback(state, "sse_done");
        return;
    }

    if let Some((event_type, fields)) = parse_tau_runtime_event(&line) {
        apply_tau_runtime_event(state, event_type.as_str(), &fields);
        return;
    }

    if apply_structured_runtime_json_line(state, &line) {
        return;
    }

    if let Some(normalized) = normalize_legacy_turn_line(&line) {
        if normalized.is_turn_line {
            if normalized.normalized.starts_with("turn.start ") {
                state.mark_turn_started();
                state.turn_request_budget_ms =
                    extract_request_budget_ms_from_line(normalized.normalized.as_str())
                        .or_else(|| extract_request_budget_ms_from_line(line.as_str()))
                        .or(state.default_request_budget_ms);
            } else if normalized.normalized.starts_with("turn.end ") {
                finish_turn_with_event_fallback(state, normalized.normalized.as_str());
            } else if normalized.normalized.starts_with("turn.final ") {
                state.mark_turn_finished();
            }
            state.turn_status = normalized.normalized;
        } else {
            state.mark_turn_started();
            state.progress_status = normalized.normalized;
        }
        if let Some(fragment) = normalized.assistant_fragment {
            push_assistant_message_lines(state, fragment.as_str());
        }
        return;
    }

    if line.starts_with("turn.start ")
        || line.starts_with("turn.end ")
        || line.starts_with("turn.final ")
    {
        if line.starts_with("turn.start ") {
            state.mark_turn_started();
            state.turn_request_budget_ms = extract_request_budget_ms_from_line(line.as_str())
                .or(state.default_request_budget_ms);
        } else if line.starts_with("turn.end ") {
            finish_turn_with_event_fallback(state, line.as_str());
        } else {
            state.mark_turn_finished();
        }
        state.turn_status = line;
        return;
    }

    if is_progress_line(&line) {
        state.mark_turn_started();
        state.progress_status = line;
        if let Some(fragment) = extract_progress_assistant_fragment(state.progress_status.as_str())
        {
            push_assistant_message_lines(state, fragment.as_str());
        }
        return;
    }

    if line.starts_with("[model.plan]") || line.starts_with("[model.reasoning]") {
        push_history_line(&mut state.timeline_lines, line);
        return;
    }

    if line.starts_with("assistant.timeline ") {
        push_history_line(&mut state.timeline_lines, line);
        return;
    }

    if is_cortex_observer_line(&line) {
        push_history_line(&mut state.event_lines, line.clone());
        if is_cortex_observer_timeline_line(&line) {
            push_history_line(&mut state.timeline_lines, line.clone());
        }
        if is_cortex_observer_tool_line(&line) {
            push_history_line(&mut state.tool_lines, line);
        }
        return;
    }

    if is_tool_line(&line) {
        if line.starts_with("{\"tool_call\"") || line.starts_with("{\"tool_calls\"") {
            if let Some(name_fragment) = line.split("\"name\":\"").nth(1) {
                if let Some(name) = name_fragment.split('"').next() {
                    let candidate = name.trim();
                    if !candidate.is_empty() {
                        state.active_tool_name = Some(candidate.to_string());
                        state.turn_phase = TurnPhase::Tool;
                    }
                }
            }
        }
        push_history_line(&mut state.tool_lines, line);
        return;
    }

    if line.starts_with("tau>") || line.starts_with("...>") {
        state.prompt_line = line;
        return;
    }

    if line.starts_with("controls:")
        || line.starts_with("commands:")
        || line.starts_with("shortcuts:")
        || line.starts_with("launch:")
    {
        return;
    }

    if line.starts_with("interactive turn failed:")
        || line.starts_with("request timed out")
        || line.starts_with("request cancelled")
    {
        finish_turn_with_event_fallback(state, line.as_str());
    }

    if source == AgentOutputSource::Stdout
        && !line.starts_with("model catalog:")
        && !line.starts_with("interactive turn failed:")
        && !line.starts_with("request timed out")
        && !line.starts_with("request cancelled")
    {
        push_assistant_message_lines(state, line.as_str());
        return;
    }

    push_history_line(&mut state.event_lines, line);
}

fn wrap_line(value: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    if width == 0 {
        return out;
    }
    let mut remaining = value.trim();
    if remaining.is_empty() {
        out.push(String::new());
        return out;
    }
    while remaining.chars().count() > width {
        let mut split_idx = 0usize;
        let mut last_ws_idx = None;
        for (count, (idx, ch)) in remaining.char_indices().enumerate() {
            if count >= width {
                break;
            }
            split_idx = idx + ch.len_utf8();
            if ch.is_whitespace() {
                last_ws_idx = Some(idx);
            }
        }
        let cut_idx = last_ws_idx.filter(|idx| *idx > 0).unwrap_or(split_idx);
        let segment = remaining[..cut_idx].trim_end();
        out.push(segment.to_string());
        remaining = remaining[cut_idx..].trim_start();
        if remaining.is_empty() {
            break;
        }
    }
    if !remaining.is_empty() {
        out.push(remaining.to_string());
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PanelBodyView {
    lines: Vec<String>,
    start: usize,
    end: usize,
    total: usize,
    offset_from_tail: usize,
}

fn panel_body_view(
    lines: &[String],
    width: usize,
    body_lines_budget: usize,
    scroll_from_tail: usize,
) -> PanelBodyView {
    let mut body = Vec::new();
    if lines.is_empty() {
        body.push("(none)".to_string());
    } else {
        for line in lines {
            let wrapped = wrap_line(line, width.max(1));
            body.extend(wrapped);
        }
    }

    let body_lines_budget = body_lines_budget.max(1);
    let max_start = body.len().saturating_sub(body_lines_budget);
    let offset_from_tail = scroll_from_tail.min(max_start);
    let start = max_start.saturating_sub(offset_from_tail);
    let end = (start + body_lines_budget).min(body.len());
    PanelBodyView {
        lines: body[start..end].to_vec(),
        start,
        end,
        total: body.len(),
        offset_from_tail,
    }
}

fn panel_body_view_lines(
    lines: &[String],
    width: usize,
    body_lines_budget: usize,
    scroll_from_tail: usize,
) -> Vec<String> {
    let mut view = panel_body_view(lines, width, body_lines_budget, scroll_from_tail);
    let mut out = std::mem::take(&mut view.lines);
    if view.total > body_lines_budget.max(1) {
        let view_mode = if view.offset_from_tail == 0 {
            "tail".to_string()
        } else {
            format!("scrolled +{}", view.offset_from_tail)
        };
        out.push(format!(
            "view: {}-{} / {} ({view_mode})",
            view.start + 1,
            view.end,
            view.total
        ));
    }
    out
}

fn input_prompt_view(pending_input: &str, visible_width: usize) -> (String, u16) {
    let prefix = "tau> ";
    let prefix_width = prefix.chars().count();
    let min_width = prefix_width + 1;
    if visible_width < min_width {
        return ("tau>".to_string(), 3);
    }

    let max_input_width = visible_width.saturating_sub(prefix_width);
    let chars = pending_input.chars().collect::<Vec<_>>();
    let mut visible = if chars.len() > max_input_width {
        let mut tail = chars[chars.len().saturating_sub(max_input_width)..]
            .iter()
            .collect::<String>();
        if max_input_width >= 2 {
            tail = chars[chars.len().saturating_sub(max_input_width - 1)..]
                .iter()
                .collect::<String>();
            tail.insert(0, '…');
        }
        tail
    } else {
        pending_input.to_string()
    };
    if visible.chars().count() > max_input_width {
        visible = visible.chars().take(max_input_width).collect();
    }
    let line = format!("{prefix}{visible}");
    (line, (prefix_width + visible.chars().count()) as u16)
}

fn current_terminal_dimensions(default_width: usize, default_height: usize) -> (usize, usize) {
    let mut width = env::var("COLUMNS")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .unwrap_or(default_width);
    let mut height = env::var("LINES")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .unwrap_or(default_height);

    if std::io::stdout().is_terminal() {
        let stty_size = Command::new("stty")
            .arg("size")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());
        if let Some(size) = stty_size {
            let mut parts = size.split_whitespace();
            let parsed_height = parts.next().and_then(|raw| raw.parse::<usize>().ok());
            let parsed_width = parts.next().and_then(|raw| raw.parse::<usize>().ok());
            if let Some(parsed_width) = parsed_width {
                width = parsed_width;
            }
            if let Some(parsed_height) = parsed_height {
                height = parsed_height;
            }
        }
    }

    (width.max(20), height.max(12))
}

fn command_has_flag(command: &str, flag: &str) -> bool {
    command
        .split_whitespace()
        .any(|token| token == flag || token.starts_with(&format!("{flag}=")))
}

fn command_flag_value(command: &str, flag: &str) -> Option<String> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        if let Some(value) = token.strip_prefix(&format!("{flag}=")) {
            if !value.trim().is_empty() {
                return Some(value.to_string());
            }
        }
        if *token == flag {
            if let Some(next) = tokens.get(index + 1) {
                if !next.starts_with("--") && !next.trim().is_empty() {
                    return Some((*next).to_string());
                }
            }
        }
    }
    None
}

fn render_panel(
    title: &str,
    lines: &[String],
    width: usize,
    body_lines_budget: usize,
    scroll_from_tail: usize,
    is_focused: bool,
    is_expanded: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut section_title = format!("[{title}]");
    if is_expanded {
        section_title.push_str(" [expanded]");
    }
    if is_focused {
        section_title = format!("> {section_title}");
    } else {
        section_title = format!("  {section_title}");
    }
    let separator_len = width
        .saturating_sub(section_title.chars().count())
        .saturating_sub(1)
        .max(3);
    out.push(format!("{section_title} {}", "-".repeat(separator_len)));
    let body = panel_body_view_lines(
        lines,
        width.saturating_sub(2),
        body_lines_budget,
        scroll_from_tail,
    );
    for line in body {
        out.push(format!("  {line}"));
    }
    out
}

fn fit_line_to_width(line: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut count = 0usize;
    for ch in line.chars() {
        if count >= width {
            break;
        }
        out.push(ch);
        count += 1;
    }
    if line.chars().count() > width {
        if width >= 3 {
            let mut truncated = out.chars().take(width - 3).collect::<String>();
            truncated.push_str("...");
            return truncated;
        }
        return ".".repeat(width);
    }
    if count < width {
        out.push_str(&" ".repeat(width - count));
    }
    out
}

fn merge_panel_columns(
    left_lines: &[String],
    right_lines: &[String],
    left_width: usize,
    right_width: usize,
    gap: &str,
) -> Vec<String> {
    let row_count = left_lines.len().max(right_lines.len());
    let mut out = Vec::with_capacity(row_count);
    for index in 0..row_count {
        let left = left_lines
            .get(index)
            .map_or_else(String::new, |line| fit_line_to_width(line, left_width));
        let right = right_lines
            .get(index)
            .map_or_else(String::new, |line| fit_line_to_width(line, right_width));
        out.push(format!("{left}{gap}{right}"));
    }
    out
}

fn shortcut_help_lines() -> Vec<String> {
    vec![
        "Enter submit | Ctrl+C cancel turn | Ctrl+D quit".to_string(),
        "Ctrl+G toggle color mode (semantic/minimal)".to_string(),
        "Tab/Shift+Tab or Ctrl+N/Ctrl+P switch panel focus".to_string(),
        "Arrow up/down or Ctrl+B/Ctrl+F scroll focused panel".to_string(),
        "[ / ] switch panel focus | j / k scroll (when input is empty)".to_string(),
        "PageUp/PageDown jump-scroll focused panel".to_string(),
        "Home/End jump to oldest/latest lines".to_string(),
        "Ctrl+E expand/collapse focused content panel".to_string(),
        "Resize split: Ctrl+Left/Right and Ctrl+Up/Down".to_string(),
        "? toggle this shortcuts panel".to_string(),
    ]
}

fn compose_agent_app_lines(
    args: &AgentArgs,
    summary_lines: &[String],
    state: &AgentAppState,
    launch_command: &str,
) -> Vec<String> {
    let (terminal_width, terminal_height) = current_terminal_dimensions(args.width, 40);
    let width = terminal_width.saturating_sub(1).max(40);
    let mut frame_lines = Vec::new();

    let mut session_lines = Vec::new();
    for prefix in ["Tau agent CLI", "model:"] {
        if let Some(line) = summary_lines.iter().find(|line| line.starts_with(prefix)) {
            session_lines.push(line.clone());
        }
    }
    let runtime_binary = launch_command
        .split_whitespace()
        .next()
        .map(str::to_string)
        .unwrap_or_else(|| "tau-coding-agent".to_string());
    session_lines.push(format!(
        "live: {} events={} last_event={}",
        state.heartbeat_glyph(),
        state.runtime_event_count,
        format_runtime_age(state.last_runtime_event_age())
    ));
    session_lines.push(format_turn_activity_line(state));
    let mcp_enabled = if command_has_flag(launch_command, "--mcp-client") {
        "on"
    } else {
        "off"
    };
    let memory_state_dir = command_flag_value(launch_command, "--memory-state-dir")
        .unwrap_or_else(|| ".tau/memory".to_string());
    let skills_dir = command_flag_value(launch_command, "--skills-dir")
        .unwrap_or_else(|| ".tau/skills".to_string());
    session_lines.push(format!(
        "wiring: mcp_client={mcp_enabled} | memory={memory_state_dir} | skills={skills_dir}"
    ));
    session_lines.push(format!(
        "integration: {} | age={}",
        state.gateway_sync_status,
        format_runtime_age(state.gateway_sync_age())
    ));
    session_lines.push(format!(
        "legend: {} | colors={}",
        state.tone_mode.legend(),
        state.tone_mode.label()
    ));
    session_lines.push(format!("runtime: {runtime_binary}"));
    session_lines.push(
        "help: /help /status /dashboard /tools /routines /cortex /memory /memory-distill /sync /colors | shortcuts ? | cancel Ctrl+C | exit /quit"
            .to_string(),
    );

    let mut turn_lines = Vec::new();
    turn_lines.push(format_live_status_bar_line(state));
    if state.turn_in_progress {
        let elapsed = state
            .turn_elapsed()
            .map(format_duration_as_s)
            .unwrap_or_else(|| "?s".to_string());
        turn_lines.push(format!(
            "{} processing turn (model/tools active) elapsed={elapsed}",
            state.spinner_glyph(),
        ));
        if let Some(silent_for) = turn_silence_duration(state) {
            if silent_for >= Duration::from_secs(4) {
                turn_lines.push(format!(
                    "runtime silent for {} (waiting for first model/tool event)",
                    format_duration_as_s(silent_for)
                ));
            }
        }
    } else {
        turn_lines.push(format!(
            "{} idle (awaiting prompt/runtime event)",
            state.heartbeat_glyph()
        ));
    }
    turn_lines.push(format!("phase: {}", state.turn_phase.label()));
    if !state.last_submitted_prompt.is_empty() {
        turn_lines.push(format!(
            "submitted[{}]: {}",
            state.submitted_turn_count, state.last_submitted_prompt
        ));
    }
    if !state.turn_status.is_empty() {
        turn_lines.push(state.turn_status.clone());
    }
    if !state.progress_status.is_empty() {
        turn_lines.push(state.progress_status.clone());
    }
    let memory_activity = recent_memory_activity_lines(state, 3);
    if memory_activity.is_empty() {
        turn_lines.push("memory: (none yet)".to_string());
    } else {
        turn_lines.push(format!("memory: {}", memory_activity.join(" | ")));
    }
    if turn_lines.is_empty() {
        turn_lines.push("waiting for first turn".to_string());
    }

    let assistant_lines = state.assistant_lines.iter().cloned().collect::<Vec<_>>();
    let timeline_lines = state.timeline_lines.iter().cloned().collect::<Vec<_>>();
    let mut tool_lines = state.integration_tools_lines.clone();
    if !state.integration_routines_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("---".to_string());
        }
        tool_lines.push("routines".to_string());
        tool_lines.extend(state.integration_routines_lines.clone());
    }
    if !state.integration_cortex_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("---".to_string());
        }
        tool_lines.push("cortex".to_string());
        tool_lines.extend(state.integration_cortex_lines.clone());
    }
    if !state.integration_memory_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("---".to_string());
        }
        tool_lines.push("memory".to_string());
        tool_lines.extend(state.integration_memory_lines.clone());
    }
    if !state.tool_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("--- runtime tool activity ---".to_string());
        }
        tool_lines.extend(state.tool_lines.iter().cloned());
    }
    let mut event_lines = state.dashboard_lines.clone();
    if !state.event_lines.is_empty() {
        if !event_lines.is_empty() {
            event_lines.push("--- runtime events ---".to_string());
        }
        event_lines.extend(state.event_lines.iter().cloned());
    }

    let session_budget = 4_usize;
    let turn_budget = 3_usize;
    let shortcuts_budget = 3_usize;
    let input_budget = 2_usize;
    let render_agent_panel =
        |panel: AgentPanel, lines: &[String], panel_width: usize, budget: usize| {
            render_panel(
                &state.panel_title(panel),
                lines,
                panel_width,
                budget,
                state.panel_offset(panel),
                state.panel_is_focused(panel),
                state.panel_is_expanded(panel),
            )
        };

    frame_lines.extend(render_agent_panel(
        AgentPanel::Session,
        &session_lines,
        width,
        session_budget,
    ));
    frame_lines.push(String::new());
    frame_lines.extend(render_agent_panel(
        AgentPanel::Turn,
        &turn_lines,
        width,
        turn_budget,
    ));
    frame_lines.push(String::new());

    if let Some(expanded) = state.expanded_panel {
        let shortcuts_rows = if state.show_shortcuts {
            1 + shortcuts_budget + 1 + 1
        } else {
            0
        };
        let fixed_rows = (1 + session_budget + 1)
            + 1
            + (1 + turn_budget + 1)
            + 1
            + shortcuts_rows
            + (1 + input_budget + 1);
        let expanded_budget = terminal_height.saturating_sub(fixed_rows).max(8);
        let expanded_content = match expanded {
            AgentPanel::Session => &session_lines,
            AgentPanel::Turn => &turn_lines,
            AgentPanel::Assistant => &assistant_lines,
            AgentPanel::Timeline => &timeline_lines,
            AgentPanel::Tools => &tool_lines,
            AgentPanel::Events => &event_lines,
        };
        frame_lines.extend(render_agent_panel(
            expanded,
            expanded_content,
            width,
            expanded_budget,
        ));
        frame_lines.push(String::new());
    } else {
        let split_layout = width >= 100;
        if split_layout {
            let gap = "   ";
            let gap_width = gap.chars().count();
            let shortcuts_rows = if state.show_shortcuts {
                1 + shortcuts_budget + 1 + 1
            } else {
                0
            };
            let fixed_rows = (1 + session_budget + 1)
                + 1
                + (1 + turn_budget + 1)
                + 1
                + shortcuts_rows
                + (1 + input_budget + 1);
            let main_total_budget = terminal_height.saturating_sub(fixed_rows).max(10);

            let total_content_width = width.saturating_sub(gap_width);
            let mut left_width = (total_content_width * state.split_left_percent as usize) / 100;
            let min_left_width = 42_usize.min(total_content_width.saturating_sub(1));
            let min_right_width = if width >= 140 { 40_usize } else { 34_usize }
                .min(total_content_width.saturating_sub(1));
            if left_width < min_left_width {
                left_width = min_left_width;
            }
            if total_content_width.saturating_sub(left_width) < min_right_width {
                left_width = total_content_width.saturating_sub(min_right_width);
            }
            let right_width = total_content_width.saturating_sub(left_width);

            let mut top_budget = (main_total_budget * state.split_top_percent as usize) / 100;
            let min_bottom = 4_usize;
            let min_top = 6_usize;
            if top_budget < min_top {
                top_budget = min_top;
            }
            if main_total_budget.saturating_sub(top_budget) < min_bottom {
                top_budget = main_total_budget.saturating_sub(min_bottom);
            }
            let bottom_budget = main_total_budget.saturating_sub(top_budget);

            let assistant_panel = render_agent_panel(
                AgentPanel::Assistant,
                &assistant_lines,
                left_width,
                top_budget,
            );
            let timeline_panel = render_agent_panel(
                AgentPanel::Timeline,
                &timeline_lines,
                right_width,
                top_budget,
            );
            frame_lines.extend(merge_panel_columns(
                &assistant_panel,
                &timeline_panel,
                left_width,
                right_width,
                gap,
            ));
            frame_lines.push(String::new());

            let tools_panel =
                render_agent_panel(AgentPanel::Tools, &tool_lines, left_width, bottom_budget);
            let events_panel =
                render_agent_panel(AgentPanel::Events, &event_lines, right_width, bottom_budget);
            frame_lines.extend(merge_panel_columns(
                &tools_panel,
                &events_panel,
                left_width,
                right_width,
                gap,
            ));
            frame_lines.push(String::new());
        } else {
            let panel_min_budgets = [2_usize, 2, 4, 4, 3, 3];
            let panel_weights = [1_usize, 1, 6, 4, 2, 3];
            let fixed_headers = 7_usize;
            let fixed_blanks = 6_usize;
            let panel_minimum_total = panel_min_budgets.iter().sum::<usize>();
            let shortcuts_rows = if state.show_shortcuts {
                1 + shortcuts_budget + 1 + 1
            } else {
                0
            };
            let available_panel_body = terminal_height
                .saturating_sub(
                    fixed_headers
                        + fixed_blanks
                        + input_budget
                        + AgentPanel::ORDER.len()
                        + shortcuts_rows,
                )
                .max(panel_minimum_total);
            let mut panel_budgets = panel_min_budgets;
            let remaining = available_panel_body.saturating_sub(panel_minimum_total);
            let total_weight = panel_weights.iter().sum::<usize>().max(1);
            let mut distributed = 0usize;
            for panel in AgentPanel::ORDER {
                let extra = (remaining * panel_weights[panel.index()]) / total_weight;
                panel_budgets[panel.index()] = panel_budgets[panel.index()].saturating_add(extra);
                distributed = distributed.saturating_add(extra);
            }
            let mut remainder = remaining.saturating_sub(distributed);
            while remainder > 0 {
                panel_budgets[AgentPanel::Assistant.index()] =
                    panel_budgets[AgentPanel::Assistant.index()].saturating_add(1);
                remainder -= 1;
                if remainder == 0 {
                    break;
                }
                panel_budgets[AgentPanel::Timeline.index()] =
                    panel_budgets[AgentPanel::Timeline.index()].saturating_add(1);
                remainder -= 1;
            }

            frame_lines.extend(render_agent_panel(
                AgentPanel::Assistant,
                &assistant_lines,
                width,
                panel_budgets[AgentPanel::Assistant.index()],
            ));
            frame_lines.push(String::new());
            frame_lines.extend(render_agent_panel(
                AgentPanel::Timeline,
                &timeline_lines,
                width,
                panel_budgets[AgentPanel::Timeline.index()],
            ));
            frame_lines.push(String::new());
            frame_lines.extend(render_agent_panel(
                AgentPanel::Tools,
                &tool_lines,
                width,
                panel_budgets[AgentPanel::Tools.index()],
            ));
            frame_lines.push(String::new());
            frame_lines.extend(render_agent_panel(
                AgentPanel::Events,
                &event_lines,
                width,
                panel_budgets[AgentPanel::Events.index()],
            ));
            frame_lines.push(String::new());
        }
    }

    if state.show_shortcuts {
        frame_lines.extend(render_panel(
            "Shortcuts",
            &shortcut_help_lines(),
            width,
            shortcuts_budget,
            0,
            false,
            false,
        ));
        frame_lines.push(String::new());
    }

    let input_lines = vec![state
        .prompt_line
        .clone()
        .if_empty_then("tau> (type and press Enter)".to_string())];
    frame_lines.extend(render_panel(
        "Input",
        &input_lines,
        width,
        input_budget,
        0,
        false,
        false,
    ));
    frame_lines
}

fn redraw_agent_app(
    args: &AgentArgs,
    summary_lines: &[String],
    state: &AgentAppState,
    launch_command: &str,
) {
    let mut frame_lines = compose_agent_app_lines(args, summary_lines, state, launch_command);
    let (_, terminal_height) = current_terminal_dimensions(args.width, 40);
    if frame_lines.len() > terminal_height {
        let overflow = frame_lines.len().saturating_sub(terminal_height);
        frame_lines.drain(0..overflow);
    }
    print!("\x1b[2J\x1b[H");
    for line in &frame_lines {
        println!("{line}");
    }
    if let Some((row_index, prompt_line)) = frame_lines
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| line.starts_with("  tau>"))
    {
        let row = row_index + 1;
        let col = prompt_line.chars().count().saturating_add(1);
        print!("\x1b[{row};{col}H");
    } else {
        let row = frame_lines.len().max(1);
        print!("\x1b[{row};1H");
    }
    print!("\x1b[?25h");
    let _ = std::io::stdout().flush();
}

fn panel_header_label(state: &AgentAppState, panel: AgentPanel) -> String {
    let mut title = state.panel_title(panel);
    if state.panel_is_expanded(panel) {
        title.push_str(" [expanded]");
    }
    if state.panel_is_focused(panel) {
        format!("> {title}")
    } else {
        format!("  {title}")
    }
}

fn panel_header_label_with_count(state: &AgentAppState, panel: AgentPanel, count: usize) -> String {
    let base = panel_header_label(state, panel);
    format!("{base} ({count})")
}

fn panel_accent_color(panel: AgentPanel) -> Color {
    match panel {
        AgentPanel::Session => Color::LightBlue,
        AgentPanel::Turn => Color::LightCyan,
        AgentPanel::Assistant => Color::White,
        AgentPanel::Timeline => Color::LightMagenta,
        AgentPanel::Tools => Color::LightYellow,
        AgentPanel::Events => Color::LightRed,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum IntegrationStatusTone {
    Neutral,
    Info,
    Good,
    Warn,
    Bad,
}

fn status_token_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split_whitespace().find_map(|token| {
        let (token_key, token_value) = token.split_once('=')?;
        if !token_key.eq_ignore_ascii_case(key) {
            return None;
        }
        Some(token_value.trim_end_matches([',', ';']))
    })
}

fn status_token_u64(line: &str, key: &str) -> Option<u64> {
    status_token_value(line, key)?.parse::<u64>().ok()
}

fn merge_status_tone(
    current: IntegrationStatusTone,
    next: IntegrationStatusTone,
) -> IntegrationStatusTone {
    current.max(next)
}

fn map_health_like_value_to_tone(value: &str) -> IntegrationStatusTone {
    match value.trim().to_ascii_lowercase().as_str() {
        "healthy" | "running" | "ready" | "ok" | "open" | "enabled" => IntegrationStatusTone::Good,
        "unknown" | "pending" | "degraded" | "limited" => IntegrationStatusTone::Warn,
        "error" | "failed" | "unhealthy" | "down" | "blocked" => IntegrationStatusTone::Bad,
        _ => IntegrationStatusTone::Info,
    }
}

fn map_runtime_status_to_tone(value: &str) -> IntegrationStatusTone {
    match value.trim().to_ascii_lowercase().as_str() {
        "completed" | "running" | "succeeded" | "ready" => IntegrationStatusTone::Good,
        "queued" | "pending" | "starting" | "in_flight" | "in-flight" => {
            IntegrationStatusTone::Info
        }
        "unknown" => IntegrationStatusTone::Warn,
        "failed" | "error" | "cancelled" | "canceled" | "timed-out" => IntegrationStatusTone::Bad,
        _ => IntegrationStatusTone::Info,
    }
}

fn style_for_integration_tone(tone: IntegrationStatusTone) -> Style {
    match tone {
        IntegrationStatusTone::Good => Style::default().fg(Color::LightGreen),
        IntegrationStatusTone::Warn => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        IntegrationStatusTone::Bad => Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD),
        IntegrationStatusTone::Info => Style::default().fg(Color::LightCyan),
        IntegrationStatusTone::Neutral => Style::default().fg(Color::Gray),
    }
}

fn integration_status_line_style(line: &str) -> Option<Style> {
    let trimmed = line.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    let mut tone = IntegrationStatusTone::Neutral;

    if trimmed.starts_with("dashboard ") {
        if lower.contains(" unavailable") {
            return Some(style_for_integration_tone(IntegrationStatusTone::Bad));
        }
        if lower.contains(" partial") || lower.contains("degraded") {
            tone = merge_status_tone(tone, IntegrationStatusTone::Warn);
        }
        if let Some(health) = status_token_value(trimmed, "health") {
            tone = merge_status_tone(tone, map_health_like_value_to_tone(health));
        }
        if let Some(run_state) = status_token_value(trimmed, "run_state") {
            tone = merge_status_tone(tone, map_runtime_status_to_tone(run_state));
        }
        if let Some(severity) = status_token_value(trimmed, "severity") {
            tone = merge_status_tone(
                tone,
                match severity.trim().to_ascii_lowercase().as_str() {
                    "info" => IntegrationStatusTone::Info,
                    "warn" | "warning" => IntegrationStatusTone::Warn,
                    "error" | "critical" => IntegrationStatusTone::Bad,
                    _ => IntegrationStatusTone::Neutral,
                },
            );
        }
        return Some(style_for_integration_tone(tone));
    }

    if trimmed.starts_with("routines ") {
        if lower.contains(" unavailable") {
            return Some(style_for_integration_tone(IntegrationStatusTone::Bad));
        }
        if let Some(health) = status_token_value(trimmed, "health") {
            tone = merge_status_tone(tone, map_health_like_value_to_tone(health));
        }
        if let Some(status) = status_token_value(trimmed, "status") {
            tone = merge_status_tone(tone, map_runtime_status_to_tone(status));
        }
        if let Some(queue_depth) = status_token_u64(trimmed, "queue_depth") {
            if queue_depth > 0 {
                tone = merge_status_tone(tone, IntegrationStatusTone::Info);
            }
        }
        return Some(style_for_integration_tone(tone));
    }

    if trimmed.starts_with("cortex ") {
        if lower.contains(" unavailable") {
            return Some(style_for_integration_tone(IntegrationStatusTone::Bad));
        }
        if let Some(health) = status_token_value(trimmed, "health") {
            tone = merge_status_tone(tone, map_health_like_value_to_tone(health));
        }
        if let Some(rollout_gate) = status_token_value(trimmed, "rollout_gate") {
            tone = merge_status_tone(
                tone,
                match rollout_gate.trim().to_ascii_lowercase().as_str() {
                    "open" => IntegrationStatusTone::Good,
                    "unknown" => IntegrationStatusTone::Warn,
                    "closed" | "paused" => IntegrationStatusTone::Warn,
                    _ => IntegrationStatusTone::Info,
                },
            );
        }
        if lower.contains("diagnostics=(none)") {
            tone = merge_status_tone(tone, IntegrationStatusTone::Good);
        } else if lower.contains("diagnostics=") {
            tone = merge_status_tone(tone, IntegrationStatusTone::Warn);
        }
        return Some(style_for_integration_tone(tone));
    }

    if trimmed.starts_with("memory ") {
        if lower.contains(" unavailable") {
            return Some(style_for_integration_tone(IntegrationStatusTone::Bad));
        }
        if let Some(enabled) = status_token_value(trimmed, "enabled") {
            tone = merge_status_tone(
                tone,
                if enabled.eq_ignore_ascii_case("true") {
                    IntegrationStatusTone::Good
                } else {
                    IntegrationStatusTone::Warn
                },
            );
        }
        if let Some(failures) = status_token_u64(trimmed, "write_failures") {
            tone = merge_status_tone(
                tone,
                if failures > 0 {
                    IntegrationStatusTone::Bad
                } else {
                    IntegrationStatusTone::Good
                },
            );
        }
        if lower.contains("reason_codes=(none)") {
            tone = merge_status_tone(tone, IntegrationStatusTone::Info);
        }
        return Some(style_for_integration_tone(tone));
    }

    None
}

fn styled_tools_metric_line(line: &str) -> Option<Line<'static>> {
    fn parse_count(value: &str) -> Option<u64> {
        if value == "-" {
            None
        } else {
            value.parse::<u64>().ok()
        }
    }

    fn source_style(source: &str) -> Style {
        match source {
            "runtime" => Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
            "ui-tools" => Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
            "ui-all" => Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::Gray),
        }
    }

    fn events_style(events: &str) -> Style {
        match parse_count(events) {
            None => Style::default().fg(Color::DarkGray),
            Some(0) => Style::default().fg(Color::Gray),
            Some(_) => Style::default().fg(Color::LightGreen),
        }
    }

    fn invalid_style(invalid: &str) -> Style {
        match parse_count(invalid) {
            None => Style::default().fg(Color::DarkGray),
            Some(0) => Style::default().fg(Color::LightGreen),
            Some(1..=3) => Style::default().fg(Color::Yellow),
            Some(_) => Style::default().fg(Color::LightRed),
        }
    }

    fn status_style(status: &str) -> Style {
        match status {
            "ok" => Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            "idle" => Style::default().fg(Color::LightBlue),
            "warn" => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            "bad" => Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
            "n/a" => Style::default().fg(Color::DarkGray),
            _ => Style::default().fg(Color::Gray),
        }
    }

    let trimmed = line.trim();
    if trimmed == "tools metrics" {
        return Some(Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )));
    }

    if trimmed.starts_with("source")
        && trimmed.contains("events")
        && trimmed.contains("invalid")
        && trimmed.contains("status")
    {
        return Some(Line::from(Span::styled(
            format!(
                "{:<11} {:>7} {:>7} {:<7} {}",
                "source", "events", "invalid", "status", "note"
            ),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let mut parts = trimmed.split_whitespace();
    let source = parts.next()?;
    if !matches!(source, "runtime" | "ui-tools" | "ui-all") {
        return None;
    }
    let events = parts.next().unwrap_or("-");
    let invalid = parts.next().unwrap_or("-");
    let status = parts.next().unwrap_or("n/a");
    let note = parts.collect::<Vec<_>>().join(" ");

    let note_style = if matches!(status, "bad") {
        Style::default().fg(Color::LightRed)
    } else if matches!(status, "warn") {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    Some(Line::from(vec![
        Span::styled(format!("{:<11}", source), source_style(source)),
        Span::raw(" "),
        Span::styled(format!("{:>7}", events), events_style(events)),
        Span::raw(" "),
        Span::styled(format!("{:>7}", invalid), invalid_style(invalid)),
        Span::raw(" "),
        Span::styled(format!("{:<7}", status), status_style(status)),
        Span::raw(" "),
        Span::styled(note, note_style),
    ]))
}

fn styled_panel_line(panel: AgentPanel, line: &str, tone_mode: ToneMode) -> Line<'static> {
    if matches!(tone_mode, ToneMode::Semantic) {
        if let Some(style) = integration_status_line_style(line) {
            return Line::from(Span::styled(line.to_string(), style));
        }
        if matches!(panel, AgentPanel::Tools) {
            if let Some(styled_metrics) = styled_tools_metric_line(line) {
                return styled_metrics;
            }
        }
    }

    let lower = line.to_ascii_lowercase();
    let is_error = lower.contains("error")
        || lower.contains("failed")
        || lower.contains("invalid")
        || lower.contains("timed out");
    let is_warning = lower.contains("warn") || lower.contains("degraded");
    let is_success = lower.contains("completed")
        || lower.contains("healthy")
        || lower.contains("ready")
        || lower.contains("succeeded");
    let is_json = line.trim_start().starts_with('{') || line.trim_start().starts_with('[');
    let is_user = line.starts_with("you>");
    let is_hint = line.starts_with("auth hint:") || line.starts_with("auth fix:");
    let is_view_marker = line.starts_with("view: ");
    let is_cortex_observer = is_cortex_observer_line(line);
    let is_tau_event = line.starts_with("tau.event ");

    let style = if is_user {
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD)
    } else if is_cortex_observer {
        if line.contains("status=error") {
            Style::default().fg(Color::LightRed)
        } else if is_cortex_observer_tool_line(line) {
            Style::default().fg(Color::LightYellow)
        } else if is_cortex_observer_timeline_line(line) {
            Style::default().fg(Color::LightCyan)
        } else {
            Style::default().fg(Color::LightBlue)
        }
    } else if is_tau_event {
        if line.contains("failed") || line.contains("status=error") {
            Style::default().fg(Color::LightRed)
        } else if line.contains("memory.") {
            Style::default().fg(Color::LightGreen)
        } else if line.contains("tool.") {
            Style::default().fg(Color::LightYellow)
        } else {
            Style::default().fg(Color::LightCyan)
        }
    } else if is_error {
        Style::default().fg(Color::LightRed)
    } else if is_warning {
        Style::default().fg(Color::Yellow)
    } else if is_success {
        Style::default().fg(Color::LightGreen)
    } else if is_hint {
        Style::default().fg(Color::LightMagenta)
    } else if is_view_marker {
        Style::default().fg(Color::DarkGray)
    } else if is_json {
        match panel {
            AgentPanel::Tools => Style::default().fg(Color::LightYellow),
            AgentPanel::Timeline => Style::default().fg(Color::LightMagenta),
            _ => Style::default().fg(Color::LightBlue),
        }
    } else {
        match panel {
            AgentPanel::Session | AgentPanel::Turn => Style::default().fg(Color::Gray),
            AgentPanel::Assistant => Style::default().fg(Color::White),
            AgentPanel::Timeline => Style::default().fg(Color::LightMagenta),
            AgentPanel::Tools => Style::default().fg(Color::LightYellow),
            AgentPanel::Events => Style::default().fg(Color::Gray),
        }
    };
    Line::from(Span::styled(line.to_string(), style))
}

#[allow(clippy::too_many_arguments)]
fn render_ratatui_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    panel: AgentPanel,
    title: String,
    lines: &[String],
    scroll_from_tail: usize,
    is_focused: bool,
    tone_mode: ToneMode,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }
    let body_width = area.width.saturating_sub(2) as usize;
    let body_height = area.height.saturating_sub(2) as usize;
    let view = panel_body_view(lines, body_width, body_height, scroll_from_tail);
    let paragraph_lines = view
        .lines
        .iter()
        .map(|line| styled_panel_line(panel, line, tone_mode))
        .collect::<Vec<_>>();
    let border_color = if is_focused {
        panel_accent_color(panel)
    } else {
        Color::DarkGray
    };
    let title_style = if is_focused {
        Style::default()
            .fg(panel_accent_color(panel))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let paragraph = Paragraph::new(paragraph_lines)
        .block(
            Block::default()
                .title(Line::from(Span::styled(title, title_style)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);

    if view.total > body_height && area.width > 5 && area.height > 3 {
        let mut scrollbar_state = ScrollbarState::new(view.total).position(view.start);
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(if is_focused {
                panel_accent_color(panel)
            } else {
                Color::DarkGray
            }))
            .track_style(Style::default().fg(Color::DarkGray));
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn draw_agent_ratatui(
    frame: &mut ratatui::Frame<'_>,
    summary_lines: &[String],
    state: &AgentAppState,
    launch_command: &str,
    pending_input: &str,
) {
    let area = frame.area();
    if area.width < 10 || area.height < 8 {
        return;
    }

    let shell = Block::default()
        .title(" Tau Agent Live ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let shell_inner = shell.inner(area);
    frame.render_widget(shell, area);

    if shell_inner.width < 8 || shell_inner.height < 6 {
        return;
    }

    let runtime_binary = launch_command
        .split_whitespace()
        .next()
        .map(str::to_string)
        .unwrap_or_else(|| "tau-coding-agent".to_string());

    let mut session_lines = Vec::new();
    for prefix in ["Tau agent CLI", "model:"] {
        if let Some(line) = summary_lines.iter().find(|line| line.starts_with(prefix)) {
            session_lines.push(line.clone());
        }
    }
    session_lines.push(format!(
        "live: {} events={} last_event={} turns={}",
        state.heartbeat_glyph(),
        state.runtime_event_count,
        format_runtime_age(state.last_runtime_event_age()),
        state.submitted_turn_count
    ));
    session_lines.push(format_turn_activity_line(state));
    let mcp_enabled = if command_has_flag(launch_command, "--mcp-client") {
        "on"
    } else {
        "off"
    };
    let memory_state_dir = command_flag_value(launch_command, "--memory-state-dir")
        .unwrap_or_else(|| ".tau/memory".to_string());
    let skills_dir = command_flag_value(launch_command, "--skills-dir")
        .unwrap_or_else(|| ".tau/skills".to_string());
    session_lines.push(format!(
        "wiring: mcp_client={mcp_enabled} | memory={memory_state_dir} | skills={skills_dir}"
    ));
    session_lines.push(format!(
        "integration: {} | age={}",
        state.gateway_sync_status,
        format_runtime_age(state.gateway_sync_age())
    ));
    session_lines.push(format!(
        "legend: {} | colors={}",
        state.tone_mode.legend(),
        state.tone_mode.label()
    ));
    session_lines.push(format!("runtime: {runtime_binary}"));
    session_lines.push(
        "help: /help /status /dashboard /tools /routines /cortex /memory /memory-distill /sync /colors | shortcuts ? | cancel Ctrl+C | exit /quit"
            .to_string(),
    );

    let mut turn_lines = Vec::new();
    turn_lines.push(format_live_status_bar_line(state));
    if state.turn_in_progress {
        let elapsed = state
            .turn_elapsed()
            .map(format_duration_as_s)
            .unwrap_or_else(|| "?s".to_string());
        turn_lines.push(format!(
            "{} processing turn (model/tools active) elapsed={elapsed}",
            state.spinner_glyph(),
        ));
        if let Some(silent_for) = turn_silence_duration(state) {
            if silent_for >= Duration::from_secs(4) {
                turn_lines.push(format!(
                    "runtime silent for {} (waiting for first model/tool event)",
                    format_duration_as_s(silent_for)
                ));
            }
        }
    } else {
        turn_lines.push(format!(
            "{} idle (awaiting prompt/runtime event)",
            state.heartbeat_glyph()
        ));
    }
    turn_lines.push(format!("phase: {}", state.turn_phase.label()));
    if !state.last_submitted_prompt.is_empty() {
        turn_lines.push(format!(
            "submitted[{}]: {}",
            state.submitted_turn_count, state.last_submitted_prompt
        ));
    }
    if !state.turn_status.is_empty() {
        turn_lines.push(state.turn_status.clone());
    }
    if !state.progress_status.is_empty() {
        turn_lines.push(state.progress_status.clone());
    }
    let memory_activity = recent_memory_activity_lines(state, 3);
    if memory_activity.is_empty() {
        turn_lines.push("memory: (none yet)".to_string());
    } else {
        turn_lines.push(format!("memory: {}", memory_activity.join(" | ")));
    }
    if turn_lines.is_empty() {
        turn_lines.push("waiting for first turn".to_string());
    }

    let assistant_lines = state.assistant_lines.iter().cloned().collect::<Vec<_>>();
    let timeline_lines = state.timeline_lines.iter().cloned().collect::<Vec<_>>();
    let mut tool_lines = state.integration_tools_lines.clone();
    if !state.integration_routines_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("---".to_string());
        }
        tool_lines.push("routines".to_string());
        tool_lines.extend(state.integration_routines_lines.clone());
    }
    if !state.integration_cortex_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("---".to_string());
        }
        tool_lines.push("cortex".to_string());
        tool_lines.extend(state.integration_cortex_lines.clone());
    }
    if !state.integration_memory_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("---".to_string());
        }
        tool_lines.push("memory".to_string());
        tool_lines.extend(state.integration_memory_lines.clone());
    }
    if !state.tool_lines.is_empty() {
        if !tool_lines.is_empty() {
            tool_lines.push("--- runtime tool activity ---".to_string());
        }
        tool_lines.extend(state.tool_lines.iter().cloned());
    }
    let mut event_lines = state.dashboard_lines.clone();
    if !state.event_lines.is_empty() {
        if !event_lines.is_empty() {
            event_lines.push("--- runtime events ---".to_string());
        }
        event_lines.extend(state.event_lines.iter().cloned());
    }
    let input_visible_width = shell_inner.width.saturating_sub(2) as usize;
    let (prompt_line, prompt_col) = input_prompt_view(pending_input, input_visible_width);
    let input_lines = vec![prompt_line];

    let compact = shell_inner.height < 34 || shell_inner.width < 104;
    let session_height = if compact { 3_u16 } else { 4_u16 };
    let turn_height = if compact { 4_u16 } else { 5_u16 };
    let input_height = 3_u16;
    let shortcuts_height = if state.show_shortcuts {
        if compact {
            5_u16
        } else {
            6_u16
        }
    } else {
        0_u16
    };
    let main_min = if compact { 10_u16 } else { 14_u16 };
    let fixed_total = session_height
        .saturating_add(turn_height)
        .saturating_add(input_height)
        .saturating_add(shortcuts_height);
    let main_height = shell_inner.height.saturating_sub(fixed_total).max(main_min);
    let top_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(session_height),
            Constraint::Length(turn_height),
            Constraint::Length(main_height),
            Constraint::Length(shortcuts_height),
            Constraint::Length(input_height),
        ])
        .split(shell_inner);

    let session_rect = top_area[0];
    let turn_rect = top_area[1];
    let main_rect = top_area[2];
    let shortcuts_rect = top_area[3];
    let input_rect = top_area[4];

    render_ratatui_panel(
        frame,
        session_rect,
        AgentPanel::Session,
        panel_header_label_with_count(state, AgentPanel::Session, session_lines.len()),
        &session_lines,
        state.panel_offset(AgentPanel::Session),
        state.panel_is_focused(AgentPanel::Session),
        state.tone_mode,
    );
    render_ratatui_panel(
        frame,
        turn_rect,
        AgentPanel::Turn,
        panel_header_label_with_count(state, AgentPanel::Turn, turn_lines.len()),
        &turn_lines,
        state.panel_offset(AgentPanel::Turn),
        state.panel_is_focused(AgentPanel::Turn),
        state.tone_mode,
    );

    if let Some(expanded_panel) = state.expanded_panel {
        let expanded_lines = match expanded_panel {
            AgentPanel::Session => &session_lines,
            AgentPanel::Turn => &turn_lines,
            AgentPanel::Assistant => &assistant_lines,
            AgentPanel::Timeline => &timeline_lines,
            AgentPanel::Tools => &tool_lines,
            AgentPanel::Events => &event_lines,
        };
        render_ratatui_panel(
            frame,
            main_rect,
            expanded_panel,
            panel_header_label_with_count(state, expanded_panel, expanded_lines.len()),
            expanded_lines,
            state.panel_offset(expanded_panel),
            state.panel_is_focused(expanded_panel),
            state.tone_mode,
        );
    } else if shell_inner.width >= 108 {
        let min_left = 42_usize;
        let min_right = if shell_inner.width >= 140 {
            40_usize
        } else {
            34_usize
        };
        let left_width = ((main_rect.width as usize * state.split_left_percent as usize) / 100)
            .clamp(
                min_left.min(main_rect.width.saturating_sub(1) as usize),
                main_rect.width.saturating_sub(min_right as u16) as usize,
            ) as u16;
        let right_width = main_rect.width.saturating_sub(left_width);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_width),
                Constraint::Length(right_width),
            ])
            .split(main_rect);
        let detail_top = ((columns[1].height as usize * state.split_top_percent as usize) / 100)
            .clamp(5, columns[1].height.saturating_sub(6) as usize) as u16;
        let detail_bottom = columns[1].height.saturating_sub(detail_top);
        let detail_stack = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(detail_top),
                Constraint::Length(detail_bottom),
            ])
            .split(columns[1]);
        let detail_bottom_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(detail_stack[1]);

        render_ratatui_panel(
            frame,
            columns[0],
            AgentPanel::Assistant,
            panel_header_label_with_count(state, AgentPanel::Assistant, assistant_lines.len()),
            &assistant_lines,
            state.panel_offset(AgentPanel::Assistant),
            state.panel_is_focused(AgentPanel::Assistant),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            detail_stack[0],
            AgentPanel::Timeline,
            panel_header_label_with_count(state, AgentPanel::Timeline, timeline_lines.len()),
            &timeline_lines,
            state.panel_offset(AgentPanel::Timeline),
            state.panel_is_focused(AgentPanel::Timeline),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            detail_bottom_split[0],
            AgentPanel::Tools,
            panel_header_label_with_count(state, AgentPanel::Tools, tool_lines.len()),
            &tool_lines,
            state.panel_offset(AgentPanel::Tools),
            state.panel_is_focused(AgentPanel::Tools),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            detail_bottom_split[1],
            AgentPanel::Events,
            panel_header_label_with_count(state, AgentPanel::Events, event_lines.len()),
            &event_lines,
            state.panel_offset(AgentPanel::Events),
            state.panel_is_focused(AgentPanel::Events),
            state.tone_mode,
        );
    } else if shell_inner.width >= 90 {
        let stack = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(22),
                Constraint::Percentage(18),
            ])
            .split(main_rect);
        let lower_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(stack[2]);
        render_ratatui_panel(
            frame,
            stack[0],
            AgentPanel::Assistant,
            panel_header_label_with_count(state, AgentPanel::Assistant, assistant_lines.len()),
            &assistant_lines,
            state.panel_offset(AgentPanel::Assistant),
            state.panel_is_focused(AgentPanel::Assistant),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            stack[1],
            AgentPanel::Timeline,
            panel_header_label_with_count(state, AgentPanel::Timeline, timeline_lines.len()),
            &timeline_lines,
            state.panel_offset(AgentPanel::Timeline),
            state.panel_is_focused(AgentPanel::Timeline),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            lower_split[0],
            AgentPanel::Tools,
            panel_header_label_with_count(state, AgentPanel::Tools, tool_lines.len()),
            &tool_lines,
            state.panel_offset(AgentPanel::Tools),
            state.panel_is_focused(AgentPanel::Tools),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            lower_split[1],
            AgentPanel::Events,
            panel_header_label_with_count(state, AgentPanel::Events, event_lines.len()),
            &event_lines,
            state.panel_offset(AgentPanel::Events),
            state.panel_is_focused(AgentPanel::Events),
            state.tone_mode,
        );
    } else {
        let stack = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(22),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
            ])
            .split(main_rect);
        render_ratatui_panel(
            frame,
            stack[0],
            AgentPanel::Assistant,
            panel_header_label_with_count(state, AgentPanel::Assistant, assistant_lines.len()),
            &assistant_lines,
            state.panel_offset(AgentPanel::Assistant),
            state.panel_is_focused(AgentPanel::Assistant),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            stack[1],
            AgentPanel::Timeline,
            panel_header_label_with_count(state, AgentPanel::Timeline, timeline_lines.len()),
            &timeline_lines,
            state.panel_offset(AgentPanel::Timeline),
            state.panel_is_focused(AgentPanel::Timeline),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            stack[2],
            AgentPanel::Tools,
            panel_header_label_with_count(state, AgentPanel::Tools, tool_lines.len()),
            &tool_lines,
            state.panel_offset(AgentPanel::Tools),
            state.panel_is_focused(AgentPanel::Tools),
            state.tone_mode,
        );
        render_ratatui_panel(
            frame,
            stack[3],
            AgentPanel::Events,
            panel_header_label_with_count(state, AgentPanel::Events, event_lines.len()),
            &event_lines,
            state.panel_offset(AgentPanel::Events),
            state.panel_is_focused(AgentPanel::Events),
            state.tone_mode,
        );
    }

    if state.show_shortcuts && shortcuts_rect.height > 0 {
        render_ratatui_panel(
            frame,
            shortcuts_rect,
            AgentPanel::Session,
            "  Shortcuts".to_string(),
            &shortcut_help_lines(),
            0,
            false,
            state.tone_mode,
        );
    }
    render_ratatui_panel(
        frame,
        input_rect,
        AgentPanel::Turn,
        "  Input".to_string(),
        &input_lines,
        0,
        false,
        state.tone_mode,
    );

    if input_rect.width > 3 && input_rect.height > 2 {
        let cursor_x = input_rect
            .x
            .saturating_add(1)
            .saturating_add(prompt_col.min(input_rect.width.saturating_sub(3)));
        let cursor_y = input_rect.y.saturating_add(1);
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn handle_ratatui_key_event(
    key: KeyEvent,
    app_state: &mut AgentAppState,
    pending_input: &mut String,
    child_stdin: &mut ChildStdin,
    child_pid: u32,
    quit_requested_at: &mut Option<Instant>,
    gateway_sync_tx: Option<&mpsc::Sender<GatewaySyncCommand>>,
) {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return;
    }
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    let control_char = if control {
        if let KeyCode::Char(ch) = key.code {
            Some(ch.to_ascii_lowercase())
        } else {
            None
        }
    } else {
        None
    };

    match (key.code, control_char) {
        (_, Some('c')) => {
            send_interrupt_to_child(child_pid);
            pending_input.clear();
        }
        (_, Some('d')) => {
            let _ = child_stdin.write_all(b"/quit\n");
            let _ = child_stdin.flush();
            if quit_requested_at.is_none() {
                *quit_requested_at = Some(Instant::now());
            }
        }
        (_, Some('n')) | (KeyCode::Tab, _) => app_state.focus_next_panel(),
        (_, Some('p')) | (KeyCode::BackTab, _) => app_state.focus_previous_panel(),
        (_, Some('b')) => app_state.scroll_focused_panel_up(),
        (_, Some('f')) => app_state.scroll_focused_panel_down(),
        (_, Some('g')) => app_state.toggle_tone_mode(),
        (_, Some('e')) => app_state.toggle_expand_focused_panel(),
        (KeyCode::Up, Some(_)) => app_state.adjust_split_top_percent(5),
        (KeyCode::Down, Some(_)) => app_state.adjust_split_top_percent(-5),
        (KeyCode::Left, Some(_)) => app_state.adjust_split_left_percent(-5),
        (KeyCode::Right, Some(_)) => app_state.adjust_split_left_percent(5),
        (KeyCode::Up, None) => app_state.scroll_focused_panel_up(),
        (KeyCode::Down, None) => app_state.scroll_focused_panel_down(),
        (KeyCode::Left, None) => app_state.focus_previous_panel(),
        (KeyCode::Right, None) => app_state.focus_next_panel(),
        (KeyCode::PageUp, _) => app_state.page_scroll_focused_panel_up(),
        (KeyCode::PageDown, _) => app_state.page_scroll_focused_panel_down(),
        (KeyCode::Home, _) => app_state.scroll_focused_panel_to_oldest(),
        (KeyCode::End, _) => app_state.scroll_focused_panel_to_latest(),
        (KeyCode::Backspace, _) => {
            pending_input.pop();
        }
        (_, Some('u')) => {
            pending_input.clear();
        }
        (KeyCode::Enter, _) => {
            record_submitted_prompt(app_state, pending_input);
            let local_command = parse_local_tui_command(pending_input.as_str());
            let submit_was_quit = is_local_quit_command(pending_input.as_str());
            if let Some(command) = local_command {
                handle_local_tui_command(app_state, command, gateway_sync_tx);
            } else {
                if !pending_input.is_empty() {
                    let _ = child_stdin.write_all(pending_input.as_bytes());
                }
                let _ = child_stdin.write_all(b"\n");
                let _ = child_stdin.flush();
                app_state.mark_turn_started();
            }
            pending_input.clear();
            if app_state.show_shortcuts {
                app_state.show_shortcuts = false;
                app_state.ui_prefs_dirty = true;
            }
            if submit_was_quit && quit_requested_at.is_none() {
                *quit_requested_at = Some(Instant::now());
            }
        }
        (KeyCode::Char('?'), _) if pending_input.is_empty() => {
            app_state.show_shortcuts = !app_state.show_shortcuts;
            app_state.ui_prefs_dirty = true;
        }
        (KeyCode::Char('['), None) if pending_input.is_empty() => {
            app_state.focus_previous_panel();
        }
        (KeyCode::Char(']'), None) if pending_input.is_empty() => {
            app_state.focus_next_panel();
        }
        (KeyCode::Char('k'), None) if pending_input.is_empty() => {
            app_state.scroll_focused_panel_up();
        }
        (KeyCode::Char('j'), None) if pending_input.is_empty() => {
            app_state.scroll_focused_panel_down();
        }
        (KeyCode::Char(ch), None) => {
            pending_input.push(ch);
            if app_state.show_shortcuts {
                app_state.show_shortcuts = false;
                app_state.ui_prefs_dirty = true;
            }
        }
        (_, Some('l')) => {
            // redraw on next frame
        }
        _ => {}
    }
}

struct RatatuiRuntimeHandles {
    child: Child,
    child_stdin: ChildStdin,
    rx: mpsc::Receiver<AgentRuntimeEvent>,
    stdout_handle: thread::JoinHandle<()>,
    stderr_handle: thread::JoinHandle<()>,
    gateway_sync_tx: mpsc::Sender<GatewaySyncCommand>,
    gateway_sync_handle: thread::JoinHandle<()>,
}

fn run_agent_ratatui(
    summary_lines: &[String],
    launch_command: &str,
    mut handles: RatatuiRuntimeHandles,
    prefs_path: &Path,
    default_request_budget_ms: Option<u64>,
) -> Result<(), String> {
    enable_raw_mode().map_err(|error| format!("failed to enable raw mode: {error}"))?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)
        .map_err(|error| format!("failed to enter alternate screen: {error}"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|error| format!("failed to create terminal: {error}"))?;

    let mut app_state = AgentAppState {
        default_request_budget_ms,
        ..AgentAppState::default()
    };
    if let Err(error) = load_tui_prefs(prefs_path, &mut app_state) {
        push_history_line(
            &mut app_state.event_lines,
            format!("prefs warning: {}", compact_ui_snippet(error.as_str(), 140)),
        );
    }
    let mut pending_input = String::new();
    let mut quit_requested_at: Option<Instant> = None;
    update_prompt_line(&mut app_state, pending_input.as_str());

    let result = loop {
        while let Ok(event) = handles.rx.try_recv() {
            match event {
                AgentRuntimeEvent::Output(source, line) => {
                    update_agent_app_state(&mut app_state, source, &line);
                }
                AgentRuntimeEvent::GatewaySync(snapshot) => {
                    apply_gateway_sync_snapshot(&mut app_state, snapshot);
                }
                AgentRuntimeEvent::InputByte(_) => {}
            }
        }
        app_state.tick_spinner();
        persist_tui_prefs_if_dirty(prefs_path, &mut app_state);
        update_prompt_line(&mut app_state, pending_input.as_str());
        terminal
            .draw(|frame| {
                draw_agent_ratatui(
                    frame,
                    summary_lines,
                    &app_state,
                    launch_command,
                    &pending_input,
                )
            })
            .map_err(|error| format!("failed to draw ratatui frame: {error}"))?;

        if let Some(status) = handles
            .child
            .try_wait()
            .map_err(|error| format!("failed to poll interactive runtime status: {error}"))?
        {
            let _ = handles.stdout_handle.join();
            let _ = handles.stderr_handle.join();
            while let Ok(event) = handles.rx.try_recv() {
                match event {
                    AgentRuntimeEvent::Output(source, line) => {
                        update_agent_app_state(&mut app_state, source, &line);
                    }
                    AgentRuntimeEvent::GatewaySync(snapshot) => {
                        apply_gateway_sync_snapshot(&mut app_state, snapshot);
                    }
                    AgentRuntimeEvent::InputByte(_) => {}
                }
            }
            update_prompt_line(&mut app_state, pending_input.as_str());
            let _ = terminal.draw(|frame| {
                draw_agent_ratatui(
                    frame,
                    summary_lines,
                    &app_state,
                    launch_command,
                    &pending_input,
                )
            });
            if status.success() {
                break Ok(());
            }
            break Err(format_runtime_exit_error(status, &app_state));
        }

        if event::poll(Duration::from_millis(40))
            .map_err(|error| format!("failed to poll terminal event: {error}"))?
        {
            match event::read()
                .map_err(|error| format!("failed to read terminal event: {error}"))?
            {
                CEvent::Key(key) => {
                    handle_ratatui_key_event(
                        key,
                        &mut app_state,
                        &mut pending_input,
                        &mut handles.child_stdin,
                        handles.child.id(),
                        &mut quit_requested_at,
                        Some(&handles.gateway_sync_tx),
                    );
                    persist_tui_prefs_if_dirty(prefs_path, &mut app_state);
                }
                CEvent::Resize(_, _) => {}
                _ => {}
            }
        }

        if let Some(requested_at) = quit_requested_at {
            if requested_at.elapsed() >= Duration::from_millis(AGENT_LOCAL_QUIT_GRACE_MS) {
                let _ = handles.child.kill();
                let _ = handles.child.wait();
                let _ = handles.stdout_handle.join();
                let _ = handles.stderr_handle.join();
                break Ok(());
            }
        }
    };

    persist_tui_prefs_if_dirty(prefs_path, &mut app_state);
    shutdown_gateway_sync_worker(&handles.gateway_sync_tx, handles.gateway_sync_handle);
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), Show, LeaveAlternateScreen);
    let _ = terminal.show_cursor();
    result
}

#[derive(Debug, Default)]
struct TerminalUiGuard {
    alternate_screen_enabled: bool,
    stty_state: Option<String>,
}

impl TerminalUiGuard {
    fn enter(enable_raw_mode: bool) -> Self {
        let mut guard = Self::default();
        if enable_raw_mode {
            let stty_state = Command::new("stty")
                .arg("-g")
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                .filter(|state| !state.is_empty());
            if let Some(state) = stty_state {
                let raw_status = Command::new("stty")
                    .args(["raw", "-echo", "-isig"])
                    .status()
                    .ok()
                    .filter(|status| status.success());
                let fallback_status = if raw_status.is_none() {
                    Command::new("stty")
                        .args(["-echo", "-icanon", "min", "1", "time", "0", "-isig"])
                        .status()
                        .ok()
                        .filter(|status| status.success())
                } else {
                    None
                };
                if raw_status.is_some() || fallback_status.is_some() {
                    guard.stty_state = Some(state);
                }
            }
        }

        if std::io::stdout().is_terminal() {
            print!("\x1b[?1049h\x1b[2J\x1b[H\x1b[?25h");
            let _ = std::io::stdout().flush();
            guard.alternate_screen_enabled = true;
        }
        guard
    }
}

impl Drop for TerminalUiGuard {
    fn drop(&mut self) {
        if let Some(state) = self.stty_state.as_deref() {
            let _ = Command::new("stty").arg(state).status();
        }
        if self.alternate_screen_enabled {
            print!("\x1b[?25h\x1b[?1049l");
            let _ = std::io::stdout().flush();
        }
    }
}

trait StringFallback {
    fn if_empty_then(self, fallback: String) -> String;
}

impl StringFallback for String {
    fn if_empty_then(self, fallback: String) -> String {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}

fn spawn_output_pump<R: Read + Send + 'static>(
    mut reader: R,
    source: AgentOutputSource,
    tx: mpsc::Sender<AgentRuntimeEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut pending = String::new();
        let mut buffer = [0_u8; 2048];
        loop {
            let read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };
            let chunk = String::from_utf8_lossy(&buffer[..read]);
            for ch in chunk.chars() {
                if ch == '\n' || ch == '\r' {
                    if !pending.trim().is_empty() {
                        let _ = tx.send(AgentRuntimeEvent::Output(source, pending.clone()));
                    }
                    pending.clear();
                } else {
                    pending.push(ch);
                }
            }
        }
        if !pending.trim().is_empty() {
            let _ = tx.send(AgentRuntimeEvent::Output(source, pending));
        }
    })
}

fn spawn_input_pump(tx: mpsc::Sender<AgentRuntimeEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        let mut buffer = [0_u8; 1];
        loop {
            let read = match stdin.read(&mut buffer) {
                Ok(0) => {
                    let _ = tx.send(AgentRuntimeEvent::InputByte(4));
                    break;
                }
                Ok(read) => read,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            };
            if read > 0 {
                let _ = tx.send(AgentRuntimeEvent::InputByte(buffer[0]));
            }
        }
    })
}

fn send_interrupt_to_child(child_pid: u32) {
    let _ = Command::new("kill")
        .arg("-INT")
        .arg(child_pid.to_string())
        .status();
}

fn update_prompt_line(state: &mut AgentAppState, pending_input: &str) {
    state.prompt_line = format!("tau> {pending_input}");
}

fn is_local_quit_command(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.eq_ignore_ascii_case("/quit") || trimmed.eq_ignore_ascii_case("quit")
}

fn compact_ui_snippet(value: &str, max_chars: usize) -> String {
    let mut snippet = String::new();
    for ch in value.chars() {
        if snippet.chars().count() >= max_chars {
            snippet.push('…');
            return snippet;
        }
        snippet.push(ch);
    }
    snippet
}

fn render_tools_metrics_table(
    runtime_events: Option<u64>,
    runtime_invalid: Option<u64>,
    ui_tools_events: Option<u64>,
    ui_tools_invalid: Option<u64>,
    ui_all_events: u64,
) -> Vec<String> {
    fn classify_status(events: Option<u64>, invalid: Option<u64>) -> &'static str {
        match (events, invalid) {
            (None, _) => "n/a",
            (Some(event_count), None) => {
                if event_count == 0 {
                    "idle"
                } else {
                    "ok"
                }
            }
            (Some(event_count), Some(invalid_count)) => {
                if invalid_count == 0 {
                    if event_count == 0 {
                        "idle"
                    } else {
                        "ok"
                    }
                } else if event_count == 0 || invalid_count.saturating_mul(2) >= event_count {
                    "bad"
                } else {
                    "warn"
                }
            }
        }
    }

    fn fmt_count(value: Option<u64>) -> String {
        value
            .map(|count| count.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    vec![
        "tools metrics".to_string(),
        format!(
            "{:<11} {:>7} {:>7} {:<7} {}",
            "source", "events", "invalid", "status", "note"
        ),
        format!(
            "{:<11} {:>7} {:>7} {:<7} {}",
            "runtime",
            fmt_count(runtime_events),
            fmt_count(runtime_invalid),
            classify_status(runtime_events, runtime_invalid),
            "session tool results"
        ),
        format!(
            "{:<11} {:>7} {:>7} {:<7} {}",
            "ui-tools",
            fmt_count(ui_tools_events),
            fmt_count(ui_tools_invalid),
            classify_status(ui_tools_events, ui_tools_invalid),
            "tools view telemetry"
        ),
        format!(
            "{:<11} {:>7} {:>7} {:<7} {}",
            "ui-all",
            ui_all_events,
            "-",
            classify_status(Some(ui_all_events), None),
            "all dashboard views"
        ),
    ]
}

fn append_memory_recent_write_lines(lines: &mut Vec<String>, status_payload: &serde_json::Value) {
    let recent_writes = status_payload
        .pointer("/gateway/web_ui/memory_distill_runtime/recent_writes")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if recent_writes.is_empty() {
        lines.push("memory distill recent_writes=(none)".to_string());
        return;
    }

    for write in recent_writes.iter().rev().take(3) {
        lines.push(format_memory_recent_write_line(write));
    }
}

fn format_memory_recent_write_line(write: &serde_json::Value) -> String {
    let summary = write
        .get("summary")
        .and_then(serde_json::Value::as_str)
        .map(|value| compact_ui_snippet(value, 80))
        .unwrap_or_else(|| "(summary unavailable)".to_string());
    let session = write
        .get("session_key")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let entry = write
        .get("entry_id")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let memory_type = write
        .get("memory_type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    format!(
        "memory distill write {} session={} entry={} summary={}",
        memory_type, session, entry, summary
    )
}

fn parse_local_tui_command(input: &str) -> Option<LocalTuiCommand> {
    let normalized = input.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "/help" | "help" => Some(LocalTuiCommand::Help),
        "/status" => Some(LocalTuiCommand::Status),
        "/dashboard" => Some(LocalTuiCommand::Dashboard),
        "/tools" => Some(LocalTuiCommand::Tools),
        "/routines" => Some(LocalTuiCommand::Routines),
        "/cortex" => Some(LocalTuiCommand::Cortex),
        "/memory" | "/memory-distill" => Some(LocalTuiCommand::Memory),
        "/sync" => Some(LocalTuiCommand::Sync),
        "/colors" | "/color" => Some(LocalTuiCommand::Colors),
        _ => None,
    }
}

fn resolve_gateway_sync_base_url() -> String {
    env::var("TAU_TUI_GATEWAY_BASE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_TUI_GATEWAY_BASE_URL.to_string())
}

fn resolve_gateway_sync_auth_token() -> Option<String> {
    for key in [
        "TAU_UNIFIED_AUTH_TOKEN",
        "TAU_GATEWAY_BEARER_TOKEN",
        "TAU_OPS_TOKEN",
    ] {
        if let Ok(value) = env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn resolve_gateway_endpoint_url(base_url: &str, endpoint_or_url: &str) -> String {
    let trimmed = endpoint_or_url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    if trimmed.starts_with('/') {
        return format!("{base_url}{trimmed}");
    }
    format!("{base_url}/{trimmed}")
}

fn json_pointer_str(payload: &serde_json::Value, pointer: &str) -> Option<String> {
    payload
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn json_pointer_u64(payload: &serde_json::Value, pointer: &str) -> Option<u64> {
    payload.pointer(pointer).and_then(serde_json::Value::as_u64)
}

fn json_pointer_bool(payload: &serde_json::Value, pointer: &str) -> Option<bool> {
    payload
        .pointer(pointer)
        .and_then(serde_json::Value::as_bool)
}

fn json_pointer_string_list(payload: &serde_json::Value, pointer: &str) -> Vec<String> {
    payload
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn fetch_gateway_json(url: &str, auth_token: Option<&str>) -> Result<serde_json::Value, String> {
    const STATUS_MARKER: &str = "__TAU_HTTP_STATUS__:";
    let mut command = Command::new("curl");
    command
        .arg("-sS")
        .arg("-L")
        .arg("--max-time")
        .arg(GATEWAY_SYNC_CURL_TIMEOUT_SECONDS.to_string())
        .arg("--connect-timeout")
        .arg("1")
        .arg("-H")
        .arg("Accept: application/json")
        .arg("-w")
        .arg(format!("\n{STATUS_MARKER}%{{http_code}}"));
    if let Some(token) = auth_token {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            command
                .arg("-H")
                .arg(format!("Authorization: Bearer {trimmed}"));
        }
    }
    command.arg(url);

    let output = command
        .output()
        .map_err(|error| format!("curl failed for {url}: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("curl failed for {url} with status {}", output.status)
        } else {
            format!("curl failed for {url}: {stderr}")
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let Some((body, status_raw)) = stdout.rsplit_once(STATUS_MARKER) else {
        return Err(format!("curl response missing status marker for {url}"));
    };
    let status = status_raw.trim().parse::<u16>().unwrap_or(0);
    if !(200..300).contains(&status) {
        let body_snippet = compact_ui_snippet(body.trim(), 240);
        return Err(format!(
            "http {status} from {url}: {}",
            body_snippet.if_empty_then("(empty body)".to_string())
        ));
    }
    serde_json::from_str::<serde_json::Value>(body.trim()).map_err(|error| {
        format!(
            "invalid json from {url}: {error} body={}",
            compact_ui_snippet(body.trim(), 180)
        )
    })
}

fn sync_error_is_rate_limited(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("http 429")
        || normalized.contains("rate_limited")
        || normalized.contains("gateway rate limit exceeded")
}

fn gateway_sync_full_refresh_seconds() -> u64 {
    (GATEWAY_SYNC_INTERVAL_MS.saturating_mul(GATEWAY_SYNC_FULL_REFRESH_EVERY_LIGHT_CYCLES))
        .saturating_div(1000)
        .max(1)
}

fn gateway_sync_backoff_wait_ms(consecutive_rate_limit_hits: u8) -> u64 {
    let exponent = consecutive_rate_limit_hits.saturating_sub(1).min(3);
    GATEWAY_SYNC_BACKOFF_INTERVAL_MS
        .saturating_mul(1_u64 << exponent)
        .min(GATEWAY_SYNC_BACKOFF_MAX_INTERVAL_MS)
}

fn collect_gateway_sync_snapshot(
    base_url: &str,
    auth_token: Option<&str>,
    mode: GatewaySyncFetchMode,
) -> GatewaySyncSnapshot {
    let status_url = resolve_gateway_endpoint_url(base_url, "/gateway/status");
    let status_payload = match fetch_gateway_json(status_url.as_str(), auth_token) {
        Ok(payload) => payload,
        Err(error) => {
            let rate_limited = sync_error_is_rate_limited(error.as_str());
            return GatewaySyncSnapshot {
                status_line: format!(
                    "gateway sync failed ({})",
                    compact_ui_snippet(error.as_str(), 96)
                ),
                dashboard_lines: vec![
                    "dashboard unavailable".to_string(),
                    compact_ui_snippet(error.as_str(), 180),
                ],
                tools_lines: vec!["tools unavailable".to_string()],
                routines_lines: vec!["routines unavailable".to_string()],
                cortex_lines: vec!["cortex unavailable".to_string()],
                memory_lines: vec!["memory unavailable".to_string()],
                event_line: format!(
                    "integration.sync failed mode={} source={} {}",
                    match mode {
                        GatewaySyncFetchMode::Light => "light",
                        GatewaySyncFetchMode::Full => "full",
                    },
                    status_url,
                    error
                ),
                success: false,
                rate_limited,
            };
        }
    };

    let mut degraded = false;
    let mut rate_limited = false;
    let service_status = json_pointer_str(&status_payload, "/service/service_status")
        .or_else(|| json_pointer_str(&status_payload, "/service/status"))
        .unwrap_or_else(|| "unknown".to_string());
    let multi_channel_state = json_pointer_str(&status_payload, "/multi_channel/health_state")
        .unwrap_or_else(|| "unknown".to_string());
    let reason_code = json_pointer_str(&status_payload, "/events/reason_code")
        .unwrap_or_else(|| "unknown".to_string());
    let heartbeat_state = json_pointer_str(&status_payload, "/runtime_heartbeat/run_state")
        .unwrap_or_else(|| "unknown".to_string());
    let training_run_state = json_pointer_str(&status_payload, "/training/run_state")
        .unwrap_or_else(|| "unknown".to_string());
    let training_rollouts_total =
        json_pointer_u64(&status_payload, "/training/rollouts_total").unwrap_or_default();
    let queue_depth = json_pointer_u64(&status_payload, "/events/queue_depth").unwrap_or_default();

    let dashboard_health_endpoint =
        json_pointer_str(&status_payload, "/gateway/dashboard/health_endpoint")
            .unwrap_or_else(|| "/dashboard/health".to_string());
    let dashboard_widgets_endpoint =
        json_pointer_str(&status_payload, "/gateway/dashboard/widgets_endpoint")
            .unwrap_or_else(|| "/dashboard/widgets".to_string());
    let dashboard_alerts_endpoint =
        json_pointer_str(&status_payload, "/gateway/dashboard/alerts_endpoint")
            .unwrap_or_else(|| "/dashboard/alerts".to_string());
    let tools_endpoint = json_pointer_str(&status_payload, "/gateway/web_ui/tools_endpoint")
        .unwrap_or_else(|| "/gateway/tools".to_string());
    let tool_stats_endpoint =
        json_pointer_str(&status_payload, "/gateway/web_ui/tool_stats_endpoint")
            .unwrap_or_else(|| "/gateway/tools/stats".to_string());
    let jobs_endpoint = json_pointer_str(&status_payload, "/gateway/web_ui/jobs_endpoint")
        .unwrap_or_else(|| "/gateway/jobs".to_string());
    let cortex_chat_endpoint =
        json_pointer_str(&status_payload, "/gateway/web_ui/cortex_chat_endpoint")
            .unwrap_or_else(|| "/cortex/chat".to_string());
    let cortex_status_endpoint =
        json_pointer_str(&status_payload, "/gateway/web_ui/cortex_status_endpoint")
            .unwrap_or_else(|| "/cortex/status".to_string());

    let mut dashboard_lines = Vec::new();
    dashboard_lines.push(format!(
        "dashboard run_state={training_run_state} rollouts_total={training_rollouts_total} queue_depth={queue_depth}"
    ));
    dashboard_lines.push(format!(
        "dashboard endpoints health={} widgets={} alerts={}",
        dashboard_health_endpoint, dashboard_widgets_endpoint, dashboard_alerts_endpoint
    ));

    let mut tools_lines = Vec::new();
    let telemetry_total = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/telemetry_runtime/total_events",
    )
    .unwrap_or_default();
    tools_lines.push(format!(
        "tools endpoints inventory={} stats={}",
        tools_endpoint, tool_stats_endpoint
    ));
    let mut runtime_tool_events: Option<u64> = None;
    let mut runtime_tool_invalid: Option<u64> = None;
    let mut ui_tools_events: Option<u64> = None;
    let mut ui_tools_invalid: Option<u64> = None;

    let mut routines_lines = Vec::new();
    routines_lines.push(format!(
        "routines health={multi_channel_state} reason={reason_code} queue_depth={queue_depth}"
    ));
    routines_lines.push(format!("routines jobs endpoint={jobs_endpoint}"));

    let mut cortex_lines = Vec::new();
    cortex_lines.push(format!(
        "cortex endpoints chat={} status={}",
        cortex_chat_endpoint, cortex_status_endpoint
    ));

    let mut memory_lines = Vec::new();
    let memory_distill_enabled = json_pointer_bool(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/enabled",
    )
    .unwrap_or(false);
    let memory_distill_in_flight = json_pointer_bool(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/in_flight",
    )
    .unwrap_or(false);
    let memory_distill_cycles = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/cycle_count",
    )
    .unwrap_or_default();
    let memory_distill_writes = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/writes_applied",
    )
    .unwrap_or_default();
    let memory_distill_failures = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/write_failures",
    )
    .unwrap_or_default();
    let memory_last_cycle_sessions = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/last_cycle_sessions_scanned",
    )
    .unwrap_or_default();
    let memory_last_cycle_entries = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/last_cycle_entries_scanned",
    )
    .unwrap_or_default();
    let memory_last_cycle_candidates = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/last_cycle_candidates_extracted",
    )
    .unwrap_or_default();
    let memory_last_cycle_writes = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/last_cycle_writes_applied",
    )
    .unwrap_or_default();
    let memory_last_cycle_write_failures = json_pointer_u64(
        &status_payload,
        "/gateway/web_ui/memory_distill_runtime/last_cycle_write_failures",
    )
    .unwrap_or_default();
    memory_lines.push(format!(
        "memory distill enabled={} in_flight={} cycles={} writes={} write_failures={}",
        memory_distill_enabled,
        memory_distill_in_flight,
        memory_distill_cycles,
        memory_distill_writes,
        memory_distill_failures
    ));
    memory_lines.push(format!(
        "memory distill last_cycle sessions={} entries={} candidates={} writes={} write_failures={}",
        memory_last_cycle_sessions,
        memory_last_cycle_entries,
        memory_last_cycle_candidates,
        memory_last_cycle_writes,
        memory_last_cycle_write_failures
    ));
    append_memory_recent_write_lines(&mut memory_lines, &status_payload);
    if matches!(mode, GatewaySyncFetchMode::Light) {
        let full_refresh_seconds = gateway_sync_full_refresh_seconds();
        dashboard_lines.push(format!(
            "dashboard detail mode=light (full refresh every {}s or /sync)",
            full_refresh_seconds
        ));
        tools_lines.push(format!(
            "tools detail mode=light (full refresh every {}s or /sync)",
            full_refresh_seconds
        ));
        routines_lines.push(format!(
            "routines detail mode=light (full refresh every {}s or /sync)",
            full_refresh_seconds
        ));
        cortex_lines.push(format!(
            "cortex detail mode=light (full refresh every {}s or /sync)",
            full_refresh_seconds
        ));
        memory_lines.push(format!(
            "memory detail mode=light (full refresh every {}s or /sync)",
            full_refresh_seconds
        ));
        let mut tools_metric_lines =
            render_tools_metrics_table(None, None, None, None, telemetry_total);
        tools_metric_lines.append(&mut tools_lines);
        tools_lines = tools_metric_lines;
    } else {
        let dashboard_health = fetch_gateway_json(
            resolve_gateway_endpoint_url(base_url, dashboard_health_endpoint.as_str()).as_str(),
            auth_token,
        );
        let dashboard_widgets = fetch_gateway_json(
            resolve_gateway_endpoint_url(base_url, dashboard_widgets_endpoint.as_str()).as_str(),
            auth_token,
        );
        let dashboard_alerts = fetch_gateway_json(
            resolve_gateway_endpoint_url(base_url, dashboard_alerts_endpoint.as_str()).as_str(),
            auth_token,
        );
        let tools_inventory = fetch_gateway_json(
            resolve_gateway_endpoint_url(base_url, tools_endpoint.as_str()).as_str(),
            auth_token,
        );
        let tools_stats = fetch_gateway_json(
            resolve_gateway_endpoint_url(base_url, tool_stats_endpoint.as_str()).as_str(),
            auth_token,
        );
        let jobs = fetch_gateway_json(
            resolve_gateway_endpoint_url(base_url, jobs_endpoint.as_str()).as_str(),
            auth_token,
        );
        let cortex_status = fetch_gateway_json(
            resolve_gateway_endpoint_url(base_url, cortex_status_endpoint.as_str()).as_str(),
            auth_token,
        );

        match dashboard_health {
            Ok(payload) => {
                let health_state = json_pointer_str(&payload, "/health/state")
                    .unwrap_or_else(|| "unknown".to_string());
                let control_mode = json_pointer_str(&payload, "/control/mode")
                    .unwrap_or_else(|| "unknown".to_string());
                let run_state = json_pointer_str(&payload, "/training/run_state")
                    .unwrap_or_else(|| "unknown".to_string());
                let queue_depth =
                    json_pointer_u64(&payload, "/health/queue_depth").unwrap_or_default();
                dashboard_lines.push(format!(
                    "dashboard health={health_state} control={control_mode} run_state={run_state} queue={queue_depth}"
                ));
            }
            Err(error) => {
                degraded = true;
                rate_limited |= sync_error_is_rate_limited(error.as_str());
                dashboard_lines.push(format!(
                    "dashboard health unavailable: {}",
                    compact_ui_snippet(error.as_str(), 140)
                ));
            }
        }
        match (dashboard_widgets, dashboard_alerts) {
            (Ok(widgets_payload), Ok(alerts_payload)) => {
                let widget_count = widgets_payload
                    .pointer("/widgets")
                    .and_then(serde_json::Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0);
                let alert_count = alerts_payload
                    .pointer("/alerts")
                    .and_then(serde_json::Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0);
                dashboard_lines.push(format!(
                    "dashboard widgets={widget_count} alerts={alert_count}"
                ));
                if let Some(first_alert) = alerts_payload
                    .pointer("/alerts")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|entries| entries.first())
                {
                    let code = first_alert
                        .get("code")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("unknown");
                    let severity = first_alert
                        .get("severity")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("unknown");
                    dashboard_lines.push(format!("dashboard top_alert={code} severity={severity}"));
                }
            }
            (widgets_result, alerts_result) => {
                degraded = true;
                let widgets_status = widgets_result.err().unwrap_or_else(|| "ok".to_string());
                let alerts_status = alerts_result.err().unwrap_or_else(|| "ok".to_string());
                rate_limited |= sync_error_is_rate_limited(widgets_status.as_str());
                rate_limited |= sync_error_is_rate_limited(alerts_status.as_str());
                dashboard_lines.push(format!(
                    "dashboard widgets/alerts partial: widgets={} alerts={}",
                    compact_ui_snippet(widgets_status.as_str(), 80),
                    compact_ui_snippet(alerts_status.as_str(), 80)
                ));
            }
        }

        match tools_inventory {
            Ok(payload) => {
                let total_tools = json_pointer_u64(&payload, "/total_tools").unwrap_or_default();
                tools_lines.push(format!("tools inventory total_tools={total_tools}"));
                if let Some(tool_entries) = payload
                    .pointer("/tools")
                    .and_then(serde_json::Value::as_array)
                {
                    let names = tool_entries
                        .iter()
                        .take(6)
                        .filter_map(|entry| entry.get("name").and_then(serde_json::Value::as_str))
                        .collect::<Vec<_>>();
                    if names.is_empty() {
                        tools_lines.push("tools listed: (none)".to_string());
                    } else {
                        tools_lines.push(format!("tools listed: {}", names.join(", ")));
                    }
                }
            }
            Err(error) => {
                degraded = true;
                rate_limited |= sync_error_is_rate_limited(error.as_str());
                tools_lines.push(format!(
                    "tools inventory unavailable: {}",
                    compact_ui_snippet(error.as_str(), 140)
                ));
            }
        }
        match tools_stats {
            Ok(payload) => {
                let total_events = json_pointer_u64(&payload, "/total_events").unwrap_or_default();
                let invalid_records =
                    json_pointer_u64(&payload, "/invalid_records").unwrap_or_default();
                let ui_total_events =
                    json_pointer_u64(&payload, "/ui_total_events").unwrap_or_default();
                let ui_invalid_records =
                    json_pointer_u64(&payload, "/ui_invalid_records").unwrap_or_default();
                runtime_tool_events = Some(total_events);
                runtime_tool_invalid = Some(invalid_records);
                ui_tools_events = Some(ui_total_events);
                ui_tools_invalid = Some(ui_invalid_records);
                let diagnostics = json_pointer_string_list(&payload, "/diagnostics");
                let ui_diagnostics = json_pointer_string_list(&payload, "/ui_diagnostics");
                if !diagnostics.is_empty() {
                    tools_lines.push(format!(
                        "tools diagnostics={}",
                        diagnostics
                            .iter()
                            .take(2)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ));
                }
                if !ui_diagnostics.is_empty() {
                    tools_lines.push(format!(
                        "tools ui diagnostics={}",
                        ui_diagnostics
                            .iter()
                            .take(2)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ));
                }
                if let Some(stats_entries) = payload
                    .pointer("/stats")
                    .and_then(serde_json::Value::as_array)
                {
                    if let Some(top) = stats_entries.iter().max_by_key(|entry| {
                        entry
                            .get("event_count")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or_default()
                    }) {
                        let name = top
                            .get("tool_name")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("unknown");
                        let event_count = top
                            .get("event_count")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or_default();
                        if event_count > 0 {
                            tools_lines
                                .push(format!("tools top_usage={name} events={event_count}"));
                        } else if invalid_records > 0 {
                            tools_lines.push(
                                "tools top_usage=(none yet; session stats has malformed records)"
                                    .to_string(),
                            );
                        } else {
                            tools_lines.push("tools top_usage=(none yet)".to_string());
                        }
                    }
                }
            }
            Err(error) => {
                degraded = true;
                rate_limited |= sync_error_is_rate_limited(error.as_str());
                tools_lines.push(format!(
                    "tools stats unavailable: {}",
                    compact_ui_snippet(error.as_str(), 140)
                ));
            }
        }
        let mut tools_metric_lines = render_tools_metrics_table(
            runtime_tool_events,
            runtime_tool_invalid,
            ui_tools_events,
            ui_tools_invalid,
            telemetry_total,
        );
        tools_metric_lines.append(&mut tools_lines);
        tools_lines = tools_metric_lines;

        match jobs {
            Ok(payload) => {
                let total_jobs = json_pointer_u64(&payload, "/total_jobs").unwrap_or_default();
                let running_jobs = payload
                    .pointer("/jobs")
                    .and_then(serde_json::Value::as_array)
                    .map(|entries| {
                        entries
                            .iter()
                            .filter(|entry| {
                                entry
                                    .get("status")
                                    .and_then(serde_json::Value::as_str)
                                    .map(|status| status.eq_ignore_ascii_case("running"))
                                    .unwrap_or(false)
                            })
                            .count()
                    })
                    .unwrap_or(0);
                routines_lines.push(format!(
                    "routines jobs total={total_jobs} running={running_jobs}"
                ));
                if let Some(job_entries) = payload
                    .pointer("/jobs")
                    .and_then(serde_json::Value::as_array)
                {
                    for job in job_entries.iter().take(3) {
                        let job_id = job
                            .get("job_id")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("unknown");
                        let status = job
                            .get("status")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("unknown");
                        routines_lines.push(format!("routines job {job_id} status={status}"));
                    }
                }
            }
            Err(error) => {
                degraded = true;
                rate_limited |= sync_error_is_rate_limited(error.as_str());
                routines_lines.push(format!(
                    "routines jobs unavailable: {}",
                    compact_ui_snippet(error.as_str(), 140)
                ));
            }
        }

        let memory_distill_reason_codes = json_pointer_string_list(
            &status_payload,
            "/gateway/web_ui/memory_distill_runtime/last_reason_codes",
        );
        if memory_distill_reason_codes.is_empty() {
            memory_lines.push("memory distill reason_codes=(none)".to_string());
        } else {
            memory_lines.push(format!(
                "memory distill reason_codes={}",
                memory_distill_reason_codes
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        match cortex_status {
            Ok(payload) => {
                let health_state = json_pointer_str(&payload, "/health_state")
                    .unwrap_or_else(|| "unknown".to_string());
                let reason_code = json_pointer_str(&payload, "/reason_code")
                    .unwrap_or_else(|| "unknown".to_string());
                let rollout_gate = json_pointer_str(&payload, "/rollout_gate")
                    .unwrap_or_else(|| "unknown".to_string());
                cortex_lines.push(format!(
                    "cortex health={} rollout_gate={} reason={}",
                    health_state, rollout_gate, reason_code
                ));
                let diagnostics = json_pointer_string_list(&payload, "/diagnostics");
                if diagnostics.is_empty() {
                    cortex_lines.push("cortex diagnostics=(none)".to_string());
                } else {
                    cortex_lines.push(format!(
                        "cortex diagnostics={}",
                        diagnostics
                            .iter()
                            .take(3)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ));
                }
            }
            Err(error) => {
                degraded = true;
                rate_limited |= sync_error_is_rate_limited(error.as_str());
                cortex_lines.push(format!(
                    "cortex status unavailable: {}",
                    compact_ui_snippet(error.as_str(), 140)
                ));
            }
        }
    }

    GatewaySyncSnapshot {
        status_line: format!(
            "gateway service={service_status} multi_channel={multi_channel_state} heartbeat={heartbeat_state} mode={}",
            match mode {
                GatewaySyncFetchMode::Light => "light",
                GatewaySyncFetchMode::Full => "full",
            }
        ),
        dashboard_lines,
        tools_lines,
        routines_lines,
        cortex_lines,
        memory_lines,
        event_line: if degraded {
            if rate_limited {
                "integration.sync completed with partial failures (rate-limited; backing off polling)".to_string()
            } else {
                "integration.sync completed with partial failures (dashboard/tools/routines)"
                    .to_string()
            }
        } else if rate_limited {
            "integration.sync rate-limited; backing off polling".to_string()
        } else {
            "integration.sync completed (dashboard/tools/routines)".to_string()
        },
        success: !degraded,
        rate_limited,
    }
}

fn spawn_gateway_sync_pump(
    tx: mpsc::Sender<AgentRuntimeEvent>,
) -> (mpsc::Sender<GatewaySyncCommand>, thread::JoinHandle<()>) {
    let (command_tx, command_rx) = mpsc::channel::<GatewaySyncCommand>();
    let base_url = resolve_gateway_sync_base_url();
    let auth_token = resolve_gateway_sync_auth_token();
    let handle = thread::spawn(move || {
        let mut mode = GatewaySyncFetchMode::Full;
        let mut light_cycles_since_full = 0_u64;
        let mut consecutive_rate_limit_hits = 0_u8;
        loop {
            let snapshot =
                collect_gateway_sync_snapshot(base_url.as_str(), auth_token.as_deref(), mode);
            let rate_limited = snapshot.rate_limited;
            let _ = tx.send(AgentRuntimeEvent::GatewaySync(snapshot));

            if rate_limited {
                consecutive_rate_limit_hits = consecutive_rate_limit_hits.saturating_add(1);
            } else {
                consecutive_rate_limit_hits = 0;
            }

            if matches!(mode, GatewaySyncFetchMode::Full) {
                light_cycles_since_full = 0;
            } else {
                light_cycles_since_full = light_cycles_since_full.saturating_add(1);
            }

            let wait_ms = if rate_limited {
                gateway_sync_backoff_wait_ms(consecutive_rate_limit_hits)
            } else {
                GATEWAY_SYNC_INTERVAL_MS
            };

            match command_rx.recv_timeout(Duration::from_millis(wait_ms)) {
                Ok(GatewaySyncCommand::RefreshNow) => {
                    mode = GatewaySyncFetchMode::Full;
                    consecutive_rate_limit_hits = 0;
                }
                Ok(GatewaySyncCommand::Shutdown) => {
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    mode = if light_cycles_since_full
                        >= GATEWAY_SYNC_FULL_REFRESH_EVERY_LIGHT_CYCLES
                    {
                        GatewaySyncFetchMode::Full
                    } else {
                        GatewaySyncFetchMode::Light
                    };
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    });
    (command_tx, handle)
}

fn shutdown_gateway_sync_worker(
    command_tx: &mpsc::Sender<GatewaySyncCommand>,
    handle: thread::JoinHandle<()>,
) {
    let _ = command_tx.send(GatewaySyncCommand::Shutdown);
    let _ = handle.join();
}

fn apply_gateway_sync_snapshot(state: &mut AgentAppState, snapshot: GatewaySyncSnapshot) {
    state.gateway_sync_ok = snapshot.success;
    state.gateway_sync_status = snapshot.status_line;
    state.gateway_sync_last_at = Some(Instant::now());
    state.dashboard_lines = snapshot.dashboard_lines;
    state.integration_tools_lines = snapshot.tools_lines;
    state.integration_routines_lines = snapshot.routines_lines;
    state.integration_cortex_lines = snapshot.cortex_lines;
    state.integration_memory_lines = snapshot.memory_lines.clone();
    if let Some(summary) = snapshot
        .memory_lines
        .iter()
        .find(|line| line.starts_with("memory distill enabled="))
        .or_else(|| {
            snapshot
                .memory_lines
                .iter()
                .find(|line| line.starts_with("memory distill reason_codes="))
        })
    {
        let compact = compact_ui_snippet(summary.as_str(), 120);
        if compact != state.last_memory_activity_summary {
            state.push_memory_activity(format!("sync {compact}"));
            state.last_memory_activity_summary = compact;
        }
    }
    let should_append_event = state
        .event_lines
        .back()
        .map(|line| line != &snapshot.event_line)
        .unwrap_or(true);
    if should_append_event {
        push_history_line(&mut state.event_lines, snapshot.event_line.clone());
    }
    if !snapshot.success && should_append_event {
        push_history_line(
            &mut state.timeline_lines,
            format!("integration warning: {}", snapshot.event_line),
        );
    }
}

fn emit_local_command_help(state: &mut AgentAppState) {
    for line in [
        "local commands:",
        "  /status      show latest gateway sync status",
        "  /dashboard   show latest dashboard snapshot",
        "  /tools       show latest tools inventory/telemetry snapshot",
        "  /routines    show latest routines/jobs snapshot",
        "  /cortex      show latest cortex health/diagnostics snapshot",
        "  /memory      show latest memory-distill runtime snapshot (/memory-distill alias)",
        "  /sync        force gateway sync refresh now",
        "  /colors      toggle color mode (semantic/minimal)",
    ] {
        push_history_line(&mut state.assistant_lines, line.to_string());
    }
}

fn emit_snapshot_to_assistant(state: &mut AgentAppState, title: &str, lines: &[String]) {
    push_history_line(&mut state.assistant_lines, title.to_string());
    if lines.is_empty() {
        push_history_line(&mut state.assistant_lines, "  (none)".to_string());
        return;
    }
    for line in lines {
        push_history_line(&mut state.assistant_lines, format!("  {line}"));
    }
}

fn handle_local_tui_command(
    state: &mut AgentAppState,
    command: LocalTuiCommand,
    gateway_sync_tx: Option<&mpsc::Sender<GatewaySyncCommand>>,
) {
    state.turn_in_progress = false;
    state.turn_phase = TurnPhase::Idle;
    state.turn_status = "local command handled by TUI".to_string();
    state.progress_status.clear();
    match command {
        LocalTuiCommand::Help => {
            emit_local_command_help(state);
        }
        LocalTuiCommand::Status => {
            let sync_age = format_runtime_age(state.gateway_sync_age());
            push_history_line(
                &mut state.assistant_lines,
                format!(
                    "gateway sync: {} | ok={} | age={sync_age}",
                    state.gateway_sync_status, state.gateway_sync_ok
                ),
            );
            let dashboard_lines = state.dashboard_lines.clone();
            let tools_lines = state.integration_tools_lines.clone();
            let routines_lines = state.integration_routines_lines.clone();
            let cortex_lines = state.integration_cortex_lines.clone();
            let memory_lines = state.integration_memory_lines.clone();
            emit_snapshot_to_assistant(state, "dashboard", &dashboard_lines);
            emit_snapshot_to_assistant(state, "tools", &tools_lines);
            emit_snapshot_to_assistant(state, "routines", &routines_lines);
            emit_snapshot_to_assistant(state, "cortex", &cortex_lines);
            emit_snapshot_to_assistant(state, "memory", &memory_lines);
        }
        LocalTuiCommand::Dashboard => {
            let dashboard_lines = state.dashboard_lines.clone();
            emit_snapshot_to_assistant(state, "dashboard", &dashboard_lines);
        }
        LocalTuiCommand::Tools => {
            let tools_lines = state.integration_tools_lines.clone();
            emit_snapshot_to_assistant(state, "tools", &tools_lines);
        }
        LocalTuiCommand::Routines => {
            let routines_lines = state.integration_routines_lines.clone();
            emit_snapshot_to_assistant(state, "routines", &routines_lines);
        }
        LocalTuiCommand::Cortex => {
            let cortex_lines = state.integration_cortex_lines.clone();
            emit_snapshot_to_assistant(state, "cortex", &cortex_lines);
        }
        LocalTuiCommand::Memory => {
            let memory_lines = state.integration_memory_lines.clone();
            emit_snapshot_to_assistant(state, "memory", &memory_lines);
        }
        LocalTuiCommand::Sync => {
            if let Some(tx) = gateway_sync_tx {
                let _ = tx.send(GatewaySyncCommand::RefreshNow);
                state.progress_status = "gateway sync refresh requested".to_string();
                push_history_line(
                    &mut state.timeline_lines,
                    "integration.sync requested by local command".to_string(),
                );
            } else {
                push_history_line(
                    &mut state.assistant_lines,
                    "gateway sync worker unavailable".to_string(),
                );
            }
        }
        LocalTuiCommand::Colors => {
            state.toggle_tone_mode();
            push_history_line(
                &mut state.assistant_lines,
                format!(
                    "color mode: {} ({})",
                    state.tone_mode.label(),
                    state.tone_mode.legend()
                ),
            );
            push_history_line(
                &mut state.timeline_lines,
                format!("view.color_mode {}", state.tone_mode.label()),
            );
        }
    }
}

fn record_submitted_prompt(state: &mut AgentAppState, pending_input: &str) {
    let trimmed = pending_input.trim();
    if trimmed.is_empty() {
        return;
    }
    state.note_turn_submitted();
    let compact_prompt = compact_ui_snippet(trimmed, 120);
    state.last_submitted_prompt = compact_prompt.clone();
    state.turn_status = "turn.start submitted to runtime".to_string();
    state.progress_status = "waiting for model/tool events".to_string();
    state.turn_phase = TurnPhase::Queued;
    state.turn_request_budget_ms = state.default_request_budget_ms;
    let user_line = format!("you> {trimmed}");
    push_history_line(&mut state.assistant_lines, user_line);
    push_history_line(&mut state.timeline_lines, format!("user.input {trimmed}"));
    push_history_line(
        &mut state.event_lines,
        format!("turn.submit prompt=\"{compact_prompt}\""),
    );
}

fn push_unique_exit_line(out: &mut Vec<String>, line: &str) {
    let normalized = line.trim();
    if normalized.is_empty() {
        return;
    }
    if out.iter().any(|existing| existing == normalized) {
        return;
    }
    out.push(normalized.to_string());
}

fn collect_exit_context_lines(state: &AgentAppState, max_lines: usize) -> Vec<String> {
    let mut out = Vec::new();

    for line in state.event_lines.iter().rev() {
        if line.contains("Error:")
            || line.contains("invalid response")
            || line.contains("request timed out")
            || line.contains("invalid_api_key")
            || line.contains("Please upgrade Node.js")
            || line.contains("interactive turn failed")
        {
            push_unique_exit_line(&mut out, line);
            if out.len() >= max_lines {
                break;
            }
        }
    }
    if out.len() < max_lines {
        for line in state.assistant_lines.iter().rev() {
            if line.contains("Error:")
                || line.contains("invalid response")
                || line.contains("request timed out")
                || line.contains("invalid_api_key")
                || line.contains("Please upgrade Node.js")
            {
                push_unique_exit_line(&mut out, line);
                if out.len() >= max_lines {
                    break;
                }
            }
        }
    }
    if out.is_empty() {
        for line in state.event_lines.iter().rev().take(max_lines) {
            push_unique_exit_line(&mut out, line);
        }
    }
    out.reverse();
    out
}

fn format_runtime_exit_error(status: std::process::ExitStatus, state: &AgentAppState) -> String {
    let context = collect_exit_context_lines(state, 4);
    if context.is_empty() {
        return format!("interactive runtime exited with status {status}");
    }
    format!(
        "interactive runtime exited with status {status}\nlast runtime output:\n{}",
        context
            .into_iter()
            .map(|line| format!("  - {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn is_escape_sequence_terminator(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'~'
}

fn handle_escape_sequence(state: &mut AgentAppState, sequence: &str) -> bool {
    match sequence {
        "[A" => {
            state.scroll_focused_panel_up();
            return true;
        }
        "[B" => {
            state.scroll_focused_panel_down();
            return true;
        }
        "[C" => {
            state.focus_next_panel();
            return true;
        }
        "[D" => {
            state.focus_previous_panel();
            return true;
        }
        "[Z" => {
            state.focus_previous_panel();
            return true;
        }
        "[5~" => {
            state.page_scroll_focused_panel_up();
            return true;
        }
        "[6~" => {
            state.page_scroll_focused_panel_down();
            return true;
        }
        "[H" | "[1~" => {
            state.scroll_focused_panel_to_oldest();
            return true;
        }
        "[F" | "[4~" => {
            state.scroll_focused_panel_to_latest();
            return true;
        }
        _ => {}
    }

    let ctrl_modifier = sequence.contains(";5");
    if !ctrl_modifier {
        return false;
    }
    match sequence.chars().last() {
        Some('A') => {
            state.adjust_split_top_percent(5);
            true
        }
        Some('B') => {
            state.adjust_split_top_percent(-5);
            true
        }
        Some('C') => {
            state.adjust_split_left_percent(5);
            true
        }
        Some('D') => {
            state.adjust_split_left_percent(-5);
            true
        }
        _ => false,
    }
}

fn build_agent_launch_summary_lines(
    frame: &OperatorShellFrame,
    args: &AgentArgs,
    launch_command: &str,
) -> Vec<String> {
    let ready_providers = frame
        .auth_rows
        .iter()
        .filter(|row| row.state.eq_ignore_ascii_case("ready"))
        .count();
    let provider_count = frame.auth_rows.len();
    vec![
        format!(
            "Tau agent CLI - profile={} env={} heartbeat={}",
            frame.profile, frame.environment, frame.heartbeat
        ),
        format!(
            "model: {} | auth mode: {} | ready: {}/{}",
            args.model, frame.auth_mode, ready_providers, provider_count
        ),
        format!(
            "health: {} | alert: {} ({})",
            frame.health_reason, frame.primary_alert_message, frame.primary_alert_severity
        ),
        format!(
            "paths: dashboard={} gateway={}",
            args.dashboard_state_dir, args.gateway_state_dir
        ),
        match resolve_gateway_session_bridge_path(args) {
            Some(path) => format!("session bridge: {path}"),
            None => "session bridge: disabled (explicit --session override)".to_string(),
        },
        "commands: /help /status /dashboard /tools /routines /cortex /memory /memory-distill /sync /colors /model /mcp /permissions /quit".to_string(),
        "shortcuts: Ctrl+C cancel turn | Ctrl+D exit | Ctrl+R history | Ctrl+G colors | Tab complete".to_string(),
        format!("interactive.command={launch_command}"),
    ]
}

fn run_agent(args: AgentArgs) -> Result<(), String> {
    let theme = Theme::default();
    let frame = OperatorShellFrame::from_dashboard_state_dir(
        args.profile.clone(),
        Path::new(args.dashboard_state_dir.as_str()),
    );
    let command_tokens = build_agent_runtime_command(&args);
    let launch_command = format_shell_command(&command_tokens);
    let summary_lines = build_agent_launch_summary_lines(&frame, &args, &launch_command);
    let prefs_path = resolve_tui_prefs_path();
    let default_request_budget_ms = args.request_timeout_ms;
    let use_app_renderer = !args.passthrough;
    if !use_app_renderer {
        println!(
            "{}",
            paint(
                &theme,
                ThemeRole::Muted,
                "launch: agent-interactive ready".to_string(),
                args.color
            )
        );
        println!(
            "{}",
            paint(
                &theme,
                ThemeRole::Muted,
                format_interactive_controls_marker().to_string(),
                args.color
            )
        );
        for line in &summary_lines {
            let role = if line.starts_with("Tau agent CLI") {
                ThemeRole::Accent
            } else if line.starts_with("commands:")
                || line.starts_with("shortcuts:")
                || line.starts_with("interactive.command=")
            {
                ThemeRole::Muted
            } else {
                ThemeRole::Primary
            };
            println!("{}", paint(&theme, role, line.clone(), args.color));
        }
    }

    if args.dry_run {
        if use_app_renderer {
            println!(
                "{}",
                paint(
                    &theme,
                    ThemeRole::Muted,
                    "launch: agent-interactive ready".to_string(),
                    args.color
                )
            );
            println!(
                "{}",
                paint(
                    &theme,
                    ThemeRole::Muted,
                    format_interactive_controls_marker().to_string(),
                    args.color
                )
            );
            for line in &summary_lines {
                let role = if line.starts_with("Tau agent CLI") {
                    ThemeRole::Accent
                } else if line.starts_with("commands:")
                    || line.starts_with("shortcuts:")
                    || line.starts_with("interactive.command=")
                {
                    ThemeRole::Muted
                } else {
                    ThemeRole::Primary
                };
                println!("{}", paint(&theme, role, line.clone(), args.color));
            }
        }
        return Ok(());
    }

    let (program, remaining_args) = command_tokens
        .split_first()
        .ok_or_else(|| "interactive runtime command is empty".to_string())?;
    if !use_app_renderer {
        let status = Command::new(program)
            .args(remaining_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|error| format!("failed to launch interactive runtime: {error}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("interactive runtime exited with status {status}"));
    }

    let mut child = Command::new(program)
        .args(remaining_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch interactive runtime: {error}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture interactive runtime stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "failed to capture interactive runtime stderr".to_string())?;
    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| "failed to capture interactive runtime stdin".to_string())?;
    let (tx, rx) = mpsc::channel::<AgentRuntimeEvent>();
    let stdout_handle = spawn_output_pump(stdout, AgentOutputSource::Stdout, tx.clone());
    let stderr_handle = spawn_output_pump(stderr, AgentOutputSource::Stderr, tx.clone());
    let (gateway_sync_tx, gateway_sync_handle) = spawn_gateway_sync_pump(tx.clone());
    let input_enabled = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if args.ratatui && input_enabled {
        drop(tx);
        return run_agent_ratatui(
            &summary_lines,
            &launch_command,
            RatatuiRuntimeHandles {
                child,
                child_stdin,
                rx,
                stdout_handle,
                stderr_handle,
                gateway_sync_tx,
                gateway_sync_handle,
            },
            prefs_path.as_path(),
            default_request_budget_ms,
        );
    }

    let _terminal_guard = TerminalUiGuard::enter(input_enabled);
    let _input_handle = if input_enabled {
        Some(spawn_input_pump(tx.clone()))
    } else {
        None
    };
    drop(tx);

    let mut app_state = AgentAppState {
        default_request_budget_ms,
        ..AgentAppState::default()
    };
    if let Err(error) = load_tui_prefs(prefs_path.as_path(), &mut app_state) {
        push_history_line(
            &mut app_state.event_lines,
            format!("prefs warning: {}", compact_ui_snippet(error.as_str(), 140)),
        );
    }
    let mut pending_input = String::new();
    let mut quit_requested_at: Option<Instant> = None;
    let mut escape_in_progress = false;
    let mut escape_sequence = String::new();
    update_prompt_line(&mut app_state, pending_input.as_str());
    redraw_agent_app(&args, &summary_lines, &app_state, &launch_command);
    loop {
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentRuntimeEvent::Output(source, line) => {
                    update_agent_app_state(&mut app_state, source, &line);
                }
                AgentRuntimeEvent::GatewaySync(snapshot) => {
                    apply_gateway_sync_snapshot(&mut app_state, snapshot);
                }
                AgentRuntimeEvent::InputByte(byte) => match byte {
                    27 => {
                        escape_in_progress = true;
                        escape_sequence.clear();
                    }
                    byte if escape_in_progress => {
                        if (32..=126).contains(&byte) {
                            escape_sequence.push(byte as char);
                            if is_escape_sequence_terminator(byte) {
                                let _ = handle_escape_sequence(&mut app_state, &escape_sequence);
                                escape_in_progress = false;
                                escape_sequence.clear();
                            } else if escape_sequence.len() > 20 {
                                escape_in_progress = false;
                                escape_sequence.clear();
                            }
                        } else {
                            escape_in_progress = false;
                            escape_sequence.clear();
                        }
                    }
                    9 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.focus_next_panel();
                    }
                    12 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        // Ctrl+L redraw request: no-op here, redraw occurs after event handling.
                    }
                    b'?' if pending_input.is_empty() => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.show_shortcuts = !app_state.show_shortcuts;
                        app_state.ui_prefs_dirty = true;
                    }
                    b'[' if pending_input.is_empty() => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.focus_previous_panel();
                    }
                    b']' if pending_input.is_empty() => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.focus_next_panel();
                    }
                    b'k' if pending_input.is_empty() => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.scroll_focused_panel_up();
                    }
                    b'j' if pending_input.is_empty() => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.scroll_focused_panel_down();
                    }
                    b'?' => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        pending_input.push('?');
                        update_prompt_line(&mut app_state, pending_input.as_str());
                    }
                    3 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        send_interrupt_to_child(child.id());
                        pending_input.clear();
                        update_prompt_line(&mut app_state, pending_input.as_str());
                    }
                    2 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.scroll_focused_panel_up();
                    }
                    4 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        let _ = child_stdin.write_all(b"/quit\n");
                        let _ = child_stdin.flush();
                        if quit_requested_at.is_none() {
                            quit_requested_at = Some(Instant::now());
                        }
                    }
                    5 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.toggle_expand_focused_panel();
                    }
                    6 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.scroll_focused_panel_down();
                    }
                    7 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.toggle_tone_mode();
                    }
                    8 | 127 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        pending_input.pop();
                        update_prompt_line(&mut app_state, pending_input.as_str());
                    }
                    14 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.focus_next_panel();
                    }
                    16 => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        app_state.focus_previous_panel();
                    }
                    b'\r' | b'\n' => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        if app_state.show_shortcuts {
                            app_state.show_shortcuts = false;
                            app_state.ui_prefs_dirty = true;
                        }
                        record_submitted_prompt(&mut app_state, pending_input.as_str());
                        let local_command = parse_local_tui_command(pending_input.as_str());
                        let submit_was_quit = is_local_quit_command(pending_input.as_str());
                        if let Some(command) = local_command {
                            handle_local_tui_command(
                                &mut app_state,
                                command,
                                Some(&gateway_sync_tx),
                            );
                        } else {
                            if !pending_input.is_empty() {
                                let _ = child_stdin.write_all(pending_input.as_bytes());
                            }
                            let _ = child_stdin.write_all(b"\n");
                            let _ = child_stdin.flush();
                            app_state.mark_turn_started();
                        }
                        pending_input.clear();
                        update_prompt_line(&mut app_state, pending_input.as_str());
                        if submit_was_quit && quit_requested_at.is_none() {
                            quit_requested_at = Some(Instant::now());
                        }
                    }
                    byte if (32..=126).contains(&byte) => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                        if app_state.show_shortcuts {
                            app_state.show_shortcuts = false;
                            app_state.ui_prefs_dirty = true;
                        }
                        pending_input.push(byte as char);
                        update_prompt_line(&mut app_state, pending_input.as_str());
                    }
                    _ => {
                        escape_in_progress = false;
                        escape_sequence.clear();
                    }
                },
            }
            redraw_agent_app(&args, &summary_lines, &app_state, &launch_command);
        }

        app_state.tick_spinner();
        persist_tui_prefs_if_dirty(prefs_path.as_path(), &mut app_state);

        if let Some(requested_at) = quit_requested_at {
            if requested_at.elapsed() >= Duration::from_millis(AGENT_LOCAL_QUIT_GRACE_MS) {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_handle.join();
                let _ = stderr_handle.join();
                persist_tui_prefs_if_dirty(prefs_path.as_path(), &mut app_state);
                shutdown_gateway_sync_worker(&gateway_sync_tx, gateway_sync_handle);
                return Ok(());
            }
        }

        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll interactive runtime status: {error}"))?
        {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            while let Ok(event) = rx.try_recv() {
                match event {
                    AgentRuntimeEvent::Output(source, line) => {
                        update_agent_app_state(&mut app_state, source, &line);
                    }
                    AgentRuntimeEvent::GatewaySync(snapshot) => {
                        apply_gateway_sync_snapshot(&mut app_state, snapshot);
                    }
                    AgentRuntimeEvent::InputByte(_) => {}
                }
            }
            redraw_agent_app(&args, &summary_lines, &app_state, &launch_command);
            if status.success() {
                persist_tui_prefs_if_dirty(prefs_path.as_path(), &mut app_state);
                shutdown_gateway_sync_worker(&gateway_sync_tx, gateway_sync_handle);
                return Ok(());
            }
            persist_tui_prefs_if_dirty(prefs_path.as_path(), &mut app_state);
            shutdown_gateway_sync_worker(&gateway_sync_tx, gateway_sync_handle);
            return Err(format_runtime_exit_error(status, &app_state));
        }

        thread::sleep(Duration::from_millis(40));
    }
}

fn main() {
    let action = match parse_args(env::args()) {
        Ok(action) => action,
        Err(err) => {
            eprintln!("{err}");
            eprintln!();
            eprintln!("{HELP}");
            std::process::exit(2);
        }
    };

    match action {
        ParseAction::Help => {
            println!("{HELP}");
        }
        ParseAction::RunDemo(args) => {
            if let Err(err) = run_demo(args) {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
        ParseAction::RunShell(args) => run_shell(args),
        ParseAction::RunShellLive(args) => run_shell_live(args),
        ParseAction::RunAgent(args) => {
            if let Err(err) = run_agent(args) {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{compose_frame, parse_args, ParseAction, HELP};
    use std::collections::HashMap;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tau_tui::{render_operator_shell_frame, EditorBuffer, LumaImage, OperatorShellFrame};

    fn agent_args_fixture() -> super::AgentArgs {
        super::AgentArgs {
            width: 88,
            profile: "ops-interactive".to_string(),
            dashboard_state_dir: ".tau/custom-dashboard".to_string(),
            gateway_state_dir: ".tau/custom-gateway".to_string(),
            model: "openai/gpt-5.2".to_string(),
            dry_run: true,
            color: false,
            ..super::AgentArgs::default()
        }
    }

    fn temp_tui_prefs_path(label: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("tau-tui-{label}-{}-{ts}.json", std::process::id()))
    }

    fn spawn_json_fixture_server(
        routes: HashMap<String, String>,
    ) -> (String, Arc<AtomicBool>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
        listener
            .set_nonblocking(true)
            .expect("set fixture listener nonblocking");
        let addr = listener.local_addr().expect("fixture listener addr");
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_for_thread = Arc::clone(&shutdown);
        let handle = thread::spawn(move || {
            while !shutdown_for_thread.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _peer)) => {
                        let mut buffer = [0_u8; 16_384];
                        let read_len = stream.read(&mut buffer).unwrap_or(0);
                        let request = String::from_utf8_lossy(&buffer[..read_len]).to_string();
                        let path = request
                            .lines()
                            .next()
                            .and_then(|line| line.split_whitespace().nth(1))
                            .unwrap_or("/");
                        let (status, body) = match routes.get(path) {
                            Some(payload) => ("200 OK", payload.clone()),
                            None => (
                                "404 Not Found",
                                serde_json::json!({
                                    "error": "fixture_route_missing",
                                    "path": path
                                })
                                .to_string(),
                            ),
                        };
                        let response = format!(
                            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        (format!("http://{addr}"), shutdown, handle)
    }

    #[test]
    fn unit_parse_args_defaults_are_stable() {
        let action = parse_args(vec!["tau-tui".to_string()]).expect("parse succeeds");
        let ParseAction::RunDemo(parsed) = action else {
            panic!("expected run action");
        };
        assert_eq!(parsed.frames, 3);
        assert_eq!(parsed.width, 72);
        assert_eq!(parsed.sleep_ms, 120);
        assert!(parsed.color);
    }

    #[test]
    fn functional_parse_args_supports_custom_values() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "--frames".to_string(),
            "5".to_string(),
            "--width".to_string(),
            "90".to_string(),
            "--sleep-ms".to_string(),
            "0".to_string(),
            "--no-color".to_string(),
        ])
        .expect("parse succeeds");
        let ParseAction::RunDemo(parsed) = action else {
            panic!("expected run action");
        };
        assert_eq!(parsed.frames, 5);
        assert_eq!(parsed.width, 90);
        assert_eq!(parsed.sleep_ms, 0);
        assert!(!parsed.color);
    }

    #[test]
    fn regression_parse_args_rejects_zero_frames() {
        let err = parse_args(vec![
            "tau-tui".to_string(),
            "--frames".to_string(),
            "0".to_string(),
        ])
        .expect_err("expected parse failure");
        assert!(err.contains("--frames must be >= 1"));
    }

    #[test]
    fn regression_compose_frame_overlays_frame_metadata() {
        let mut buffer = EditorBuffer::new();
        buffer.insert_text("let tau = true;");
        let image = LumaImage::from_luma(2, 2, vec![0, 128, 200, 255]).expect("image");
        let args = super::DemoArgs {
            frames: 2,
            width: 40,
            sleep_ms: 0,
            color: false,
        };

        let frame = compose_frame(&buffer, &image, &args, 0);
        assert!(!frame.is_empty());
        assert!(frame[0].contains("frame=1/2"));
        assert!(frame.iter().any(|line| line.contains("ascii preview")));
    }

    #[test]
    fn spec_c01_parse_args_accepts_shell_mode_and_overrides() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "shell".to_string(),
            "--width".to_string(),
            "96".to_string(),
            "--profile".to_string(),
            "prod-west".to_string(),
            "--no-color".to_string(),
        ])
        .expect("parse succeeds");

        let ParseAction::RunShell(args) = action else {
            panic!("expected shell action");
        };
        assert_eq!(args.width, 96);
        assert_eq!(args.profile, "prod-west");
        assert!(!args.color);
    }

    #[test]
    fn spec_c01_shell_renderer_includes_all_operator_panels() {
        let frame = OperatorShellFrame::deterministic_fixture("local-dev".to_string());
        let lines = render_operator_shell_frame(&frame, 78);
        let rendered = lines.join("\n");

        for panel in ["STATUS", "AUTH", "TRAINING", "ALERTS", "ACTIONS"] {
            assert!(
                rendered.contains(panel),
                "missing shell panel header `{panel}` in:\n{rendered}"
            );
        }
    }

    #[test]
    fn spec_c02_parse_args_rejects_shell_profile_without_value() {
        let err = parse_args(vec![
            "tau-tui".to_string(),
            "shell".to_string(),
            "--profile".to_string(),
        ])
        .expect_err("expected parse failure");
        assert!(err.contains("missing value for --profile"));
    }

    #[test]
    fn spec_c03_parse_args_accepts_shell_live_mode_and_state_dir() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "shell-live".to_string(),
            "--width".to_string(),
            "92".to_string(),
            "--profile".to_string(),
            "ops-live".to_string(),
            "--state-dir".to_string(),
            ".tau/custom-dashboard".to_string(),
            "--no-color".to_string(),
        ])
        .expect("parse succeeds");

        let ParseAction::RunShellLive(args) = action else {
            panic!("expected shell-live action");
        };
        assert_eq!(args.width, 92);
        assert_eq!(args.profile, "ops-live");
        assert_eq!(args.state_dir, ".tau/custom-dashboard");
        assert!(!args.watch);
        assert_eq!(args.iterations, 3);
        assert_eq!(args.interval_ms, 1000);
        assert!(!args.color);
    }

    #[test]
    fn regression_parse_args_rejects_shell_live_state_dir_without_value() {
        let err = parse_args(vec![
            "tau-tui".to_string(),
            "shell-live".to_string(),
            "--state-dir".to_string(),
        ])
        .expect_err("expected parse failure");
        assert!(err.contains("missing value for --state-dir"));
    }

    #[test]
    fn integration_spec_3474_c01_parse_args_accepts_shell_live_watch_mode_controls() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "shell-live".to_string(),
            "--watch".to_string(),
            "--iterations".to_string(),
            "4".to_string(),
            "--interval-ms".to_string(),
            "25".to_string(),
        ])
        .expect("expected watch-mode parse success");

        let ParseAction::RunShellLive(args) = action else {
            panic!("expected shell-live action");
        };
        assert!(args.watch);
        assert_eq!(args.iterations, 4);
        assert_eq!(args.interval_ms, 25);
    }

    #[test]
    fn regression_spec_3474_c02_parse_args_rejects_shell_live_zero_iterations() {
        let err = parse_args(vec![
            "tau-tui".to_string(),
            "shell-live".to_string(),
            "--watch".to_string(),
            "--iterations".to_string(),
            "0".to_string(),
        ])
        .expect_err("expected parse failure");
        assert!(err.contains("--iterations must be >= 1"));
    }

    #[test]
    fn functional_spec_3474_c03_live_watch_marker_contract_is_deterministic() {
        let marker = super::format_live_watch_marker(2, 4, 25, 7);
        assert_eq!(
            marker,
            "watch.cycle=2/4 watch.interval_ms=25 watch.diff_ops=7"
        );
    }

    #[test]
    fn functional_spec_3474_c04_help_text_exposes_shell_live_watch_flags() {
        assert!(HELP.contains("--watch"));
        assert!(HELP.contains("--iterations"));
        assert!(HELP.contains("--interval-ms"));
    }

    #[test]
    fn spec_c05_parse_args_accepts_agent_mode_and_overrides() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--profile".to_string(),
            "ops-interactive".to_string(),
            "--model".to_string(),
            "openai/gpt-5.2".to_string(),
            "--dashboard-state-dir".to_string(),
            ".tau/custom-dashboard".to_string(),
            "--gateway-state-dir".to_string(),
            ".tau/custom-gateway".to_string(),
            "--dry-run".to_string(),
            "--no-color".to_string(),
        ])
        .expect("expected agent-mode parse success");

        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert_eq!(args.profile, "ops-interactive");
        assert_eq!(args.model, "openai/gpt-5.2");
        assert_eq!(args.dashboard_state_dir, ".tau/custom-dashboard");
        assert_eq!(args.gateway_state_dir, ".tau/custom-gateway");
        assert!(args.ratatui);
        assert!(args.dry_run);
        assert!(!args.color);
    }

    #[test]
    fn regression_spec_c05_parse_args_allows_disabling_ratatui_renderer() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--no-ratatui".to_string(),
            "--dry-run".to_string(),
        ])
        .expect("expected agent-mode parse success");

        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert!(!args.ratatui);
    }

    #[test]
    fn functional_spec_c05_build_agent_runtime_command_contract_is_stable() {
        let args = agent_args_fixture();
        let command = super::build_agent_runtime_command(&args);
        assert!(
            command[0] == "cargo" || command[0].ends_with("tau-coding-agent"),
            "expected cargo launcher or direct tau-coding-agent binary, got {:?}",
            command
        );
        assert!(command.contains(&"--model".to_string()));
        assert!(command.contains(&"openai/gpt-5.2".to_string()));
        assert!(command.contains(&"--dashboard-state-dir".to_string()));
        assert!(command.contains(&".tau/custom-dashboard".to_string()));
        assert!(command.contains(&"--gateway-state-dir".to_string()));
        assert!(command.contains(&".tau/custom-gateway".to_string()));
        assert!(command.contains(&"--session".to_string()));
        assert!(command
            .contains(&".tau/custom-gateway/openresponses/sessions/default.jsonl".to_string()));
        assert!(command.contains(&"--interactive-launch-surface".to_string()));
        assert!(command.contains(&"tui".to_string()));
    }

    #[test]
    fn regression_spec_c05_agent_runtime_command_respects_explicit_session_override() {
        let args = super::AgentArgs {
            agent_args: vec![
                "--session".to_string(),
                ".tau/sessions/custom.sqlite".to_string(),
            ],
            ..agent_args_fixture()
        };
        let command = super::build_agent_runtime_command(&args);
        let bridge_path = ".tau/custom-gateway/openresponses/sessions/default.jsonl".to_string();
        assert!(!command.contains(&bridge_path));
        assert_eq!(
            command
                .iter()
                .filter(|token| token.as_str() == "--session")
                .count(),
            1
        );
        assert!(
            command.ends_with(&[
                "--session".to_string(),
                ".tau/sessions/custom.sqlite".to_string()
            ]),
            "explicit --session override should remain command tail"
        );
    }

    #[test]
    fn regression_spec_c05_parse_args_rejects_agent_gateway_state_dir_without_value() {
        let err = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--gateway-state-dir".to_string(),
        ])
        .expect_err("expected parse failure");
        assert!(err.contains("missing value for --gateway-state-dir"));
    }

    #[test]
    fn functional_spec_c05_help_text_exposes_agent_flags() {
        assert!(HELP.contains("agent"));
        assert!(HELP.contains("--dashboard-state-dir"));
        assert!(HELP.contains("--gateway-state-dir"));
        assert!(HELP.contains("gateway/webchat session bridging"));
        assert!(HELP.contains("--max-turns"));
        assert!(HELP.contains("--request-timeout-ms"));
        assert!(HELP.contains("--agent-request-max-retries"));
        assert!(HELP.contains("--codex-reasoning-effort"));
        assert!(HELP.contains("--interactive-timeline-verbose"));
        assert!(HELP.contains("--ratatui"));
        assert!(HELP.contains("--no-ratatui"));
        assert!(HELP.contains("--dry-run"));
    }

    #[test]
    fn regression_spec_c06_agent_mode_defaults_to_gpt5_baseline() {
        let action = parse_args(vec!["tau-tui".to_string(), "agent".to_string()])
            .expect("expected parse success");
        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert_eq!(args.model, "openai/gpt-5.2");
    }

    #[test]
    fn spec_c07_parse_args_accepts_agent_timeout_and_retry_overrides() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--request-timeout-ms".to_string(),
            "45000".to_string(),
            "--agent-request-max-retries".to_string(),
            "0".to_string(),
            "--dry-run".to_string(),
        ])
        .expect("expected parse success for timeout/retry overrides");

        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert_eq!(args.request_timeout_ms, Some(45_000));
        assert_eq!(args.agent_request_max_retries, Some(0));
    }

    #[test]
    fn functional_spec_c08_agent_runtime_command_includes_timeout_retry_flags() {
        let args = super::AgentArgs {
            request_timeout_ms: Some(45_000),
            agent_request_max_retries: Some(0),
            ..agent_args_fixture()
        };
        let command = super::build_agent_runtime_command(&args);
        assert!(command.contains(&"--request-timeout-ms".to_string()));
        assert!(command.contains(&"45000".to_string()));
        assert!(command.contains(&"--agent-request-max-retries".to_string()));
        assert!(command.contains(&"0".to_string()));
    }

    #[test]
    fn regression_spec_c08_agent_runtime_command_prefers_explicit_binary_override() {
        let args = super::AgentArgs {
            request_timeout_ms: Some(45_000),
            agent_request_max_retries: Some(0),
            agent_binary: Some("target/debug/tau-coding-agent".to_string()),
            ..agent_args_fixture()
        };
        let command = super::build_agent_runtime_command(&args);
        assert_eq!(command[0], "target/debug/tau-coding-agent");
        assert!(
            !command.contains(&"cargo".to_string()),
            "explicit binary override should bypass cargo run launcher"
        );
    }

    #[test]
    fn spec_c09_parse_args_accepts_codex_reasoning_effort_override() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--codex-reasoning-effort".to_string(),
            "high".to_string(),
        ])
        .expect("expected parse success for codex reasoning override");

        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert_eq!(args.codex_reasoning_effort.as_deref(), Some("high"));
    }

    #[test]
    fn regression_spec_c09_parse_args_rejects_invalid_codex_reasoning_effort() {
        let error = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--codex-reasoning-effort".to_string(),
            "turbo".to_string(),
        ])
        .expect_err("expected parse failure");
        assert!(error.contains("invalid value for --codex-reasoning-effort"));
    }

    #[test]
    fn spec_c09_parse_args_accepts_interactive_timeline_verbose_flag() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--interactive-timeline-verbose".to_string(),
        ])
        .expect("expected parse success for interactive timeline verbose override");
        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert!(args.interactive_timeline_verbose);
    }

    #[test]
    fn functional_spec_c10_agent_runtime_command_includes_reasoning_effort_override() {
        let args = super::AgentArgs {
            request_timeout_ms: Some(45_000),
            agent_request_max_retries: Some(0),
            codex_reasoning_effort: Some("minimal".to_string()),
            ..agent_args_fixture()
        };
        let command = super::build_agent_runtime_command(&args);
        assert!(command
            .contains(&"--openai-codex-args=-c,model_reasoning_effort=\"minimal\"".to_string()));
    }

    #[test]
    fn functional_spec_c10_agent_runtime_command_includes_interactive_timeline_verbose_flag() {
        let args = super::AgentArgs {
            interactive_timeline_verbose: true,
            ..agent_args_fixture()
        };
        let command = super::build_agent_runtime_command(&args);
        assert!(command.contains(&"--interactive-timeline-verbose".to_string()));
    }

    #[test]
    fn spec_c11_parse_args_accepts_agent_max_turns_override() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--max-turns".to_string(),
            "73".to_string(),
            "--dry-run".to_string(),
        ])
        .expect("expected parse success for max-turns override");

        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert_eq!(args.max_turns, Some(73));
    }

    #[test]
    fn regression_spec_c11_parse_args_rejects_zero_agent_max_turns() {
        let err = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--max-turns".to_string(),
            "0".to_string(),
        ])
        .expect_err("expected parse failure");
        assert!(err.contains("--max-turns must be >= 1"));
    }

    #[test]
    fn functional_spec_c11_agent_runtime_command_includes_max_turns_flag() {
        let args = super::AgentArgs {
            max_turns: Some(50),
            request_timeout_ms: Some(45_000),
            agent_request_max_retries: Some(0),
            ..agent_args_fixture()
        };
        let command = super::build_agent_runtime_command(&args);
        assert!(command.contains(&"--max-turns".to_string()));
        assert!(command.contains(&"50".to_string()));
    }

    #[test]
    fn spec_c12_parse_args_accepts_mcp_memory_skills_and_rl_flags() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "agent".to_string(),
            "--mcp-client".to_string(),
            "--mcp-external-server-config".to_string(),
            ".tau/mcp/servers.json".to_string(),
            "--memory-state-dir".to_string(),
            ".tau/memory".to_string(),
            "--skills-dir".to_string(),
            ".tau/skills".to_string(),
            "--skill".to_string(),
            "web-mcp".to_string(),
            "--skill".to_string(),
            "summarizer".to_string(),
            "--prompt-optimization-config".to_string(),
            ".tau/rl/config.json".to_string(),
            "--prompt-optimization-store-sqlite".to_string(),
            ".tau/rl/store.sqlite".to_string(),
            "--prompt-optimization-json".to_string(),
            "--agent-arg".to_string(),
            "--json-events".to_string(),
        ])
        .expect("expected parse success");

        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert!(args.mcp_client);
        assert_eq!(
            args.mcp_external_server_config.as_deref(),
            Some(".tau/mcp/servers.json")
        );
        assert_eq!(args.memory_state_dir.as_deref(), Some(".tau/memory"));
        assert_eq!(args.skills_dir.as_deref(), Some(".tau/skills"));
        assert_eq!(
            args.skills,
            vec!["web-mcp".to_string(), "summarizer".to_string()]
        );
        assert_eq!(
            args.prompt_optimization_config.as_deref(),
            Some(".tau/rl/config.json")
        );
        assert_eq!(
            args.prompt_optimization_store_sqlite.as_deref(),
            Some(".tau/rl/store.sqlite")
        );
        assert!(args.prompt_optimization_json);
        assert_eq!(args.agent_args, vec!["--json-events".to_string()]);
    }

    #[test]
    fn functional_spec_c13_agent_runtime_command_includes_mcp_memory_skills_and_rl_flags() {
        let args = super::AgentArgs {
            mcp_client: true,
            mcp_external_server_config: Some(".tau/mcp/servers.json".to_string()),
            memory_state_dir: Some(".tau/memory".to_string()),
            skills_dir: Some(".tau/skills".to_string()),
            skills: vec!["web-mcp".to_string(), "summarizer".to_string()],
            prompt_optimization_config: Some(".tau/rl/config.json".to_string()),
            prompt_optimization_store_sqlite: Some(".tau/rl/store.sqlite".to_string()),
            prompt_optimization_json: true,
            agent_args: vec![
                "--json-events".to_string(),
                "--tool-audit-log".to_string(),
                ".tau/audit/tools.jsonl".to_string(),
            ],
            ..agent_args_fixture()
        };

        let command = super::build_agent_runtime_command(&args);
        assert!(command.contains(&"--mcp-client".to_string()));
        assert!(command.contains(&"--mcp-external-server-config".to_string()));
        assert!(command.contains(&".tau/mcp/servers.json".to_string()));
        assert!(command.contains(&"--memory-state-dir".to_string()));
        assert!(command.contains(&".tau/memory".to_string()));
        assert!(command.contains(&"--skills-dir".to_string()));
        assert!(command.contains(&".tau/skills".to_string()));
        assert!(command.contains(&"--skill".to_string()));
        assert!(command.contains(&"web-mcp".to_string()));
        assert!(command.contains(&"summarizer".to_string()));
        assert!(command.contains(&"--prompt-optimization-config".to_string()));
        assert!(command.contains(&".tau/rl/config.json".to_string()));
        assert!(command.contains(&"--prompt-optimization-store-sqlite".to_string()));
        assert!(command.contains(&".tau/rl/store.sqlite".to_string()));
        assert!(command.contains(&"--prompt-optimization-json".to_string()));
        assert!(
            command.ends_with(&[
                "--json-events".to_string(),
                "--tool-audit-log".to_string(),
                ".tau/audit/tools.jsonl".to_string()
            ]),
            "expected raw agent args appended in-order at command tail, got {command:?}"
        );
    }

    #[test]
    fn functional_spec_c14_help_text_exposes_mcp_memory_rl_and_skills_flags() {
        assert!(HELP.contains("--mcp-client"));
        assert!(HELP.contains("--mcp-external-server-config"));
        assert!(HELP.contains("--memory-state-dir"));
        assert!(HELP.contains("--skills-dir"));
        assert!(HELP.contains("--skill"));
        assert!(HELP.contains("--prompt-optimization-config"));
        assert!(HELP.contains("--prompt-optimization-store-sqlite"));
        assert!(HELP.contains("--prompt-optimization-json"));
        assert!(HELP.contains("--agent-arg"));
    }

    #[test]
    fn functional_spec_c15_agent_launch_summary_is_compact_and_command_oriented() {
        let frame = OperatorShellFrame::deterministic_fixture("local-dev".to_string());
        let args = super::AgentArgs {
            profile: "local-dev".to_string(),
            dashboard_state_dir: ".tau/dashboard".to_string(),
            gateway_state_dir: ".tau/gateway".to_string(),
            model: "openai/gpt-5.2".to_string(),
            ..super::AgentArgs::default()
        };
        let lines = super::build_agent_launch_summary_lines(
            &frame,
            &args,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let rendered = lines.join("\n");
        assert!(rendered.contains("agent CLI"));
        assert!(rendered.contains(
            "commands: /help /status /dashboard /tools /routines /cortex /memory /memory-distill /sync /colors /model /mcp /permissions /quit"
        ));
        assert!(rendered.contains("shortcuts: Ctrl+C cancel turn"));
        assert!(rendered.contains("Ctrl+G colors"));
        assert!(rendered
            .contains("interactive.command=target/debug/tau-coding-agent --model openai/gpt-5.2"));
        assert!(!rendered.contains("STATUS"));
        assert!(!rendered.contains("TRAINING"));
        assert!(!rendered.contains("ALERTS"));
    }

    #[test]
    fn regression_spec_c11_interactive_controls_marker_contract_is_stable() {
        assert_eq!(
            super::format_interactive_controls_marker(),
            "controls: Ctrl+C cancel turn | /quit exit"
        );
    }

    #[test]
    fn unit_agent_app_state_parser_routes_lines_to_expected_sections() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "turn.start timeout=180000ms request_budget=180000ms (Ctrl+C cancel, /quit exit)",
        );
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "assistant.timeline plan: requested tool_calls=1 tools=[http]",
        );
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stdout,
            "Sure. Fetching now.",
        );
        super::update_agent_app_state(&mut state, super::AgentOutputSource::Stdout, "tau> ");

        assert!(state.turn_status.starts_with("turn.start"));
        assert_eq!(
            state.timeline_lines.back().map(String::as_str),
            Some("assistant.timeline plan: requested tool_calls=1 tools=[http]")
        );
        assert_eq!(
            state.assistant_lines.back().map(String::as_str),
            Some("Sure. Fetching now.")
        );
        assert_eq!(state.prompt_line, "tau>");
    }

    #[test]
    fn unit_tui_prefs_round_trip_persists_layout_and_tone_mode() {
        let path = temp_tui_prefs_path("prefs-roundtrip");
        let mut state = super::AgentAppState {
            split_left_percent: 74,
            split_top_percent: 81,
            show_shortcuts: true,
            focused_panel: super::AgentPanel::Tools,
            expanded_panel: Some(super::AgentPanel::Timeline),
            tone_mode: super::ToneMode::Minimal,
            ..super::AgentAppState::default()
        };
        state.panel_scroll_offsets[super::AgentPanel::Assistant.index()] = 7;
        state.panel_scroll_offsets[super::AgentPanel::Events.index()] = 11;

        super::persist_tui_prefs(path.as_path(), &state).expect("persist prefs");

        let mut loaded = super::AgentAppState::default();
        super::load_tui_prefs(path.as_path(), &mut loaded).expect("load prefs");

        assert_eq!(loaded.split_left_percent, 74);
        assert_eq!(loaded.split_top_percent, 81);
        assert!(loaded.show_shortcuts);
        assert_eq!(loaded.focused_panel, super::AgentPanel::Tools);
        assert_eq!(loaded.expanded_panel, Some(super::AgentPanel::Timeline));
        assert_eq!(loaded.tone_mode, super::ToneMode::Minimal);
        assert_eq!(loaded.panel_offset(super::AgentPanel::Assistant), 7);
        assert_eq!(loaded.panel_offset(super::AgentPanel::Events), 11);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn unit_live_status_bar_line_surfaces_budget_tool_and_memory_activity_count() {
        let mut state = super::AgentAppState::default();
        state.mark_turn_started();
        state.turn_phase = super::TurnPhase::Tool;
        state.turn_request_budget_ms = Some(180_000);
        state.active_tool_name = Some("http".to_string());
        state.push_memory_activity("memory.write via memory_write".to_string());
        state.push_memory_activity("memory.delete via memory_delete".to_string());

        let rendered = super::format_live_status_bar_line(&state);
        assert!(rendered.contains("phase=tool"));
        assert!(rendered.contains("tool=http"));
        assert!(rendered.contains("budget="));
        assert!(rendered.contains("mem=2"));
    }

    #[test]
    fn unit_recent_memory_activity_lines_returns_tail_ordered_oldest_to_newest() {
        let mut state = super::AgentAppState::default();
        state.push_memory_activity("memory.write alpha".to_string());
        state.push_memory_activity("memory.write beta".to_string());
        state.push_memory_activity("memory.write gamma".to_string());
        state.push_memory_activity("memory.write delta".to_string());

        let tail = super::recent_memory_activity_lines(&state, 2);
        assert_eq!(
            tail,
            vec![
                "memory.write gamma".to_string(),
                "memory.write delta".to_string()
            ]
        );
    }

    #[test]
    fn unit_load_tui_prefs_migrates_legacy_default_layout_to_balanced_defaults() {
        let path = temp_tui_prefs_path("prefs-migrate");
        let payload = serde_json::json!({
            "split_left_percent": super::LEGACY_DEFAULT_SPLIT_LEFT_PERCENT,
            "split_top_percent": super::LEGACY_DEFAULT_SPLIT_TOP_PERCENT,
            "show_shortcuts": false,
            "focused_panel": "assistant",
            "expanded_panel": null,
            "tone_mode": "semantic",
            "panel_scroll_offsets": [0, 0, 0, 0, 0, 0]
        });
        fs::write(
            path.as_path(),
            serde_json::to_string_pretty(&payload).expect("encode prefs fixture"),
        )
        .expect("write legacy prefs fixture");

        let mut loaded = super::AgentAppState::default();
        super::load_tui_prefs(path.as_path(), &mut loaded).expect("load migrated prefs");
        assert_eq!(loaded.split_left_percent, super::DEFAULT_SPLIT_LEFT_PERCENT);
        assert_eq!(loaded.split_top_percent, super::DEFAULT_SPLIT_TOP_PERCENT);
        assert!(
            loaded.ui_prefs_dirty,
            "migrated prefs should persist on next tick"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn unit_record_submitted_prompt_updates_turn_and_event_breadcrumbs() {
        let mut state = super::AgentAppState::default();
        super::record_submitted_prompt(&mut state, "testing");
        assert_eq!(state.last_submitted_prompt, "testing");
        assert_eq!(state.turn_status, "turn.start submitted to runtime");
        assert_eq!(state.progress_status, "waiting for model/tool events");
        assert_eq!(state.turn_phase, super::TurnPhase::Queued);
        assert_eq!(
            state.event_lines.back().map(String::as_str),
            Some("turn.submit prompt=\"testing\"")
        );
    }

    #[test]
    fn unit_local_quit_command_recognizes_slash_quit_and_quit() {
        assert!(super::is_local_quit_command("/quit"));
        assert!(super::is_local_quit_command("  /quit  "));
        assert!(super::is_local_quit_command("quit"));
        assert!(!super::is_local_quit_command("/help"));
    }

    #[test]
    fn unit_parse_local_tui_command_maps_dashboard_tools_routines_cortex_memory_sync_and_colors() {
        assert_eq!(
            super::parse_local_tui_command("/dashboard"),
            Some(super::LocalTuiCommand::Dashboard)
        );
        assert_eq!(
            super::parse_local_tui_command("/tools"),
            Some(super::LocalTuiCommand::Tools)
        );
        assert_eq!(
            super::parse_local_tui_command("/routines"),
            Some(super::LocalTuiCommand::Routines)
        );
        assert_eq!(
            super::parse_local_tui_command("/cortex"),
            Some(super::LocalTuiCommand::Cortex)
        );
        assert_eq!(
            super::parse_local_tui_command("/memory"),
            Some(super::LocalTuiCommand::Memory)
        );
        assert_eq!(
            super::parse_local_tui_command("/memory-distill"),
            Some(super::LocalTuiCommand::Memory)
        );
        assert_eq!(
            super::parse_local_tui_command("/sync"),
            Some(super::LocalTuiCommand::Sync)
        );
        assert_eq!(
            super::parse_local_tui_command("/colors"),
            Some(super::LocalTuiCommand::Colors)
        );
        assert_eq!(super::parse_local_tui_command("/unknown"), None);
    }

    #[test]
    fn unit_apply_gateway_sync_snapshot_updates_dashboard_tools_and_routines_panels() {
        let mut state = super::AgentAppState::default();
        super::apply_gateway_sync_snapshot(
            &mut state,
            super::GatewaySyncSnapshot {
                status_line: "gateway service=running multi_channel=healthy heartbeat=running"
                    .to_string(),
                dashboard_lines: vec!["dashboard health=healthy".to_string()],
                tools_lines: vec!["tools inventory total_tools=4".to_string()],
                routines_lines: vec!["routines jobs total=1 running=1".to_string()],
                cortex_lines: vec!["cortex health=healthy".to_string()],
                memory_lines: vec!["memory distill enabled=true".to_string()],
                event_line: "integration.sync completed".to_string(),
                success: true,
                rate_limited: false,
            },
        );

        assert!(state.gateway_sync_ok);
        assert!(state.gateway_sync_last_at.is_some());
        assert_eq!(
            state.gateway_sync_status,
            "gateway service=running multi_channel=healthy heartbeat=running"
        );
        assert_eq!(
            state.dashboard_lines.first().map(String::as_str),
            Some("dashboard health=healthy")
        );
        assert_eq!(
            state.integration_tools_lines.first().map(String::as_str),
            Some("tools inventory total_tools=4")
        );
        assert_eq!(
            state.integration_routines_lines.first().map(String::as_str),
            Some("routines jobs total=1 running=1")
        );
        assert_eq!(
            state.integration_cortex_lines.first().map(String::as_str),
            Some("cortex health=healthy")
        );
        assert_eq!(
            state.integration_memory_lines.first().map(String::as_str),
            Some("memory distill enabled=true")
        );
    }

    #[test]
    fn unit_gateway_sync_full_refresh_seconds_matches_light_cycle_contract() {
        assert_eq!(
            super::gateway_sync_full_refresh_seconds(),
            (super::GATEWAY_SYNC_INTERVAL_MS * super::GATEWAY_SYNC_FULL_REFRESH_EVERY_LIGHT_CYCLES)
                / 1000
        );
    }

    #[test]
    fn unit_gateway_sync_backoff_wait_ms_escalates_and_caps() {
        assert_eq!(
            super::gateway_sync_backoff_wait_ms(1),
            super::GATEWAY_SYNC_BACKOFF_INTERVAL_MS
        );
        assert_eq!(
            super::gateway_sync_backoff_wait_ms(2),
            super::GATEWAY_SYNC_BACKOFF_INTERVAL_MS * 2
        );
        assert_eq!(
            super::gateway_sync_backoff_wait_ms(3),
            super::GATEWAY_SYNC_BACKOFF_INTERVAL_MS * 4
        );
        assert_eq!(
            super::gateway_sync_backoff_wait_ms(4),
            super::GATEWAY_SYNC_BACKOFF_MAX_INTERVAL_MS
        );
        assert_eq!(
            super::gateway_sync_backoff_wait_ms(9),
            super::GATEWAY_SYNC_BACKOFF_MAX_INTERVAL_MS
        );
    }

    #[test]
    fn integration_gateway_sync_snapshot_full_mode_reflects_status_contract_for_tools_and_cortex() {
        let mut routes = HashMap::new();
        routes.insert(
            "/gateway/status".to_string(),
            serde_json::json!({
                "service": {"service_status":"running"},
                "multi_channel": {"health_state":"healthy"},
                "events": {"reason_code":"events_applied", "queue_depth": 1},
                "runtime_heartbeat": {"run_state":"running"},
                "training": {"run_state":"running", "rollouts_total": 2},
                "gateway": {
                    "dashboard": {
                        "health_endpoint": "/dashboard/health",
                        "widgets_endpoint": "/dashboard/widgets",
                        "alerts_endpoint": "/dashboard/alerts"
                    },
                    "web_ui": {
                        "tools_endpoint": "/gateway/tools",
                        "tool_stats_endpoint": "/gateway/tools/stats",
                        "jobs_endpoint": "/gateway/jobs",
                        "cortex_chat_endpoint": "/cortex/chat",
                        "cortex_status_endpoint": "/cortex/status",
                        "telemetry_runtime": {"total_events": 9},
                        "memory_distill_runtime": {
                            "enabled": true,
                            "in_flight": false,
                            "cycle_count": 4,
                            "writes_applied": 3,
                            "write_failures": 0,
                            "last_cycle_sessions_scanned": 2,
                            "last_cycle_entries_scanned": 4,
                            "last_cycle_candidates_extracted": 3,
                            "last_cycle_writes_applied": 2,
                            "last_cycle_write_failures": 0,
                            "recent_writes": [
                                {
                                    "session_key": "default",
                                    "entry_id": 41,
                                    "memory_id": "auto:default:entry:41:goal:abc123",
                                    "summary": "User goal: ship release",
                                    "memory_type": "goal",
                                    "source_event_key": "session:default:entry:41",
                                    "created": true,
                                    "observed_unix_ms": 1700000001000u64
                                },
                                {
                                    "session_key": "default",
                                    "entry_id": 42,
                                    "memory_id": "auto:default:entry:42:preference:def456",
                                    "summary": "User prefers concise answers",
                                    "memory_type": "preference",
                                    "source_event_key": "session:default:entry:42",
                                    "created": true,
                                    "observed_unix_ms": 1700000002000u64
                                }
                            ],
                            "last_reason_codes": ["distilled_preferences"]
                        }
                    }
                }
            })
            .to_string(),
        );
        routes.insert(
            "/dashboard/health".to_string(),
            serde_json::json!({"run_state":"running"}).to_string(),
        );
        routes.insert(
            "/dashboard/widgets".to_string(),
            serde_json::json!({"widgets":{"queue":{"depth":1}}}).to_string(),
        );
        routes.insert(
            "/dashboard/alerts".to_string(),
            serde_json::json!({"alerts":[]}).to_string(),
        );
        routes.insert(
            "/gateway/tools".to_string(),
            serde_json::json!({
                "total_tools": 2,
                "tools": [{"name":"bash"}, {"name":"memory_search"}]
            })
            .to_string(),
        );
        routes.insert(
            "/gateway/tools/stats".to_string(),
            serde_json::json!({
                "total_events": 4,
                "invalid_records": 0,
                "ui_total_events": 3,
                "ui_invalid_records": 1,
                "diagnostics": [],
                "ui_diagnostics": ["tools_ui_telemetry_malformed_line:3"],
                "stats": [
                    {"tool_name":"bash","event_count":3},
                    {"tool_name":"memory_search","event_count":1}
                ]
            })
            .to_string(),
        );
        routes.insert(
            "/gateway/jobs".to_string(),
            serde_json::json!({
                "total_jobs": 1,
                "jobs": [{"job_id":"job-1","status":"running"}]
            })
            .to_string(),
        );
        routes.insert(
            "/cortex/status".to_string(),
            serde_json::json!({
                "health_state":"healthy",
                "rollout_gate":"open",
                "reason_code":"cortex_ok",
                "diagnostics":[]
            })
            .to_string(),
        );

        let (base_url, shutdown, handle) = spawn_json_fixture_server(routes);
        let snapshot = super::collect_gateway_sync_snapshot(
            base_url.as_str(),
            None,
            super::GatewaySyncFetchMode::Full,
        );
        shutdown.store(true, Ordering::Relaxed);
        let _ = handle.join();

        assert!(
            snapshot.success,
            "expected full sync success: {:?}",
            snapshot
        );
        assert!(snapshot.tools_lines.iter().any(|line| {
            line.contains("tools endpoints inventory=/gateway/tools stats=/gateway/tools/stats")
        }));
        assert!(snapshot
            .tools_lines
            .iter()
            .any(|line| line == "tools metrics"));
        assert!(snapshot
            .tools_lines
            .iter()
            .any(|line| line.contains("runtime") && line.contains("session tool results")));
        assert!(snapshot
            .tools_lines
            .iter()
            .any(|line| line.contains("ui-tools") && line.contains("tools view telemetry")));
        assert!(snapshot
            .tools_lines
            .iter()
            .any(|line| line.contains("ui-all") && line.contains("all dashboard views")));
        assert!(snapshot.cortex_lines.iter().any(|line| {
            line.contains("cortex endpoints chat=/cortex/chat status=/cortex/status")
        }));
        assert!(snapshot.memory_lines.iter().any(|line| line.contains(
            "memory distill last_cycle sessions=2 entries=4 candidates=3 writes=2 write_failures=0"
        )));
        assert!(snapshot
            .memory_lines
            .iter()
            .any(|line| line.contains("memory distill write preference session=default entry=42")));
    }

    #[test]
    fn functional_agent_app_frame_contains_application_sections() {
        let frame = OperatorShellFrame::deterministic_fixture("local-dev".to_string());
        let args = super::AgentArgs {
            width: 96,
            ..agent_args_fixture()
        };
        let summary = super::build_agent_launch_summary_lines(
            &frame,
            &args,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let mut state = super::AgentAppState {
            turn_status: "turn.start timeout=180000ms request_budget=180000ms".to_string(),
            progress_status: "| model thinking (turn 1) | elapsed=2.0s | remaining=178.0s"
                .to_string(),
            ..super::AgentAppState::default()
        };
        state.timeline_lines.push_back(
            "assistant.timeline tool_plan: requested tool_calls=1 tools=[http]".to_string(),
        );
        state
            .assistant_lines
            .push_back("I will fetch the page now.".to_string());
        state
            .event_lines
            .push_back("model catalog: source=built-in entries=31".to_string());
        let lines = super::compose_agent_app_lines(
            &args,
            &summary,
            &state,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let rendered = lines.join("\n");
        for section in [
            "[Session]",
            "[Turn]",
            "[Timeline]",
            "[Assistant]",
            "[Tools]",
            "[Events]",
            "[Input]",
        ] {
            assert!(
                rendered.contains(section),
                "missing section {section}:\n{rendered}"
            );
        }
    }

    #[test]
    fn regression_agent_app_parser_keeps_dash_prefixed_plain_text_out_of_progress() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stdout,
            "--model openai/gpt-5.2",
        );
        assert!(
            state.progress_status.is_empty(),
            "dash-prefixed non-progress lines must not be interpreted as spinner updates"
        );
        assert_eq!(
            state.assistant_lines.back().map(String::as_str),
            Some("--model openai/gpt-5.2")
        );
    }

    #[test]
    fn unit_agent_app_parser_normalizes_legacy_interactive_turn_markers() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "interactive.turn=start turn_timeout_ms=180000 request_timeout_ms=180000 cancel=ctrl_c exit=/quit",
        );
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "interactive.turn=running elapsed_ms=2002 remaining_request_ms=177998 heartbeat=/ phase=model_request status=waiting_for_model_response turn=1",
        );
        assert!(
            state.turn_status.starts_with("turn.start timeout=180000ms"),
            "legacy interactive turn start should normalize into turn.start format"
        );
        assert!(
            state.progress_status.contains("elapsed=2.0s"),
            "legacy running status should normalize into compact progress line"
        );
        assert!(
            state.progress_status.contains("remaining=177.9s"),
            "legacy running status should expose remaining budget in seconds"
        );
    }

    #[test]
    fn regression_legacy_running_line_extracts_inline_assistant_fragment() {
        let mut state = super::AgentAppState::default();
        state.mark_turn_started();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stdout,
            "interactive.turn=running elapsed_ms=10002 remaining_request_ms=169998 phase=model_request detail=model thinking (turn 1)Test. What do you want to test next?",
        );
        assert!(
            state.progress_status.contains("elapsed=10.0s"),
            "expected progress line normalization to preserve elapsed status"
        );
        assert_eq!(
            state.assistant_lines.back().map(String::as_str),
            Some("Test. What do you want to test next?")
        );
    }

    #[test]
    fn regression_progress_line_extracts_inline_assistant_fragment() {
        let mut state = super::AgentAppState::default();
        state.mark_turn_started();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stdout,
            "| waiting_for_model_response [model_request (turn 1)]Test. What do you want to test next? | elapsed=10.0s | remaining=170.0s",
        );
        assert!(
            state.progress_status.contains("elapsed=10.0s"),
            "expected progress status to be captured"
        );
        assert_eq!(
            state.assistant_lines.back().map(String::as_str),
            Some("Test. What do you want to test next?")
        );
    }

    #[test]
    fn regression_agent_app_parser_routes_tool_calls_to_tools_panel() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stdout,
            "{\"tool_call\":{\"id\":\"call_1\",\"name\":\"http\"}}",
        );
        assert_eq!(
            state.tool_lines.back().map(String::as_str),
            Some("{\"tool_call\":{\"id\":\"call_1\",\"name\":\"http\"}}")
        );
        assert!(state.assistant_lines.is_empty());
    }

    #[test]
    fn regression_agent_app_parser_routes_cortex_observer_lines_to_events_and_panels() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "cortex.observer event_type=local.turn.submitted status=appended prompt_chars=7",
        );
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "cortex.observer event_type=local.tool.start status=appended tool_name=http",
        );

        assert_eq!(
            state.event_lines.back().map(String::as_str),
            Some("cortex.observer event_type=local.tool.start status=appended tool_name=http")
        );
        assert_eq!(
            state.timeline_lines.back().map(String::as_str),
            Some("cortex.observer event_type=local.turn.submitted status=appended prompt_chars=7")
        );
        assert_eq!(
            state.tool_lines.back().map(String::as_str),
            Some("cortex.observer event_type=local.tool.start status=appended tool_name=http")
        );
        assert!(state.assistant_lines.is_empty());
    }

    #[test]
    fn regression_agent_app_parser_applies_structured_tau_runtime_events() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            r#"tau.event {"schema_version":1,"timestamp_unix_ms":1,"event_type":"turn.submitted","fields":{"prompt_chars":7,"phase":"queued"}}"#,
        );
        assert_eq!(state.turn_phase, super::TurnPhase::Queued);
        assert!(state.turn_in_progress);
        assert_eq!(state.progress_status, "queued prompt_chars=7");

        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            r#"tau.event {"schema_version":1,"timestamp_unix_ms":2,"event_type":"tool.started","fields":{"tool_name":"http","tool_call_id":"call_1"}}"#,
        );
        assert_eq!(state.turn_phase, super::TurnPhase::Tool);
        assert_eq!(
            state.tool_lines.back().map(String::as_str),
            Some("tau.event tool.started tool_name=http tool_call_id=call_1")
        );

        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            r#"tau.event {"schema_version":1,"timestamp_unix_ms":3,"event_type":"memory.write","fields":{"event_type":"memory.write","tool_name":"memory_write"}}"#,
        );
        assert_eq!(
            state.assistant_lines.back().map(String::as_str),
            Some("memory updated (memory_write)")
        );

        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            r#"tau.event {"schema_version":1,"timestamp_unix_ms":4,"event_type":"turn.completed","fields":{"status":"completed","phase":"done"}}"#,
        );
        assert_eq!(state.turn_phase, super::TurnPhase::Done);
        assert!(!state.turn_in_progress);
    }

    #[test]
    fn regression_failed_turn_event_adds_assistant_failure_line_when_no_answer_text_exists() {
        let mut state = super::AgentAppState::default();
        state.mark_turn_started();
        state.assistant_output_seen_in_turn = false;

        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            r#"tau.event {"schema_version":1,"timestamp_unix_ms":5,"event_type":"turn.failed","fields":{"phase":"failed","error":"invalid response: codex cli failed with status 1 (model=gpt-5.2): Warning: no last agent message"}}"#,
        );

        assert_eq!(state.turn_phase, super::TurnPhase::Failed);
        assert_eq!(
            state.assistant_lines.back().map(String::as_str),
            Some("assistant error: invalid response: codex cli failed with status 1 (model=gpt-5.2): Warning: no last agent message")
        );
    }

    #[test]
    fn regression_agent_app_parser_accepts_embedded_tau_event_wrappers() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            r#"{"event":"tau.event","payload":{"schema_version":1,"timestamp_unix_ms":1,"event_type":"turn.submitted","fields":{"prompt_chars":12,"phase":"queued"}}}"#,
        );
        assert_eq!(state.turn_phase, super::TurnPhase::Queued);
        assert!(state.turn_in_progress);

        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "event: tau.event",
        );
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            r#"data: {"schema_version":1,"timestamp_unix_ms":2,"event_type":"tool.started","fields":{"tool_name":"http","tool_call_id":"call_1"}}"#,
        );
        assert_eq!(state.turn_phase, super::TurnPhase::Tool);
        assert_eq!(
            state.tool_lines.back().map(String::as_str),
            Some("tau.event tool.started tool_name=http tool_call_id=call_1")
        );
    }

    #[test]
    fn regression_agent_app_parser_routes_runtime_json_message_added_to_assistant() {
        let mut state = super::AgentAppState::default();
        state.mark_turn_started();

        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stdout,
            r#"{"role":"assistant","text":"Hello from json events.","tool_calls":0,"type":"message_added"}"#,
        );
        assert_eq!(
            state.assistant_lines.back().map(String::as_str),
            Some("Hello from json events.")
        );

        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stdout,
            r#"{"type":"turn_end","status":"completed"}"#,
        );
        assert_eq!(state.turn_phase, super::TurnPhase::Done);
        assert!(!state.turn_in_progress);
    }

    #[test]
    fn regression_agent_app_parser_emits_auth_fix_hint_on_invalid_api_key() {
        let mut state = super::AgentAppState::default();
        super::update_agent_app_state(
            &mut state,
            super::AgentOutputSource::Stderr,
            "\"code\": \"invalid_api_key\",",
        );
        let rendered = state
            .event_lines
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rendered.contains("auth hint: OpenAI request used API-key auth"));
        assert!(rendered.contains("auth fix: unset OPENAI_API_KEY TAU_API_KEY"));
    }

    #[test]
    fn regression_collect_exit_context_prioritizes_actionable_error_lines() {
        let mut state = super::AgentAppState::default();
        state
            .event_lines
            .push_back("model catalog: source=built-in entries=31".to_string());
        state.event_lines.push_back(
            "Error: invalid response: codex cli failed with status 1 (model=gpt-5.2)".to_string(),
        );
        state
            .event_lines
            .push_back("Please upgrade Node.js: https://nodejs.org/en/download/".to_string());
        state
            .assistant_lines
            .push_back("interactive turn failed: request timed out".to_string());

        let lines = super::collect_exit_context_lines(&state, 4);
        let rendered = lines.join("\n");
        assert!(rendered.contains("invalid response"));
        assert!(rendered.contains("Please upgrade Node.js"));
        assert!(!rendered.contains("model catalog:"));
    }

    #[test]
    fn unit_agent_app_panel_focus_and_expand_controls_are_stable() {
        let mut state = super::AgentAppState::default();
        assert_eq!(state.focused_panel, super::AgentPanel::Assistant);
        state.focus_next_panel();
        assert_eq!(state.focused_panel, super::AgentPanel::Timeline);
        state.focus_previous_panel();
        assert_eq!(state.focused_panel, super::AgentPanel::Assistant);
        state.toggle_expand_focused_panel();
        assert_eq!(state.expanded_panel, Some(super::AgentPanel::Assistant));
        state.toggle_expand_focused_panel();
        assert_eq!(state.expanded_panel, None);
    }

    #[test]
    fn unit_agent_app_panel_scroll_controls_preserve_tail_reset() {
        let mut state = super::AgentAppState::default();
        assert_eq!(
            state.panel_offset(super::AgentPanel::Assistant),
            super::AGENT_PANEL_SCROLL_FOLLOW_TAIL
        );
        state.scroll_focused_panel_up();
        assert!(
            state.panel_offset(super::AgentPanel::Assistant)
                > super::AGENT_PANEL_SCROLL_FOLLOW_TAIL
        );
        state.scroll_focused_panel_down();
        assert_eq!(
            state.panel_offset(super::AgentPanel::Assistant),
            super::AGENT_PANEL_SCROLL_FOLLOW_TAIL
        );
    }

    #[test]
    fn unit_agent_app_split_controls_clamp_to_safe_ranges() {
        let mut state = super::AgentAppState::default();
        for _ in 0..20 {
            state.adjust_split_left_percent(5);
            state.adjust_split_top_percent(5);
        }
        assert_eq!(state.split_left_percent, 80);
        assert_eq!(state.split_top_percent, 85);
        for _ in 0..30 {
            state.adjust_split_left_percent(-5);
            state.adjust_split_top_percent(-5);
        }
        assert_eq!(state.split_left_percent, 35);
        assert_eq!(state.split_top_percent, 55);
    }

    #[test]
    fn unit_escape_sequence_handler_supports_resize_and_page_scroll() {
        let mut state = super::AgentAppState::default();
        assert!(super::handle_escape_sequence(&mut state, "[1;5C"));
        assert_eq!(state.split_left_percent, 71);
        assert!(super::handle_escape_sequence(&mut state, "[1;5D"));
        assert_eq!(state.split_left_percent, 66);
        assert!(super::handle_escape_sequence(&mut state, "[1;5A"));
        assert_eq!(state.split_top_percent, 77);
        assert!(super::handle_escape_sequence(&mut state, "[1;5B"));
        assert_eq!(state.split_top_percent, 72);

        assert!(super::handle_escape_sequence(&mut state, "[5~"));
        assert!(
            state.panel_offset(super::AgentPanel::Assistant) >= super::AGENT_PANEL_PAGE_SCROLL_STEP
        );
        assert!(super::handle_escape_sequence(&mut state, "[6~"));
        assert_eq!(
            state.panel_offset(super::AgentPanel::Assistant),
            super::AGENT_PANEL_SCROLL_FOLLOW_TAIL
        );
        assert!(super::handle_escape_sequence(&mut state, "[H"));
        assert_eq!(
            state.panel_offset(super::AgentPanel::Assistant),
            super::AGENT_PANEL_SCROLL_TO_HEAD
        );
        assert!(super::handle_escape_sequence(&mut state, "[F"));
        assert_eq!(
            state.panel_offset(super::AgentPanel::Assistant),
            super::AGENT_PANEL_SCROLL_FOLLOW_TAIL
        );
    }

    #[test]
    fn functional_agent_app_frame_marks_focused_panel_and_expansion_state() {
        let frame = OperatorShellFrame::deterministic_fixture("local-dev".to_string());
        let args = super::AgentArgs {
            width: 96,
            ..agent_args_fixture()
        };
        let summary = super::build_agent_launch_summary_lines(
            &frame,
            &args,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let mut state = super::AgentAppState::default();
        state.toggle_expand_focused_panel();
        let lines = super::compose_agent_app_lines(
            &args,
            &summary,
            &state,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let rendered = lines.join("\n");
        assert!(rendered.contains("> [Assistant] [expanded]"));
    }

    #[test]
    fn functional_agent_app_frame_renders_shortcuts_panel_when_enabled() {
        let frame = OperatorShellFrame::deterministic_fixture("local-dev".to_string());
        let args = super::AgentArgs {
            width: 120,
            ..agent_args_fixture()
        };
        let summary = super::build_agent_launch_summary_lines(
            &frame,
            &args,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let state = super::AgentAppState {
            show_shortcuts: true,
            ..super::AgentAppState::default()
        };
        let lines = super::compose_agent_app_lines(
            &args,
            &summary,
            &state,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let rendered = lines.join("\n");
        assert!(rendered.contains("[Shortcuts]"));
        assert!(rendered.contains("Ctrl+E expand/collapse focused content panel"));
    }

    #[test]
    fn unit_tools_metrics_table_assigns_status_tones() {
        let lines = super::render_tools_metrics_table(Some(10), Some(0), Some(5), Some(1), 0);
        assert!(
            lines.iter().any(|line| {
                line.contains("runtime")
                    && line.contains("session tool results")
                    && line.contains("ok")
            }),
            "expected runtime row with ok status: {lines:?}"
        );
        assert!(
            lines.iter().any(|line| {
                line.contains("ui-tools")
                    && line.contains("tools view telemetry")
                    && line.contains("warn")
            }),
            "expected ui-tools row with warn status: {lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("ui-all") && line.contains("idle")),
            "expected ui-all row with idle status: {lines:?}"
        );
    }

    #[test]
    fn unit_tools_metrics_row_styling_surfaces_warn_and_bad_status_colors() {
        let warn_line = super::styled_tools_metric_line(
            "ui-tools         5       2 warn    tools view telemetry",
        )
        .expect("tools metrics warn row should render with styles");
        let warn_status_span = warn_line
            .spans
            .get(6)
            .expect("warn row should include status span at index 6");
        assert_eq!(warn_status_span.style.fg, Some(super::Color::Yellow));

        let bad_line = super::styled_tools_metric_line(
            "runtime          2       2 bad     session tool results",
        )
        .expect("tools metrics bad row should render with styles");
        let bad_status_span = bad_line
            .spans
            .get(6)
            .expect("bad row should include status span at index 6");
        assert_eq!(bad_status_span.style.fg, Some(super::Color::LightRed));
    }

    #[test]
    fn unit_integration_status_line_styling_applies_good_warn_bad_tones() {
        let good_line = super::styled_panel_line(
            super::AgentPanel::Assistant,
            "  cortex health=healthy rollout_gate=open reason=cortex_ok",
            super::ToneMode::Semantic,
        );
        assert_eq!(
            good_line.spans.first().and_then(|span| span.style.fg),
            Some(super::Color::LightGreen)
        );

        let warn_line = super::styled_panel_line(
            super::AgentPanel::Events,
            "dashboard health=unknown control=running run_state=unknown queue=0",
            super::ToneMode::Semantic,
        );
        assert_eq!(
            warn_line.spans.first().and_then(|span| span.style.fg),
            Some(super::Color::Yellow)
        );

        let bad_line = super::styled_panel_line(
            super::AgentPanel::Tools,
            "memory distill enabled=true in_flight=false cycles=3 writes=2 write_failures=1",
            super::ToneMode::Semantic,
        );
        assert_eq!(
            bad_line.spans.first().and_then(|span| span.style.fg),
            Some(super::Color::LightRed)
        );
    }

    #[test]
    fn functional_agent_app_frame_tools_panel_includes_routines_cortex_and_memory_snapshots() {
        let frame = OperatorShellFrame::deterministic_fixture("local-dev".to_string());
        let args = super::AgentArgs {
            width: 120,
            ..agent_args_fixture()
        };
        let summary = super::build_agent_launch_summary_lines(
            &frame,
            &args,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let state = super::AgentAppState {
            integration_tools_lines: vec!["tools inventory total_tools=4".to_string()],
            integration_routines_lines: vec!["routines jobs total=1 running=1".to_string()],
            integration_cortex_lines: vec![
                "cortex health=healthy rollout_gate=open reason=cortex_ok".to_string(),
            ],
            integration_memory_lines: vec![
                "memory distill enabled=true in_flight=false cycles=3 writes=2 write_failures=0"
                    .to_string(),
            ],
            ..super::AgentAppState::default()
        };

        let lines = super::compose_agent_app_lines(
            &args,
            &summary,
            &state,
            "target/debug/tau-coding-agent --model openai/gpt-5.2",
        );
        let rendered = lines.join("\n");
        assert!(rendered.contains("routines"));
        assert!(rendered.contains("cortex"));
        assert!(rendered.contains("memory"));
        assert!(rendered.contains("cortex health=healthy rollout_gate=open reason=cortex_ok"));
        assert!(rendered.contains("memory distill enabled=true in_flight=false"));
        assert!(rendered.contains("cycles=3"));
        assert!(rendered.contains("writes=2"));
        assert!(rendered.contains("write_failures=0"));
    }

    #[test]
    fn unit_local_colors_command_toggles_tone_mode() {
        let mut state = super::AgentAppState::default();
        assert_eq!(state.tone_mode, super::ToneMode::Semantic);
        super::handle_local_tui_command(&mut state, super::LocalTuiCommand::Colors, None);
        assert_eq!(state.tone_mode, super::ToneMode::Minimal);
        assert!(state
            .assistant_lines
            .back()
            .map(String::as_str)
            .unwrap_or_default()
            .contains("color mode: minimal"));
        super::handle_local_tui_command(&mut state, super::LocalTuiCommand::Colors, None);
        assert_eq!(state.tone_mode, super::ToneMode::Semantic);
    }
}
