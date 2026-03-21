use std::{env, path::Path, process::Command, process::Stdio, thread, time::Duration};

use tau_tui::{
    apply_overlay,
    interactive::{run_interactive, AppConfig, GatewayRuntimeConfig},
    render_operator_shell_frame, Component, DiffRenderer, EditorBuffer, EditorView, LumaImage,
    OperatorShellFrame, Text, Theme, ThemeRole,
};

const HELP: &str = "\
tau-tui operator terminal

Usage:
  cargo run -p tau-tui -- [demo] [--frames N] [--width N] [--sleep-ms N] [--no-color]
  cargo run -p tau-tui -- interactive [--model ID] [--profile NAME] [--bind HOST:PORT] [--auth-mode MODE] [--auth-token TOKEN] [--request-timeout-ms N] [--session-key KEY]
  cargo run -p tau-tui -- shell [--width N] [--profile NAME] [--no-color]
  cargo run -p tau-tui -- shell-live [--state-dir PATH] [--width N] [--profile NAME] [--watch] [--iterations N] [--interval-ms N] [--no-color]
  cargo run -p tau-tui -- agent [--dashboard-state-dir PATH] [--gateway-state-dir PATH] [--model ID] [--request-timeout-ms N] [--agent-request-max-retries N] [--width N] [--profile NAME] [--dry-run] [--no-color]

Options:
  demo          Animated rendering demo mode (default command)
  interactive   Full-screen interactive TUI with chat, tools, status bar, and vim keybindings
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
  --model ID    Agent: model id for interactive runtime (default: gpt-5.3-codex)
  --bind P      Interactive: gateway bind host:port (default: 127.0.0.1:8791)
  --auth-mode M Interactive: gateway auth mode localhost-dev|token (default: localhost-dev)
  --auth-token T Interactive: bearer token when --auth-mode=token
  --session-key K Interactive: gateway session key (default: default)
  --request-timeout-ms N Interactive/Agent: request timeout in milliseconds for gateway/runtime calls
  --agent-request-max-retries N Agent: max model request retries forwarded to tau-coding-agent
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
    request_timeout_ms: Option<u64>,
    agent_request_max_retries: Option<usize>,
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
            model: "gpt-5.3-codex".to_string(),
            request_timeout_ms: None,
            agent_request_max_retries: None,
            dry_run: false,
            color: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InteractiveArgs {
    model: String,
    profile: String,
    bind: String,
    auth_mode: String,
    auth_token: Option<String>,
    request_timeout_ms: u64,
    session_key: String,
}

impl Default for InteractiveArgs {
    fn default() -> Self {
        Self {
            model: "gpt-5.3-codex".to_string(),
            profile: "local-dev".to_string(),
            bind: "127.0.0.1:8791".to_string(),
            auth_mode: "localhost-dev".to_string(),
            auth_token: None,
            request_timeout_ms: 180_000,
            session_key: "default".to_string(),
        }
    }
}

#[derive(Debug)]
enum ParseAction {
    RunDemo(DemoArgs),
    RunInteractive(InteractiveArgs),
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
        Some("interactive") => {
            values.remove(0);
            parse_interactive_args(values)
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

fn parse_interactive_args(args: Vec<String>) -> Result<ParseAction, String> {
    let mut parsed = InteractiveArgs::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(ParseAction::Help),
            "--model" => {
                let raw = it.next().ok_or("missing value for --model")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--model must not be empty".to_string());
                }
                parsed.model = value.to_string();
            }
            "--profile" => {
                let raw = it.next().ok_or("missing value for --profile")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--profile must not be empty".to_string());
                }
                parsed.profile = value.to_string();
            }
            "--bind" => {
                let raw = it.next().ok_or("missing value for --bind")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--bind must not be empty".to_string());
                }
                parsed.bind = value.to_string();
            }
            "--auth-mode" => {
                let raw = it.next().ok_or("missing value for --auth-mode")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--auth-mode must not be empty".to_string());
                }
                parsed.auth_mode = value.to_string();
            }
            "--auth-token" => {
                let raw = it.next().ok_or("missing value for --auth-token")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--auth-token must not be empty".to_string());
                }
                parsed.auth_token = Some(value.to_string());
            }
            "--request-timeout-ms" => {
                let raw = it.next().ok_or("missing value for --request-timeout-ms")?;
                let value = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid u64 for --request-timeout-ms: {raw}"))?;
                if value == 0 {
                    return Err("--request-timeout-ms must be >= 1".to_string());
                }
                parsed.request_timeout_ms = value;
            }
            "--session-key" => {
                let raw = it.next().ok_or("missing value for --session-key")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--session-key must not be empty".to_string());
                }
                parsed.session_key = value.to_string();
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    match parsed.auth_mode.as_str() {
        "localhost-dev" => {}
        "token" => {
            if parsed.auth_token.is_none() {
                return Err("--auth-token is required when --auth-mode=token".to_string());
            }
        }
        _ => {
            return Err(format!(
                "invalid --auth-mode: {} (expected localhost-dev|token)",
                parsed.auth_mode
            ));
        }
    }
    Ok(ParseAction::RunInteractive(parsed))
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
    let mut command = vec![
        "cargo".to_string(),
        "run".to_string(),
        "-p".to_string(),
        "tau-coding-agent".to_string(),
        "--".to_string(),
        "--model".to_string(),
        args.model.clone(),
        "--gateway-state-dir".to_string(),
        args.gateway_state_dir.clone(),
        "--dashboard-state-dir".to_string(),
        args.dashboard_state_dir.clone(),
    ];
    if let Some(timeout_ms) = args.request_timeout_ms {
        command.push("--request-timeout-ms".to_string());
        command.push(timeout_ms.to_string());
    }
    if let Some(max_retries) = args.agent_request_max_retries {
        command.push("--agent-request-max-retries".to_string());
        command.push(max_retries.to_string());
    }
    command
}

fn format_shell_command(tokens: &[String]) -> String {
    tokens.join(" ")
}

fn run_agent(args: AgentArgs) -> Result<(), String> {
    let theme = Theme::default();
    let frame = OperatorShellFrame::from_dashboard_state_dir(
        args.profile.clone(),
        Path::new(args.dashboard_state_dir.as_str()),
    );
    let rendered = render_operator_shell_frame(&frame, args.width);
    let command_tokens = build_agent_runtime_command(&args);
    let launch_command = format_shell_command(&command_tokens);

    let header = paint(
        &theme,
        ThemeRole::Accent,
        format!(
            "Tau Operator Shell (agent-interactive) - profile={} env={} dashboard_state_dir={}",
            frame.profile, frame.environment, args.dashboard_state_dir
        ),
        args.color,
    );
    println!("{header}");
    println!(
        "{}",
        paint(
            &theme,
            ThemeRole::Muted,
            "interactive.launch=ready mode=agent-interactive".to_string(),
            args.color
        )
    );
    println!(
        "{}",
        paint(
            &theme,
            ThemeRole::Muted,
            format!("interactive.command={launch_command}"),
            args.color
        )
    );
    for line in rendered {
        println!("{}", paint(&theme, ThemeRole::Primary, line, args.color));
    }

    if args.dry_run {
        return Ok(());
    }

    let (program, remaining_args) = command_tokens
        .split_first()
        .ok_or_else(|| "interactive runtime command is empty".to_string())?;
    let status = Command::new(program)
        .args(remaining_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("failed to launch interactive runtime: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("interactive runtime exited with status {status}"))
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
        ParseAction::RunInteractive(args) => {
            let config = AppConfig {
                model: args.model,
                profile: args.profile,
                tick_rate_ms: 100,
                gateway: GatewayRuntimeConfig {
                    base_url: format!("http://{}", args.bind),
                    auth_token: args.auth_token,
                    session_key: args.session_key,
                    request_timeout_ms: args.request_timeout_ms,
                },
            };
            if let Err(err) = run_interactive(config) {
                eprintln!("interactive TUI error: {err}");
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
    use tau_tui::{render_operator_shell_frame, EditorBuffer, LumaImage, OperatorShellFrame};

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
            "gpt-5.3-codex".to_string(),
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
        assert_eq!(args.model, "gpt-5.3-codex");
        assert_eq!(args.dashboard_state_dir, ".tau/custom-dashboard");
        assert_eq!(args.gateway_state_dir, ".tau/custom-gateway");
        assert!(args.dry_run);
        assert!(!args.color);
    }

    #[test]
    fn functional_spec_c05_build_agent_runtime_command_contract_is_stable() {
        let args = super::AgentArgs {
            width: 88,
            profile: "ops-interactive".to_string(),
            dashboard_state_dir: ".tau/custom-dashboard".to_string(),
            gateway_state_dir: ".tau/custom-gateway".to_string(),
            model: "gpt-5.3-codex".to_string(),
            request_timeout_ms: None,
            agent_request_max_retries: None,
            dry_run: true,
            color: false,
        };
        let command = super::build_agent_runtime_command(&args);
        assert_eq!(command[0], "cargo");
        assert_eq!(command[1], "run");
        assert_eq!(command[3], "tau-coding-agent");
        assert!(command.contains(&"--model".to_string()));
        assert!(command.contains(&"gpt-5.3-codex".to_string()));
        assert!(command.contains(&"--dashboard-state-dir".to_string()));
        assert!(command.contains(&".tau/custom-dashboard".to_string()));
        assert!(command.contains(&"--gateway-state-dir".to_string()));
        assert!(command.contains(&".tau/custom-gateway".to_string()));
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
        assert!(HELP.contains("--request-timeout-ms"));
        assert!(HELP.contains("--agent-request-max-retries"));
        assert!(HELP.contains("--dry-run"));
    }

    #[test]
    fn regression_spec_c06_agent_mode_defaults_to_gpt53_codex() {
        let action = parse_args(vec!["tau-tui".to_string(), "agent".to_string()])
            .expect("expected parse success");
        let ParseAction::RunAgent(args) = action else {
            panic!("expected agent action");
        };
        assert_eq!(args.model, "gpt-5.3-codex");
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
            width: 88,
            profile: "ops-interactive".to_string(),
            dashboard_state_dir: ".tau/custom-dashboard".to_string(),
            gateway_state_dir: ".tau/custom-gateway".to_string(),
            model: "gpt-5.3-codex".to_string(),
            request_timeout_ms: Some(45_000),
            agent_request_max_retries: Some(0),
            dry_run: true,
            color: false,
        };
        let command = super::build_agent_runtime_command(&args);
        assert!(command.contains(&"--request-timeout-ms".to_string()));
        assert!(command.contains(&"45000".to_string()));
        assert!(command.contains(&"--agent-request-max-retries".to_string()));
        assert!(command.contains(&"0".to_string()));
    }

    #[test]
    fn red_spec_3616_parse_args_accepts_interactive_gateway_flags() {
        let action = parse_args(vec![
            "tau-tui".to_string(),
            "interactive".to_string(),
            "--model".to_string(),
            "gpt-5.3-codex".to_string(),
            "--profile".to_string(),
            "ops-interactive".to_string(),
            "--bind".to_string(),
            "127.0.0.1:8899".to_string(),
            "--auth-mode".to_string(),
            "token".to_string(),
            "--auth-token".to_string(),
            "tok_test".to_string(),
            "--request-timeout-ms".to_string(),
            "45000".to_string(),
            "--session-key".to_string(),
            "session-alpha".to_string(),
        ])
        .expect("expected interactive parse success");

        let ParseAction::RunInteractive(args) = action else {
            panic!("expected interactive action");
        };
        assert_eq!(args.bind, "127.0.0.1:8899");
        assert_eq!(args.auth_mode, "token");
        assert_eq!(args.auth_token.as_deref(), Some("tok_test"));
        assert_eq!(args.request_timeout_ms, 45_000);
        assert_eq!(args.session_key, "session-alpha");
    }

    #[test]
    fn red_spec_3616_parse_args_rejects_interactive_token_mode_without_token() {
        let err = parse_args(vec![
            "tau-tui".to_string(),
            "interactive".to_string(),
            "--auth-mode".to_string(),
            "token".to_string(),
        ])
        .expect_err("expected interactive parse failure");
        assert!(err.contains("--auth-token is required"));
    }
}
