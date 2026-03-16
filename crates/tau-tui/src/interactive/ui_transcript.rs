use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use super::activity::{attention_height, render_activity_strip, render_attention_strip};
use super::super::app::{App, FocusPanel};
use super::super::chat::MessageRole;

pub(super) fn render_transcript_shell(frame: &mut Frame, app: &App, area: Rect) {
    let attention_height = attention_height(app);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(attention_height),
            Constraint::Min(1),
        ])
        .split(area);
    render_activity_strip(frame, app, chunks[0]);
    if attention_height > 0 {
        render_attention_strip(frame, app, chunks[1]);
    }
    render_chat_panel(frame, app, chunks[2]);
}

fn render_chat_panel(frame: &mut Frame, app: &App, area: Rect) {
    let inner = render_transcript_background(frame, app, area);
    if render_empty_transcript(frame, app, inner) {
        return;
    }
    let lines = transcript_lines(app);
    let scroll = transcript_scroll(app, lines.len(), inner.height as usize);
    render_transcript_content(frame, inner, lines, scroll);
    render_transcript_scrollbar(frame, area, inner, app, scroll);
}

fn transcript_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for msg in app.chat.messages() {
        lines.extend(message_card_lines(msg.role, &msg.timestamp, &msg.content));
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

fn render_transcript_background(frame: &mut Frame, app: &App, area: Rect) -> Rect {
    let background = if app.focus == FocusPanel::Chat {
        Color::Rgb(20, 24, 31)
    } else {
        Color::Rgb(16, 18, 24)
    };
    let block = Block::default().style(Style::default().bg(background));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

fn render_empty_transcript(frame: &mut Frame, app: &App, area: Rect) -> bool {
    if !app.chat.is_empty() {
        return false;
    }
    let empty = Paragraph::new(
        "Start with a prompt below. The transcript will stay primary while details stay on demand.",
    )
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(empty, area);
    true
}

fn render_transcript_content(
    frame: &mut Frame,
    area: Rect,
    lines: Vec<Line<'static>>,
    scroll: u16,
) {
    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_transcript_scrollbar(
    frame: &mut Frame,
    area: Rect,
    inner: Rect,
    app: &App,
    scroll: u16,
) {
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

fn message_card_lines(role: MessageRole, timestamp: &str, content: &str) -> Vec<Line<'static>> {
    let role_style = match role {
        MessageRole::User => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        MessageRole::Assistant => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        MessageRole::System => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        MessageRole::Tool => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
    };
    let mut lines = vec![Line::from(vec![
        Span::styled("╭─ ", Style::default().fg(Color::DarkGray)),
        Span::styled(role.label().to_string(), role_style),
        Span::styled(
            format!(" · {timestamp}"),
            Style::default().fg(Color::DarkGray),
        ),
    ])];
    for content_line in content.lines() {
        lines.push(Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::raw(content_line.to_string()),
        ]));
    }
    lines.push(Line::from(Span::styled(
        "╰─",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines
}
