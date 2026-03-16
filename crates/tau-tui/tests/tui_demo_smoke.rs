use std::process::Command;

#[test]
fn functional_tui_demo_binary_renders_single_frame_without_color() {
    let binary = env!("CARGO_BIN_EXE_tau-tui");
    let output = Command::new(binary)
        .args([
            "--frames",
            "1",
            "--sleep-ms",
            "0",
            "--width",
            "48",
            "--no-color",
        ])
        .output()
        .expect("binary executes");
    assert!(
        output.status.success(),
        "status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tau TUI Demo - frame 1/1"));
    assert!(stdout.contains("op:update"));
}

#[test]
fn integration_tui_demo_binary_renders_multiple_frames() {
    let binary = env!("CARGO_BIN_EXE_tau-tui");
    let output = Command::new(binary)
        .args([
            "--frames",
            "2",
            "--sleep-ms",
            "0",
            "--width",
            "56",
            "--no-color",
        ])
        .output()
        .expect("binary executes");
    assert!(
        output.status.success(),
        "status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tau TUI Demo - frame 1/2"));
    assert!(stdout.contains("Tau TUI Demo - frame 2/2"));
}

#[test]
fn regression_tui_demo_binary_rejects_invalid_frames_argument() {
    let binary = env!("CARGO_BIN_EXE_tau-tui");
    let output = Command::new(binary)
        .args(["--frames", "0"])
        .output()
        .expect("binary executes");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--frames must be >= 1"));
}

#[test]
fn conformance_tui_shell_binary_renders_operator_panels() {
    let binary = env!("CARGO_BIN_EXE_tau-tui");
    let output = Command::new(binary)
        .args([
            "shell",
            "--width",
            "64",
            "--profile",
            "ops-staging",
            "--no-color",
        ])
        .output()
        .expect("binary executes");
    assert!(
        output.status.success(),
        "status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tau Operator Shell"));
    assert!(stdout.contains("STATUS"));
    assert!(stdout.contains("AUTH"));
    assert!(stdout.contains("TRAINING"));
    assert!(stdout.contains("ALERTS"));
    assert!(stdout.contains("ACTIONS"));
}

#[test]
fn conformance_tui_agent_mode_dry_run_emits_interactive_launch_contract() {
    let binary = env!("CARGO_BIN_EXE_tau-tui");
    let output = Command::new(binary)
        .args([
            "agent",
            "--profile",
            "ops-interactive",
            "--model",
            "openai/gpt-5.2",
            "--dashboard-state-dir",
            ".tau/dashboard",
            "--gateway-state-dir",
            ".tau/gateway",
            "--dry-run",
            "--no-color",
        ])
        .output()
        .expect("binary executes");
    assert!(
        output.status.success(),
        "status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tau Operator Shell (agent-interactive)"));
    assert!(stdout.contains("interactive.launch=ready"));
    assert!(stdout.contains("cargo run -p tau-coding-agent --"));
    assert!(stdout.contains("--model openai/gpt-5.2"));
    assert!(stdout.contains("--dashboard-state-dir .tau/dashboard"));
    assert!(stdout.contains("--gateway-state-dir .tau/gateway"));
}

#[test]
fn integration_tui_interactive_mode_fails_loud_without_tty() {
    let binary = env!("CARGO_BIN_EXE_tau-tui");
    let output = Command::new(binary)
        .args(["interactive", "--profile", "ops-interactive"])
        .output()
        .expect("binary executes");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("interactive TUI error:"));
}
