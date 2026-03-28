use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use super::app::{App, FocusPanel};
use super::chat::MessageRole;
use super::ui_chat_tool_lines::{build_tool_summary_lines, build_transcript_tool_lines};

pub(crate) fn render_chat_panel(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == FocusPanel::Chat {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(Span::styled(
            " Chat ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tool_summary_lines = build_tool_summary_lines(app);
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
        let (role_color, role_label) = match msg.role {
            MessageRole::User => (Color::Green, msg.role.label()),
            MessageRole::Assistant => (Color::Cyan, msg.role.label()),
            MessageRole::System => (Color::Yellow, msg.role.label()),
            MessageRole::Tool => (Color::Magenta, msg.role.label()),
        };

        // Header with role icon and timestamp
        let icon = match msg.role {
            MessageRole::User => ">",
            MessageRole::Assistant => "*",
            MessageRole::System => "!",
            MessageRole::Tool => "#",
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {icon} "),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", role_label),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", msg.timestamp),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // Render content with syntax awareness
        let content_lines = render_content_lines(&msg.content, msg.role);
        lines.extend(content_lines);

        // Visual separator
        lines.push(Line::from(""));
    }
    lines
}

/// Render message content with diff highlighting, code block detection,
/// and file path formatting.
fn render_content_lines(content: &str, role: MessageRole) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut in_diff = false;

    let base_style = match role {
        MessageRole::User => Style::default().fg(Color::White),
        MessageRole::Assistant => Style::default(),
        MessageRole::System => Style::default().fg(Color::DarkGray),
        MessageRole::Tool => Style::default().fg(Color::DarkGray),
    };

    for line in content.lines() {
        let trimmed = line.trim();

        // Code block toggle
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            lines.push(Line::from(Span::styled(
                format!("    {line}"),
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }

        // Diff detection
        if trimmed.starts_with("diff --git")
            || trimmed.starts_with("file update")
            || (trimmed.starts_with("A ") && trimmed.contains('/'))
        {
            in_diff = true;
        }
        if in_diff && trimmed.is_empty() {
            in_diff = false;
        }

        if in_code_block {
            // Code block content — monospace style
            lines.push(Line::from(Span::styled(
                format!("    {line}"),
                Style::default().fg(Color::Rgb(180, 180, 210)),
            )));
        } else if in_diff || is_diff_line(trimmed) {
            // Diff lines with color coding
            lines.push(render_diff_line(line));
        } else if is_file_path_line(trimmed) {
            // File paths — highlighted
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    line.trim().to_string(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]));
        } else if trimmed.starts_with("exec") || trimmed.starts_with("codex") {
            // Tool execution markers
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::DIM),
            )));
        } else if trimmed.starts_with("apply_patch") || trimmed.starts_with("Success.") {
            // Patch application results
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                Style::default().fg(Color::Green),
            )));
        } else if trimmed.starts_with("tokens used") || trimmed.parse::<u64>().is_ok() {
            // Token count — dim
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            // Regular content
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                base_style,
            )));
        }
    }
    lines
}

fn is_diff_line(line: &str) -> bool {
    line.starts_with('+') && !line.starts_with("+++")
        || line.starts_with('-') && !line.starts_with("---")
        || line.starts_with("@@")
        || line.starts_with("diff --git")
        || line.starts_with("new file mode")
        || line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
}

fn render_diff_line(line: &str) -> Line<'static> {
    let trimmed = line.trim();
    if trimmed.starts_with('+') && !trimmed.starts_with("+++") {
        Line::from(Span::styled(
            format!("  {line}"),
            Style::default().fg(Color::Green),
        ))
    } else if trimmed.starts_with('-') && !trimmed.starts_with("---") {
        Line::from(Span::styled(
            format!("  {line}"),
            Style::default().fg(Color::Red),
        ))
    } else if trimmed.starts_with("@@") {
        Line::from(Span::styled(
            format!("  {line}"),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
        ))
    } else if trimmed.starts_with("diff --git") {
        Line::from(Span::styled(
            format!("  {line}"),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::styled(
            format!("  {line}"),
            Style::default().fg(Color::DarkGray),
        ))
    }
}

fn is_file_path_line(line: &str) -> bool {
    // Detect lines that are primarily file paths
    let l = line.trim();
    (l.starts_with('/') || l.starts_with("./"))
        && !l.contains(' ')
        && (l.contains('.') || l.ends_with('/'))
}

/// Compute scroll position using actual line counts per message.
fn compute_chat_scroll(app: &App, total_lines: usize, visible_height: usize) -> u16 {
    if total_lines <= visible_height {
        return 0;
    }

    let msg_idx = app.chat.scroll_offset();
    let msg_count = app.chat.len();

    // Pinned to bottom
    if msg_idx >= msg_count.saturating_sub(1) {
        return (total_lines.saturating_sub(visible_height)) as u16;
    }

    // Count rendered lines up to the target message
    let mut line_offset = 0usize;
    for (i, msg) in app.chat.messages().iter().enumerate() {
        if i >= msg_idx {
            break;
        }
        // header (1) + content lines + separator (1)
        line_offset += 1 + msg.content.lines().count().max(1) + 1;
    }

    line_offset.min(total_lines.saturating_sub(visible_height)) as u16
}
