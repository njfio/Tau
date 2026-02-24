use std::{env, path::Path, thread, time::Duration};

use tau_tui::{
    apply_overlay, render_operator_shell_frame, Component, DiffRenderer, EditorBuffer, EditorView,
    LumaImage, OperatorShellFrame, Text, Theme, ThemeRole,
};

const HELP: &str = "\
tau-tui operator terminal

Usage:
  cargo run -p tau-tui -- [demo] [--frames N] [--width N] [--sleep-ms N] [--no-color]
  cargo run -p tau-tui -- shell [--width N] [--profile NAME] [--no-color]
  cargo run -p tau-tui -- shell-live [--state-dir PATH] [--width N] [--profile NAME] [--no-color]

Options:
  demo          Animated rendering demo mode (default command)
  shell         Operator shell mode with status/auth/training panels
  shell-live    State-backed operator shell mode from dashboard artifacts
  --frames N    Demo: number of frames to render (default: 3, min: 1)
  --width N     Demo/Shell: render width in characters (demo default: 72, shell default: 88)
  --sleep-ms N  Demo: delay between frames in milliseconds (default: 120)
  --profile N   Shell: operator profile label (default: local-dev)
  --state-dir P Shell-live: dashboard state directory (default: .tau/dashboard)
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
    color: bool,
}

impl Default for LiveShellArgs {
    fn default() -> Self {
        Self {
            width: 88,
            profile: "local-dev".to_string(),
            state_dir: ".tau/dashboard".to_string(),
            color: true,
        }
    }
}

#[derive(Debug)]
enum ParseAction {
    RunDemo(DemoArgs),
    RunShell(ShellArgs),
    RunShellLive(LiveShellArgs),
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
            "--state-dir" => {
                let raw = it.next().ok_or("missing value for --state-dir")?;
                let value = raw.trim();
                if value.is_empty() {
                    return Err("--state-dir must not be empty".to_string());
                }
                parsed.state_dir = value.to_string();
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(ParseAction::RunShellLive(parsed))
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

fn run_shell_live(args: LiveShellArgs) {
    let theme = Theme::default();
    let frame = OperatorShellFrame::from_dashboard_state_dir(
        args.profile.clone(),
        Path::new(args.state_dir.as_str()),
    );
    let rendered = render_operator_shell_frame(&frame, args.width);
    let header = paint(
        &theme,
        ThemeRole::Accent,
        format!(
            "Tau Operator Shell (live) - profile={} env={} state_dir={}",
            frame.profile, frame.environment, args.state_dir
        ),
        args.color,
    );
    println!("{header}");
    for line in rendered {
        println!("{}", paint(&theme, ThemeRole::Primary, line, args.color));
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
    }
}

#[cfg(test)]
mod tests {
    use super::{compose_frame, parse_args, ParseAction};
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
}
