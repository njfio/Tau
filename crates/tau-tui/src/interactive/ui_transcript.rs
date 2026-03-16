use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use super::super::app::{App, FocusPanel};
use super::super::chat::MessageRole;
use super::super::status::AgentStateDisplay;
use super::super::tools::{ToolEntry, ToolStatus};

pub(super) fn render_transcript_shell(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(area);
    render_activity_strip(frame, app, chunks[0]);
    render_chat_panel(frame, app, chunks[1]);
}

fn render_activity_strip(frame: &mut Frame, app: &App, area: Rect) {
    let title = Span::styled(
        " Live activity ",
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightYellow)
            .add_modifier(Modifier::BOLD),
    );
    let summary = Span::styled(activity_summary(app), Style::default().fg(Color::White));
    let commands = Span::styled(
        " /details  /retry  /interrupt ",
        Style::default().fg(Color::DarkGray),
    );
    let line = Line::from(vec![title, Span::raw(" "), summary, commands]);
    frame.render_widget(Paragraph::new(line).wrap(Wrap { trim: true }), area);
}

fn activity_summary(app: &App) -> String {
    match app.status.agent_state {
        AgentStateDisplay::Idle => "Ready for the next prompt.".to_string(),
        AgentStateDisplay::Thinking => "Thinking through the next step.".to_string(),
        AgentStateDisplay::Streaming => {
            "Streaming assistant output into the transcript.".to_string()
        }
        AgentStateDisplay::Error => "Last turn failed. Open details or retry.".to_string(),
        AgentStateDisplay::ToolExec => latest_running_tool(app)
            .map(|tool| format!("Running tool: {}.", tool.name))
            .unwrap_or_else(|| "Running a tool call.".to_string()),
    }
}

fn latest_running_tool(app: &App) -> Option<&ToolEntry> {
    app.tools
        .entries()
        .iter()
        .rev()
        .find(|entry| entry.status == ToolStatus::Running)
}

fn render_chat_panel(frame: &mut Frame, app: &App, area: Rect) {
    let background = if app.focus == FocusPanel::Chat {
        Color::Rgb(20, 24, 31)
    } else {
        Color::Rgb(16, 18, 24)
    };
    let block = Block::default().style(Style::default().bg(background));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.chat.is_empty() {
        let empty = Paragraph::new(
            "Start with a prompt below. The transcript will stay primary while details stay on demand.",
        )
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    let lines = transcript_lines(app);
    let scroll = transcript_scroll(app, lines.len(), inner.height as usize);
    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, inner);

    if inner.height as usize >= app.chat.len() {
        return;
    }
    let mut scrollbar_state = ScrollbarState::new(inner.height as usize + app.chat.len())
        .position(scroll as usize);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("^"))
        .end_symbol(Some("v"));
    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 0,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn transcript_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for msg in app.chat.messages() {
        let role_style = match msg.role {
            MessageRole::User => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            MessageRole::Assistant => {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            }
            MessageRole::System => {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            }
            MessageRole::Tool => {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            }
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", msg.role.label()), role_style),
            Span::styled(msg.timestamp.clone(), Style::default().fg(Color::DarkGray)),
        ]));
        for content_line in msg.content.lines() {
            lines.push(Line::from(Span::raw(format!("  {content_line}"))));
        }
        lines.push(Line::from(""));
    }
    lines
}

fn transcript_scroll(app: &App, total_lines: usize, visible_height: usize) -> u16 {
    if total_lines <= visible_height {
        return 0;
    }
    let msg_idx = app.chat.scroll_offset();
    if msg_idx >= app.chat.len().saturating_sub(1) {
        return (total_lines - visible_height) as u16;
    }
    let approx = msg_idx * 3;
    approx.min(total_lines.saturating_sub(visible_height)) as u16
}
