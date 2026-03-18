use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use ratatui::{backend::TestBackend, Terminal};

use super::app::{App, AppConfig};
use super::gateway::GatewayInteractiveConfig;
use super::status::AgentStateDisplay;
use super::ui::render;

#[test]
fn integration_spec_3582_gateway_runtime_streams_operator_state_into_app() {
    let server = spawn_sse_server(vec![
        "event: response.created\n\
data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_3\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"in_progress\",\"phase\":\"model\",\"response_id\":\"resp_3\"}}\n\n"
            .to_string(),
        "event: response.output_text.delta\n\
data: {\"type\":\"response.output_text.delta\",\"response_id\":\"resp_3\",\"delta\":\"hello \",\"operator_state\":{\"entity\":\"artifact\",\"status\":\"streaming\",\"artifact_kind\":\"assistant_output_text\",\"response_id\":\"resp_3\"}}\n\n"
            .to_string(),
        "event: response.output_text.done\n\
data: {\"type\":\"response.output_text.done\",\"response_id\":\"resp_3\",\"text\":\"hello world\",\"operator_state\":{\"entity\":\"artifact\",\"status\":\"completed\",\"artifact_kind\":\"assistant_output_text\",\"response_id\":\"resp_3\"}}\n\n"
            .to_string(),
        "event: response.completed\n\
data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_3\",\"output_text\":\"hello world\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"completed\",\"phase\":\"done\"}}\n\n"
            .to_string(),
    ]);
    let mut app = App::new(AppConfig {
        model: "openai/gpt-5.2".to_string(),
        profile: "ops-interactive".to_string(),
        session_key: "default".to_string(),
        workspace_label: "rust_pi-3583".to_string(),
        approval_mode: "ask".to_string(),
        tick_rate_ms: 25,
        gateway: Some(GatewayInteractiveConfig {
            base_url: server.base_url,
            auth_token: None,
            session_key: "default".to_string(),
            request_timeout_ms: 3_000,
        }),
    });

    for ch in "testing".chars() {
        app.input.insert_char(ch);
    }
    app.submit_input();

    wait_for(|| {
        app.pump_gateway_events();
        app.chat
            .messages()
            .iter()
            .any(|message| message.content.contains("hello "))
    });
    assert_eq!(app.status.agent_state, AgentStateDisplay::Streaming);

    wait_for(|| {
        app.pump_gateway_events();
        app.chat
            .messages()
            .iter()
            .any(|message| message.content == "hello world")
            && app.status.agent_state == AgentStateDisplay::Idle
    });
    assert_eq!(app.status.agent_state, AgentStateDisplay::Idle);
}

#[test]
fn integration_spec_3582_gateway_runtime_renders_phase_specific_streaming_visuals() {
    let server = spawn_sse_server(vec![
        "event: response.created\n\
data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_4\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"in_progress\",\"phase\":\"model\",\"response_id\":\"resp_4\"}}\n\n"
            .to_string(),
        "event: response.output_text.delta\n\
data: {\"type\":\"response.output_text.delta\",\"response_id\":\"resp_4\",\"delta\":\"hello \",\"operator_state\":{\"entity\":\"artifact\",\"status\":\"streaming\",\"phase\":\"stream\",\"artifact_kind\":\"assistant_output_text\",\"response_id\":\"resp_4\"}}\n\n"
            .to_string(),
    ]);
    let mut app = App::new(AppConfig {
        model: "openai/gpt-5.2".to_string(),
        profile: "ops-interactive".to_string(),
        session_key: "default".to_string(),
        workspace_label: "rust_pi-3582-phase".to_string(),
        approval_mode: "ask".to_string(),
        tick_rate_ms: 25,
        gateway: Some(GatewayInteractiveConfig {
            base_url: server.base_url,
            auth_token: None,
            session_key: "default".to_string(),
            request_timeout_ms: 3_000,
        }),
    });

    for ch in "testing".chars() {
        app.input.insert_char(ch);
    }
    app.submit_input();

    wait_for(|| {
        app.pump_gateway_events();
        app.status.agent_state == AgentStateDisplay::Streaming
            && app
                .chat
                .messages()
                .iter()
                .any(|message| message.content.contains("hello "))
    });

    let rendered = render_app(&mut app, 120, 28);
    assert!(rendered.contains("Streaming reply"));
    assert!(rendered.contains("artifact:stream"));
    assert!(rendered.contains("assistant_output_text"));
}

#[test]
fn integration_spec_3582_gateway_runtime_ignores_keep_alive_comment_frames() {
    let server = spawn_sse_server(vec![
        r#"event: response.created
data: {"type":"response.created","response":{"id":"resp_keepalive"},"operator_state":{"entity":"turn","status":"in_progress","phase":"model","response_id":"resp_keepalive"}}

"#
            .to_string(),
        ": keep-alive

".to_string(),
        r#"event: response.output_text.done
data: {"type":"response.output_text.done","response_id":"resp_keepalive","text":"hello world","operator_state":{"entity":"artifact","status":"completed","artifact_kind":"assistant_output_text","response_id":"resp_keepalive"}}

"#
            .to_string(),
        r#"event: response.completed
data: {"type":"response.completed","response":{"id":"resp_keepalive","output_text":"hello world"},"operator_state":{"entity":"turn","status":"completed","phase":"done"}}

"#
            .to_string(),
    ]);
    let mut app = App::new(AppConfig {
        model: "openai/gpt-5.2".to_string(),
        profile: "ops-interactive".to_string(),
        session_key: "default".to_string(),
        workspace_label: "rust_pi-3582-phase".to_string(),
        approval_mode: "ask".to_string(),
        tick_rate_ms: 25,
        gateway: Some(GatewayInteractiveConfig {
            base_url: server.base_url,
            auth_token: None,
            session_key: "default".to_string(),
            request_timeout_ms: 3_000,
        }),
    });

    for ch in "testing".chars() {
        app.input.insert_char(ch);
    }
    app.submit_input();

    wait_for(|| {
        app.pump_gateway_events();
        app.chat
            .messages()
            .iter()
            .any(|message| message.content == "hello world")
            && app.status.agent_state == AgentStateDisplay::Idle
    });

    assert!(app
        .chat
        .messages()
        .iter()
        .any(|message| message.content == "hello world"));
}

#[test]
fn red_spec_3585_gateway_runtime_surfaces_supported_model_hint_for_oauth_incompatibility() {
    let server = spawn_sse_server(vec![
        "event: response.created\n\
data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_3585\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"in_progress\",\"phase\":\"model\",\"response_id\":\"resp_3585\"}}\n\n"
            .to_string(),
        "event: response.failed\n\
data: {\"type\":\"response.failed\",\"error\":{\"code\":\"model_unsupported\",\"message\":\"The 'openai/gpt-5.2' model is not supported when using Codex with a ChatGPT account.\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"failed\",\"phase\":\"failed\",\"reason_code\":\"model_unsupported\"}}\n\n"
            .to_string(),
    ]);
    let mut app = App::new(AppConfig {
        model: "openai/gpt-5.2".to_string(),
        profile: "ops-interactive".to_string(),
        session_key: "default".to_string(),
        workspace_label: "rust_pi-3582-phase".to_string(),
        approval_mode: "ask".to_string(),
        tick_rate_ms: 25,
        gateway: Some(GatewayInteractiveConfig {
            base_url: server.base_url,
            auth_token: None,
            session_key: "default".to_string(),
            request_timeout_ms: 3_000,
        }),
    });

    for ch in "test".chars() {
        app.input.insert_char(ch);
    }
    app.submit_input();

    wait_for(|| {
        app.pump_gateway_events();
        app.status.agent_state == AgentStateDisplay::Error
    });

    let rendered = render_app(&mut app, 120, 28);
    assert!(rendered.contains("gpt-5.2-codex"));
}

struct TestServer {
    base_url: String,
}

fn spawn_sse_server(frames: Vec<String>) -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("server addr");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        read_request(&mut stream);
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            )
            .expect("write headers");
        for frame in frames {
            stream.write_all(frame.as_bytes()).expect("write frame");
            stream.flush().expect("flush frame");
            thread::sleep(Duration::from_millis(25));
        }
    });
    TestServer {
        base_url: format!("http://{address}"),
    }
}

fn read_request(stream: &mut std::net::TcpStream) {
    let mut buffer = [0_u8; 4096];
    let _ = stream.read(&mut buffer);
}

fn wait_for(mut condition: impl FnMut() -> bool) {
    for _ in 0..40 {
        if condition() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("condition not satisfied before timeout");
}

fn render_app(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|frame| render(frame, app)).expect("draw");
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
