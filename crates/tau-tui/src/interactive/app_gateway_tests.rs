use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use super::{
    app::{App, AppConfig},
    app_commands,
    chat::MessageRole,
    gateway_client::GatewayRuntimeConfig,
    status::AgentStateDisplay,
};

fn set_input(app: &mut App, text: &str) {
    app.input.clear();
    for ch in text.chars() {
        app.input.insert_char(ch);
    }
}

fn wait_for_turn(app: &mut App) {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(2) {
        app.tick();
        if app.status.agent_state != AgentStateDisplay::Thinking {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for gateway-backed turn to complete");
}

fn last_message(app: &App, role: MessageRole) -> Option<&str> {
    app.chat
        .messages()
        .iter()
        .rev()
        .find(|message| message.role == role)
        .map(|message| message.content.as_str())
}

fn build_app(bind: String) -> App {
    App::new(AppConfig {
        gateway: GatewayRuntimeConfig {
            base_url: format!("http://{bind}"),
            ..GatewayRuntimeConfig::default()
        },
        ..AppConfig::default()
    })
}

fn spawn_gateway_server(status_line: &str, body: &str) -> (String, Arc<Mutex<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test gateway");
    let addr = listener.local_addr().expect("local addr");
    let request_capture = Arc::new(Mutex::new(String::new()));
    let request_capture_thread = Arc::clone(&request_capture);
    let status_line = status_line.to_string();
    let body = body.to_string();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept gateway request");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("set timeout");
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => {
                    buffer.extend_from_slice(&chunk[..count]);
                    if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        *request_capture_thread.lock().expect("capture request") =
            String::from_utf8_lossy(&buffer).to_string();

        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write gateway response");
    });

    (addr.to_string(), request_capture)
}

#[test]
fn red_spec_3616_submit_input_uses_gateway_response_instead_of_local_echo() {
    let (bind, request_capture) = spawn_gateway_server(
        "200 OK",
        r#"{"output_text":"runtime ok","usage":{"total_tokens":42}}"#,
    );
    let mut app = build_app(bind);
    set_input(&mut app, "create a snake game");

    app_commands::submit_input(&mut app);
    wait_for_turn(&mut app);

    let assistant = last_message(&app, MessageRole::Assistant).unwrap_or_default();
    let request = request_capture.lock().expect("request capture").clone();
    assert_eq!(assistant, "runtime ok");
    assert!(
        !app.chat
            .messages()
            .iter()
            .any(|message| message.content.contains("Received your message")),
        "messages={:?}",
        app.chat
            .messages()
            .iter()
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
    );
    assert!(request.contains("POST /v1/responses"), "request={request}");
    assert!(request.contains("create a snake game"), "request={request}");
    assert!(request.contains("session_id"), "request={request}");
}

#[test]
fn red_spec_3616_submit_input_surfaces_gateway_errors_loudly() {
    let (bind, _) = spawn_gateway_server(
        "502 Bad Gateway",
        r#"{"error":{"message":"gateway runtime failed: test failure"}}"#,
    );
    let mut app = build_app(bind);
    set_input(&mut app, "create a snake game");

    app_commands::submit_input(&mut app);
    wait_for_turn(&mut app);

    let system = last_message(&app, MessageRole::System).unwrap_or_default();
    assert!(system.contains("gateway runtime failed: test failure"));
    assert_eq!(app.status.agent_state, AgentStateDisplay::Error);
}

#[test]
fn red_spec_3618_matching_prompt_surfaces_active_skill_name_in_rendered_tui() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime_skills_dir = temp.path().join(".tau/skills");
    let bundled_skills_dir = temp.path().join("skills");
    std::fs::create_dir_all(&runtime_skills_dir).expect("create runtime skills dir");
    std::fs::create_dir_all(&bundled_skills_dir).expect("create bundled skills dir");
    std::fs::write(
        bundled_skills_dir.join("web-game-phaser.md"),
        "---\nname: web-game-phaser\ndescription: Build Phaser web games.\n---\nUse Phaser 3 and validate a playable game loop.\n",
    )
    .expect("write bundled skill");

    let mut app = App::new(AppConfig {
        skills_dir: runtime_skills_dir,
        bundled_skills_dir: Some(bundled_skills_dir),
        ..AppConfig::default()
    });
    app.update_active_skills_for_prompt("create a snake and tetris mashup game using phaserjs")
        .expect("update active skills");

    let backend = ratatui::backend::TestBackend::new(120, 24);
    let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| super::ui::render(frame, &app))
        .expect("draw");
    let buffer = terminal.backend().buffer().clone();
    let mut rendered = String::new();
    for y in 0..24 {
        for x in 0..120 {
            rendered.push_str(buffer.cell((x, y)).expect("cell").symbol());
        }
        rendered.push('\n');
    }

    assert!(
        rendered.contains("Skills: web-game-phaser"),
        "expected active skill visibility in tui render, rendered:\n{rendered}"
    );
}

#[test]
fn red_spec_3618_non_matching_prompt_omits_active_skill_label() {
    let mut app = App::new(AppConfig {
        skills_dir: PathBuf::from(".tau/skills"),
        bundled_skills_dir: Some(PathBuf::from("skills")),
        ..AppConfig::default()
    });
    app.update_active_skills_for_prompt("explain the release process")
        .expect("update active skills");

    let backend = ratatui::backend::TestBackend::new(120, 24);
    let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| super::ui::render(frame, &app))
        .expect("draw");
    let buffer = terminal.backend().buffer().clone();
    let mut rendered = String::new();
    for y in 0..24 {
        for x in 0..120 {
            rendered.push_str(buffer.cell((x, y)).expect("cell").symbol());
        }
        rendered.push('\n');
    }

    assert!(
        !rendered.contains("Skills:"),
        "expected no active skill label for non-match, rendered:\n{rendered}"
    );
}
