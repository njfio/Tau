use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::super::app::App;
use super::super::status::AgentStateDisplay;
use super::super::tools::{ToolEntry, ToolStatus};

pub(super) fn run_state_height(app: &App) -> u16 {
    if run_state_card(app).is_some() { 5 } else { 0 }
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
        Line::from(Span::styled(card.primary, Style::default().fg(Color::White))),
        Line::from(Span::styled(card.secondary, Style::default().fg(Color::Gray))),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

struct RunStateCard<'a> {
    title: &'a str,
    state: String,
    primary: String,
    secondary: String,
    color: Color,
}

fn run_state_card(app: &App) -> Option<RunStateCard<'static>> {
    if let Some(tool) = latest_running_tool(app) {
        return Some(RunStateCard {
            title: " Running tool ",
            state: "tool".to_string(),
            primary: tool.name.clone(),
            secondary: tool.detail.clone(),
            color: Color::Cyan,
        });
    }
    if app.status.agent_state != AgentStateDisplay::Idle {
        return app.last_submitted_input.as_ref().map(|prompt| RunStateCard {
            title: " Turn active ",
            state: app.status.agent_state.label().to_lowercase(),
            primary: prompt.clone(),
            secondary: active_turn_summary(app.status.agent_state),
            color: Color::Yellow,
        });
    }
    app.last_submitted_input.as_ref().map(|prompt| RunStateCard {
        title: " Last turn ",
        state: "completed".to_string(),
        primary: prompt.clone(),
        secondary: completed_turn_summary(app),
        color: Color::Green,
    })
}

fn latest_running_tool(app: &App) -> Option<&ToolEntry> {
    app.tools
        .entries()
        .iter()
        .rev()
        .find(|entry| entry.status == ToolStatus::Running)
}

fn active_turn_summary(state: AgentStateDisplay) -> String {
    match state {
        AgentStateDisplay::Thinking => "planning the next action".to_string(),
        AgentStateDisplay::ToolExec => "executing tool work".to_string(),
        AgentStateDisplay::Streaming => "streaming assistant response".to_string(),
        AgentStateDisplay::Error => "turn needs intervention".to_string(),
        AgentStateDisplay::Idle => "ready".to_string(),
    }
}

fn completed_turn_summary(app: &App) -> String {
    let last_assistant = app
        .chat
        .messages()
        .iter()
        .rev()
        .find(|msg| matches!(msg.role, super::super::chat::MessageRole::Assistant))
        .map(|msg| msg.content.as_str())
        .unwrap_or("assistant reply ready");
    if last_assistant.is_empty() {
        return "assistant reply ready".to_string();
    }
    "assistant reply ready".to_string()
}

fn badge(text: &str, background: Color) -> Span<'static> {
    Span::styled(
        text.to_string(),
        Style::default()
            .fg(Color::Black)
            .bg(background)
            .add_modifier(Modifier::BOLD),
    )
}
