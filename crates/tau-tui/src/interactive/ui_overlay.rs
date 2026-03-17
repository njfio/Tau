use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::super::app::App;
use super::super::chat::MessageRole;
use super::drawer;

pub(super) fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(area, 60, 22, 4);
    frame.render_widget(Clear, popup_area);
    let paragraph = Paragraph::new(help_text())
        .block(overlay_block(" Help ", Color::Cyan))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, popup_area);
}

pub(super) fn render_detail_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(area, 52, 16, 6);
    frame.render_widget(Clear, popup_area);
    let block = overlay_block(
        format!(" Quick details [{}] ", app.detail_section.label()),
        Color::Cyan,
    );
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);
    drawer::render_detail_contents(frame, app, inner);
}

pub(super) fn render_thinking_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(area, 72, 18, 8);
    frame.render_widget(Clear, popup_area);
    let block = overlay_block(" Thinking ", Color::Magenta);
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);
    frame.render_widget(thinking_paragraph(app), inner);
}

fn thinking_paragraph(app: &App) -> Paragraph<'static> {
    Paragraph::new(thinking_text(app))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White))
}

fn thinking_text(app: &App) -> Text<'static> {
    let Some(state) = &app.last_operator_state else {
        return Text::from(vec![Line::from("No active turn context")]);
    };
    let mut lines = vec![
        Line::from(format!("State: {}:{}", state.entity, overlay_phase(state))),
        Line::from(format!("Status: {}", state.status)),
    ];
    push_field(&mut lines, "Artifact", state.artifact_kind.as_deref());
    push_field(&mut lines, "Response", state.response_id.as_deref());
    push_field(&mut lines, "Reason", state.reason_code.as_deref());
    if let Some(tool) = app.tools.entries().last() {
        lines.push(Line::from(format!("Current tool: {}", tool.name)));
    }
    if let Some(prompt) = &app.last_submitted_input {
        lines.push(Line::from(format!("Prompt: {}", prompt.trim())));
    }
    if let Some(preview) = app.chat.latest_content_by_role(MessageRole::Assistant) {
        lines.push(Line::from(""));
        lines.push(Line::from("Assistant preview:"));
        lines.push(Line::from(preview.trim().to_string()));
    }
    Text::from(lines)
}

fn overlay_phase(state: &super::super::gateway::OperatorStateEvent) -> &str {
    state.phase.as_deref().unwrap_or(state.status.as_str())
}

fn push_field(lines: &mut Vec<Line<'static>>, label: &str, value: Option<&str>) {
    let Some(value) = value else {
        return;
    };
    lines.push(Line::from(format!("{label}: {value}")));
}

fn centered_rect(area: Rect, max_width: u16, max_height: u16, margin: u16) -> Rect {
    let popup_width = area.width.saturating_sub(margin).min(max_width);
    let popup_height = area.height.saturating_sub(margin).min(max_height);
    Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    )
}

fn overlay_block(title: impl Into<ratatui::text::Line<'static>>, color: Color) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(Color::Rgb(8, 10, 14)))
}

fn help_text() -> Text<'static> {
    Text::from(vec![
        overlay_heading("Tau Interactive TUI — Keyboard Reference", Color::Cyan),
        Line::from(""),
        overlay_heading("Normal Mode", Color::White),
        Line::from("  i/a       Enter insert mode"),
        Line::from("  o         Insert mode + new line"),
        Line::from("  q         Quit"),
        Line::from("  ?         Toggle this help"),
        Line::from("  j/k       Scroll chat up/down"),
        Line::from("  g/G       Scroll to top/bottom"),
        Line::from("  Ctrl+d/u  Page down/up"),
        Line::from("  Tab       Cycle focus between panels"),
        Line::from("  1/2/3     Focus Chat/Input/Tools"),
        Line::from(""),
        overlay_heading("Insert Mode", Color::White),
        Line::from("  Enter     Send message"),
        Line::from("  Shift+Enter / Alt+Enter  New line"),
        Line::from("  Esc       Back to normal mode"),
        Line::from(""),
        overlay_heading("Global", Color::White),
        Line::from("  Ctrl+c    Quit"),
        Line::from("  Ctrl+l    Clear chat"),
        Line::from("  Ctrl+t    Toggle details drawer"),
        Line::from("  Ctrl+p    Command palette"),
        Line::from(""),
        overlay_heading("Slash Commands", Color::White),
        Line::from("  /thinking  Show live turn context"),
        Line::from("  /details  Toggle detail drawer"),
        Line::from("  /retry    Replay the last prompt"),
        Line::from("  /interrupt  Stop the active turn"),
    ])
}

fn overlay_heading(text: &'static str, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}
