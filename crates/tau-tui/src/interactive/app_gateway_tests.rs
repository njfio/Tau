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

fn spawn_scripted_gateway_server(
    responses: Vec<(&str, &str)>,
) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind scripted gateway");
    let addr = listener.local_addr().expect("local addr");
    let request_capture = Arc::new(Mutex::new(Vec::new()));
    let request_capture_thread = Arc::clone(&request_capture);
    let scripted_responses = responses
        .into_iter()
        .map(|(status, body)| (status.to_string(), body.to_string()))
        .collect::<Vec<_>>();

    thread::spawn(move || {
        for (status_line, body) in scripted_responses {
            let (mut stream, _) = listener.accept().expect("accept scripted gateway request");
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .expect("set timeout");
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 1024];
            let mut header_len = None::<usize>;
            let mut expected_total_len = None::<usize>;
            loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(count) => {
                        buffer.extend_from_slice(&chunk[..count]);
                        if header_len.is_none() {
                            header_len = buffer
                                .windows(4)
                                .position(|window| window == b"\r\n\r\n")
                                .map(|index| index + 4);
                            if let Some(header_len) = header_len {
                                let headers = String::from_utf8_lossy(&buffer[..header_len]);
                                let content_length = headers
                                    .lines()
                                    .find_map(|line| {
                                        let (name, value) = line.split_once(':')?;
                                        if name.eq_ignore_ascii_case("content-length") {
                                            value.trim().parse::<usize>().ok()
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(0);
                                expected_total_len = Some(header_len + content_length);
                            }
                        }
                        if let Some(expected_total_len) = expected_total_len {
                            if buffer.len() >= expected_total_len {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            request_capture_thread
                .lock()
                .expect("capture scripted request")
                .push(String::from_utf8_lossy(&buffer).to_string());

            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write scripted gateway response");
        }
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
        .draw(|frame| super::ui::render(frame, &mut app))
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
        .draw(|frame| super::ui::render(frame, &mut app))
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

#[test]
fn red_spec_3659_command_missions_lists_persisted_mission_summaries() {
    let (bind, _) = spawn_scripted_gateway_server(vec![(
        "200 OK",
        r#"{"missions":[{"mission_id":"checkpoint-alpha","session_key":"session-alpha","status":"checkpointed","goal_summary":"build the game scaffold","latest_output_summary":"scaffolded the first gameplay slice","iteration_count":1,"updated_unix_ms":220,"latest_verifier":{"status":"passed","reason_code":"mutation_evidence_observed","message":"observed workspace mutation"},"latest_completion":{"status":"partial","summary":"scaffolded the first gameplay slice","next_step":"run validation"}}],"limit":20}"#,
    )]);
    let mut app = build_app(bind);
    set_input(&mut app, "/missions");

    app_commands::submit_input(&mut app);

    let system = last_message(&app, MessageRole::System).unwrap_or_default();
    assert!(system.contains("Recent missions:"));
    assert!(system.contains("checkpoint-alpha [checkpointed]"));
    assert!(system.contains("session=session-alpha"));
}

#[test]
fn red_spec_3659_resume_command_binds_active_mission_and_surfaces_status() {
    let (bind, _) = spawn_scripted_gateway_server(vec![(
        "200 OK",
        r#"{"mission":{"mission_id":"checkpoint-alpha","session_key":"session-alpha","status":"checkpointed","goal_summary":"build the game scaffold","latest_output_summary":"scaffolded the first gameplay slice","iteration_count":1,"updated_unix_ms":220,"latest_verifier":{"status":"passed","reason_code":"mutation_evidence_observed","message":"observed workspace mutation"},"latest_completion":{"status":"partial","summary":"scaffolded the first gameplay slice","next_step":"run validation"}}}"#,
    )]);
    let mut app = build_app(bind);
    set_input(&mut app, "/resume checkpoint-alpha");

    app_commands::submit_input(&mut app);

    assert_eq!(
        app.config.gateway.mission_id.as_deref(),
        Some("checkpoint-alpha")
    );
    assert_eq!(app.config.gateway.session_key, "session-alpha");
    assert_eq!(
        app.status.active_mission_id.as_deref(),
        Some("checkpoint-alpha")
    );
    let system = last_message(&app, MessageRole::System).unwrap_or_default();
    assert!(system.contains("Resumed mission checkpoint-alpha"));
    assert!(system.contains("next step: run validation"));

    let backend = ratatui::backend::TestBackend::new(120, 24);
    let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| super::ui::render(frame, &mut app))
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
        rendered.contains("Mission: checkpoint-alpha"),
        "expected active mission visibility in tui render, rendered:\n{rendered}"
    );
}

#[test]
fn red_spec_3659_resumed_turn_includes_mission_id_and_session_id_metadata() {
    let (bind, request_capture) = spawn_scripted_gateway_server(vec![
        (
            "200 OK",
            r#"{"mission":{"mission_id":"checkpoint-alpha","session_key":"session-alpha","status":"checkpointed","goal_summary":"build the game scaffold","latest_output_summary":"scaffolded the first gameplay slice","iteration_count":1,"updated_unix_ms":220,"latest_verifier":{"status":"passed","reason_code":"mutation_evidence_observed","message":"observed workspace mutation"},"latest_completion":{"status":"partial","summary":"scaffolded the first gameplay slice","next_step":"run validation"}}}"#,
        ),
        (
            "200 OK",
            r#"{"output_text":"runtime ok","usage":{"total_tokens":42}}"#,
        ),
    ]);
    let mut app = build_app(bind);
    set_input(&mut app, "/resume checkpoint-alpha");
    app_commands::submit_input(&mut app);

    set_input(&mut app, "continue from the checkpoint");
    app_commands::submit_input(&mut app);
    wait_for_turn(&mut app);

    let requests = request_capture.lock().expect("request capture").clone();
    assert_eq!(requests.len(), 2, "requests={requests:?}");
    assert!(
        requests[0].contains("GET /gateway/missions/checkpoint-alpha"),
        "request={}",
        requests[0]
    );
    assert!(
        requests[1].contains("\"mission_id\":\"checkpoint-alpha\""),
        "request={}",
        requests[1]
    );
    assert!(
        requests[1].contains("\"session_id\":\"session-alpha\""),
        "request={}",
        requests[1]
    );
}
