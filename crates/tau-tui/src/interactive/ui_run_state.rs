use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::super::app::App;
use super::super::status::AgentStateDisplay;
use super::shared::{badge, latest_running_tool};

pub(super) fn run_state_height(app: &App) -> u16 {
    run_state_card(app)
        .map(|card| if card.preview.is_some() { 6 } else { 5 })
        .unwrap_or(0)
}

pub(super) fn render_run_state_card(frame: &mut Frame, app: &App, area: Rect) {
    let Some(card) = run_state_card(app) else {
        return;
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Rgb(12, 15, 20)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines = vec![
        Line::from(vec![
            badge(card.title, card.color),
            Span::raw(" "),
            Span::styled(card.state, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(Span::styled(
            card.primary,
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            card.secondary,
            Style::default().fg(Color::Gray),
        )),
    ];
    let lines = extend_with_preview(lines, card.preview);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

struct RunStateCard<'a> {
    title: &'a str,
    state: String,
    primary: String,
    secondary: String,
    preview: Option<String>,
    color: Color,
}

fn run_state_card(app: &App) -> Option<RunStateCard<'static>> {
    if let Some(tool) = latest_running_tool(app) {
        return Some(RunStateCard {
            title: " Running tool ",
            state: "tool".to_string(),
            primary: tool.name.clone(),
            secondary: tool.detail.clone(),
            preview: None,
            color: Color::Cyan,
        });
    }
    if app.status.agent_state != AgentStateDisplay::Idle {
        return app
            .last_submitted_input
            .as_ref()
            .map(|prompt| RunStateCard {
                title: " Turn active ",
                state: state_label(app.status.agent_state).to_string(),
                primary: prompt.clone(),
                secondary: active_turn_summary(app.status.agent_state),
                preview: streaming_preview(app),
                color: Color::Yellow,
            });
    }
    app.last_submitted_input
        .as_ref()
        .map(|prompt| RunStateCard {
            title: " Last turn ",
            state: "completed".to_string(),
            primary: prompt.clone(),
            secondary: completed_turn_summary(),
            preview: None,
            color: Color::Green,
        })
}

fn active_turn_summary(state: AgentStateDisplay) -> String {
    match state {
        AgentStateDisplay::Thinking => "planning the next action".to_string(),
        AgentStateDisplay::ToolExec => "executing tool work".to_string(),
        AgentStateDisplay::Streaming => "assistant output arriving".to_string(),
        AgentStateDisplay::Error => "turn needs intervention".to_string(),
        AgentStateDisplay::Idle => "ready".to_string(),
    }
}

fn completed_turn_summary() -> String {
    "assistant reply ready".to_string()
}

fn state_label(state: AgentStateDisplay) -> &'static str {
    match state {
        AgentStateDisplay::Idle => "idle",
        AgentStateDisplay::Thinking => "thinking",
        AgentStateDisplay::ToolExec => "tool",
        AgentStateDisplay::Streaming => "streaming",
        AgentStateDisplay::Error => "error",
    }
}

fn streaming_preview(app: &App) -> Option<String> {
    if app.status.agent_state != AgentStateDisplay::Streaming {
        return None;
    }
    app.chat
        .messages()
        .iter()
        .rev()
        .find(|msg| matches!(msg.role, super::super::chat::MessageRole::Assistant))
        .map(|msg| truncate(&msg.content, 72))
}

fn extend_with_preview(
    mut lines: Vec<Line<'static>>,
    preview: Option<String>,
) -> Vec<Line<'static>> {
    if let Some(text) = preview {
        lines.push(Line::from(Span::styled(
            text,
            Style::default().fg(Color::LightCyan),
        )));
    }
    lines
}

fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        return input.to_string();
    }
    if max <= 3 {
        return input[..max].to_string();
    }
    format!("{}...", &input[..max - 3])
}
