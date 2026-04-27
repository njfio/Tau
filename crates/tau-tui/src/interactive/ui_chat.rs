use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use super::app::{App, FocusPanel};
use super::chat::MessageRole;
use super::ui_chat_tool_lines::{
    build_tool_summary_lines, build_transcript_first_turn_lines, build_transcript_tool_lines,
};

pub(crate) fn render_chat_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.focus == FocusPanel::Chat {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(Span::styled(
            " Transcript ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut tool_summary_lines = build_transcript_first_turn_lines(app);
    tool_summary_lines.extend(build_tool_summary_lines(app));
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(tool_summary_lines.len() as u16),
            Constraint::Min(0),
        ])
        .split(inner);

    if !tool_summary_lines.is_empty() {
        let summary = Paragraph::new(Text::from(tool_summary_lines)).wrap(Wrap { trim: false });
        frame.render_widget(summary, content_chunks[0]);
    }

    let lines = render_chat_lines(app);
    if lines.is_empty() {
        let empty = Paragraph::new("No messages yet. Type below and press Enter.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, content_chunks[1]);
        return;
    }

    let total_lines = lines.len();
    let visible_height = content_chunks[1].height as usize;
    let max_scroll = total_lines.saturating_sub(visible_height);
    app.chat.set_max_scroll(max_scroll);

    let scroll = compute_chat_scroll(app, total_lines, visible_height);
    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, content_chunks[1]);

    if total_lines > visible_height {
        let mut scrollbar_state = ScrollbarState::new(total_lines).position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        frame.render_stateful_widget(scrollbar, content_chunks[1], &mut scrollbar_state);
    }
}

fn render_chat_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = render_message_lines(app);
    lines.extend(build_transcript_tool_lines(app));
    lines
}

fn render_message_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for msg in app.chat.messages() {
        let (role_style, role_label) = match msg.role {
            MessageRole::User => (
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
            MessageRole::Assistant => (
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
            MessageRole::System => (
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
            MessageRole::Tool => (
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("[{}] ", msg.timestamp),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("{}: ", role_label), role_style),
        ]));

        for content_line in msg.content.lines() {
            lines.push(Line::from(Span::raw(format!("  {content_line}"))));
        }
        lines.push(Line::from(""));
    }
    lines
}

fn compute_chat_scroll(app: &App, total_lines: usize, visible_height: usize) -> u16 {
    if total_lines <= visible_height {
        return 0;
    }
    let max = total_lines.saturating_sub(visible_height);
    if app.chat.follow_mode() {
        return max as u16;
    }
    app.chat.scroll_offset().min(max) as u16
}
