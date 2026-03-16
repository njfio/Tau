//! UI rendering with ratatui for the transcript-first interactive shell.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use super::app::{App, FocusPanel, InputMode};
use super::chat::MessageRole;
use super::status::{AgentStateDisplay, CircuitBreakerDisplay};
use super::tools::ToolStatus;

/// Render the full application UI.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(input_height(app)),
        ])
        .split(size);

    render_status_bar(frame, app, main_chunks[0]);
    render_body(frame, app, main_chunks[1]);
    render_input(frame, app, main_chunks[2]);

    if app.show_help {
        render_help_overlay(frame, size);
    }

    if app.focus == FocusPanel::CommandPalette {
        render_command_palette(frame, app, size);
    }
}

fn input_height(app: &App) -> u16 {
    let lines = app.input.lines().len() as u16;
    lines.clamp(2, 6) + 3
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let model_span = Span::styled(
        format!(" {} ", app.status.model),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    );

    let session_span = Span::styled(
        format!(" session={} ", app.config.session_key),
        Style::default().fg(Color::Black).bg(Color::Blue),
    );

    let workspace_span = Span::styled(
        format!(" cwd={} ", app.config.workspace_label),
        Style::default().fg(Color::Black).bg(Color::LightBlue),
    );

    let profile_span = Span::styled(
        format!(" profile={} ", app.status.profile),
        Style::default().fg(Color::Black).bg(Color::DarkGray),
    );

    let approval_span = Span::styled(
        format!(" approval={} ", app.config.approval_mode),
        Style::default().fg(Color::Black).bg(Color::Yellow),
    );

    let tokens_span = Span::styled(
        format!(" tok={} ", app.status.format_tokens()),
        Style::default().fg(Color::White).bg(Color::DarkGray),
    );

    let cb_color = match app.status.circuit_breaker_state {
        CircuitBreakerDisplay::Closed => Color::Green,
        CircuitBreakerDisplay::Open => Color::Red,
        CircuitBreakerDisplay::HalfOpen => Color::Yellow,
    };
    let cb_span = Span::styled(
        format!(" health={} ", app.status.circuit_breaker_state.label().to_lowercase()),
        Style::default().fg(Color::Black).bg(cb_color),
    );

    let state_color = match app.status.agent_state {
        AgentStateDisplay::Idle => Color::DarkGray,
        AgentStateDisplay::Thinking => Color::Yellow,
        AgentStateDisplay::ToolExec => Color::Cyan,
        AgentStateDisplay::Streaming => Color::Green,
        AgentStateDisplay::Error => Color::Red,
    };
    let state_span = Span::styled(
        format!(" active={} ", app.status.agent_state.label().to_lowercase()),
        Style::default().fg(Color::Black).bg(state_color),
    );

    let sep = Span::raw(" ");

    let line = Line::from(vec![
        model_span, sep.clone(), session_span, sep.clone(),
        workspace_span, sep.clone(), profile_span, sep.clone(),
        approval_span, sep.clone(),
        tokens_span, sep.clone(), cb_span, sep, state_span,
    ]);

    let bar = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(bar, area);
}

fn render_body(frame: &mut Frame, app: &App, area: Rect) {
    if app.show_tool_panel {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(74),
                Constraint::Percentage(26),
            ])
            .split(area);

        render_transcript_shell(frame, app, chunks[0]);
        render_detail_drawer(frame, app, chunks[1]);
    } else {
        render_transcript_shell(frame, app, area);
    }
}

fn render_transcript_shell(frame: &mut Frame, app: &App, area: Rect) {
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
    let paragraph = Paragraph::new(Line::from(vec![title, Span::raw(" "), summary, commands]))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn activity_summary(app: &App) -> String {
    match app.status.agent_state {
        AgentStateDisplay::Idle => "Ready for the next prompt.".to_string(),
        AgentStateDisplay::Thinking => "Thinking through the next step.".to_string(),
        AgentStateDisplay::Streaming => "Streaming assistant output into the transcript.".to_string(),
        AgentStateDisplay::Error => "Last turn failed. Open details or retry.".to_string(),
        AgentStateDisplay::ToolExec => latest_running_tool(app)
            .map(|tool| format!("Running tool: {}.", tool.name))
            .unwrap_or_else(|| "Running a tool call.".to_string()),
    }
}

fn latest_running_tool(app: &App) -> Option<&super::tools::ToolEntry> {
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
        let empty = Paragraph::new("Start with a prompt below. The transcript will stay primary while details stay on demand.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for msg in app.chat.messages() {
        let (role_style, role_label) = match msg.role {
            MessageRole::User => (
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
            MessageRole::Assistant => (
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
            MessageRole::System => (
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
            MessageRole::Tool => (
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                msg.role.label(),
            ),
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{} ", role_label), role_style),
            Span::styled(msg.timestamp.as_str(), Style::default().fg(Color::DarkGray)),
        ]));

        for content_line in msg.content.lines() {
            lines.push(Line::from(Span::raw(format!("  {content_line}"))));
        }
        lines.push(Line::from(""));
    }

    let total_lines = lines.len();
    let visible_height = inner.height as usize;

    let scroll = if total_lines > visible_height {
        let msg_idx = app.chat.scroll_offset();
        if msg_idx >= app.chat.len().saturating_sub(1) {
            (total_lines - visible_height) as u16
        } else {
            let approx = msg_idx * 3;
            approx.min(total_lines.saturating_sub(visible_height)) as u16
        }
    } else {
        0
    };

    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, inner);

    if total_lines > visible_height {
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}

fn render_detail_drawer(frame: &mut Frame, app: &App, area: Rect) {
    let title = " Details ";
    let border_style = if app.focus == FocusPanel::Tools {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::LEFT)
        .border_style(border_style)
        .style(Style::default().bg(Color::Rgb(12, 14, 18)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items = detail_items(app, inner.width as usize);
    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn detail_items(app: &App, max_width: usize) -> Vec<ListItem<'static>> {
    let mut items = vec![
        section_item("Tool activity"),
        detail_line_item(format!(
            "{} active / {} total",
            app.tools.active_count(),
            app.tools.total_count()
        )),
    ];

    items.extend(recent_tool_items(app, max_width));
    items.extend([
        section_item("Memory"),
        detail_line_item("No stored memory yet.".to_string()),
        section_item("Cortex"),
        detail_line_item("Observer idle until a turn completes.".to_string()),
        section_item("Sessions"),
        detail_line_item(format!("Current session: {}", app.config.session_key)),
    ]);
    items
}

fn recent_tool_items(app: &App, max_width: usize) -> Vec<ListItem<'static>> {
    let entries = app.tools.entries();
    if entries.is_empty() {
        return vec![detail_line_item("No tool executions yet.".to_string())];
    }

    let start = entries.len().saturating_sub(3);
    entries[start..]
        .iter()
        .map(|entry| {
            let status_style = match entry.status {
                ToolStatus::Running => Style::default().fg(Color::Yellow),
                ToolStatus::Success => Style::default().fg(Color::Green),
                ToolStatus::Failed => Style::default().fg(Color::Red),
                ToolStatus::Timeout => Style::default().fg(Color::Magenta),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", entry.status.label()),
                    status_style,
                ),
                Span::styled(
                    entry.name.clone(),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" "),
                Span::styled(
                    truncate(&entry.detail, max_width.saturating_sub(entry.name.len() + 8)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect()
}

fn section_item(title: &str) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )))
}

fn detail_line_item(text: String) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(Color::Gray),
    )))
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == FocusPanel::Input {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::Rgb(10, 12, 16)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(inner.height.saturating_sub(1)), Constraint::Length(1)])
        .split(inner);

    let input_text: Vec<Line> = app
        .input
        .lines()
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let prefix = if idx == 0 { "› " } else { "  " };
            Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::LightGreen)),
                Span::raw(line.as_str()),
            ])
        })
        .collect();
    let input = Paragraph::new(Text::from(input_text));
    frame.render_widget(input, chunks[0]);

    let mode = match app.input_mode {
        InputMode::Insert => "insert",
        InputMode::Normal => "normal",
    };
    let actions = format!(
        "Press / for commands  Enter send  Shift+Enter newline  Tab focus  {mode} mode"
    );
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Compose ", Style::default().fg(Color::Black).bg(Color::Green)),
        Span::raw(" "),
        Span::styled(actions, Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(footer, chunks[1]);

    if app.focus == FocusPanel::Input && app.input_mode == InputMode::Insert {
        let cursor_x = chunks[0].x + 2 + app.input.cursor_col() as u16;
        let cursor_y = chunks[0].y + app.input.cursor_line() as u16;
        if cursor_x < chunks[0].x + chunks[0].width && cursor_y < chunks[0].y + chunks[0].height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let help_width = 60u16.min(area.width.saturating_sub(4));
    let help_height = 22u16.min(area.height.saturating_sub(4));
    let x = (area.width - help_width) / 2;
    let y = (area.height - help_height) / 2;
    let popup_area = Rect::new(x, y, help_width, help_height);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled("Tau Interactive TUI — Keyboard Reference", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Normal Mode", Style::default().add_modifier(Modifier::BOLD))),
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
        Line::from(Span::styled("Insert Mode", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Enter     Send message"),
        Line::from("  Shift+Enter / Alt+Enter  New line"),
        Line::from("  Esc       Back to normal mode"),
        Line::from(""),
        Line::from(Span::styled("Global", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+c    Quit"),
        Line::from("  Ctrl+l    Clear chat"),
        Line::from("  Ctrl+t    Toggle details drawer"),
        Line::from("  Ctrl+p    Command palette"),
        Line::from(""),
        Line::from(Span::styled("Slash Commands", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /details  Toggle detail drawer"),
        Line::from("  /retry    Replay the last prompt"),
        Line::from("  /interrupt  Stop the active turn"),
    ];

    let paragraph = Paragraph::new(Text::from(help_text))
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(paragraph, popup_area);
}

fn render_command_palette(frame: &mut Frame, app: &App, area: Rect) {
    let palette_width = 50u16.min(area.width.saturating_sub(4));
    let x = (area.width - palette_width) / 2;
    let y = 2;
    let popup_area = Rect::new(x, y, palette_width, 3);

    frame.render_widget(Clear, popup_area);

    let input = Paragraph::new(Line::from(Span::raw(&app.command_input)))
        .block(
            Block::default()
                .title(" Command Palette (type command + Enter) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(input, popup_area);

    frame.set_cursor_position((
        popup_area.x + 1 + app.command_input.len() as u16,
        popup_area.y + 1,
    ));
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s[..max].to_string()
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, Terminal};

    use crate::interactive::app::{App, AppConfig};
    use crate::interactive::chat::MessageRole;

    use super::render;

    #[test]
    fn red_spec_3582_default_layout_collapses_detail_drawer_until_requested() {
        let mut app = App::new(AppConfig::default());
        app.push_message(
            MessageRole::Assistant,
            "Transcript should own the main canvas.".to_string(),
        );

        let rendered = render_app(&mut app, 120, 32);

        assert!(!rendered.contains("Tools ("));
    }

    #[test]
    fn red_spec_3582_status_bar_surfaces_session_and_approval_context() {
        let mut app = App::new(AppConfig::default());
        let rendered = render_app(&mut app, 120, 10);

        assert!(rendered.contains("session"));
        assert!(rendered.contains("approval"));
    }

    #[test]
    fn red_spec_3582_composer_hints_expose_interrupt_retry_and_details_actions() {
        let mut app = App::new(AppConfig::default());
        let rendered = render_app(&mut app, 120, 24);

        assert!(rendered.contains("interrupt"));
        assert!(rendered.contains("retry"));
        assert!(rendered.contains("details"));
    }

    #[test]
    fn red_spec_3582_composer_uses_prompt_shell_instead_of_bordered_panel_title() {
        let mut app = App::new(AppConfig::default());
        let rendered = render_app(&mut app, 120, 24);

        assert!(rendered.contains("Press / for commands"));
        assert!(!rendered.contains("Composer"));
    }

    #[test]
    fn red_spec_3582_transcript_shows_live_activity_summary_above_messages() {
        let mut app = App::new(AppConfig::default());
        app.status.agent_state = crate::interactive::status::AgentStateDisplay::Thinking;
        let rendered = render_app(&mut app, 120, 24);

        assert!(rendered.contains("Live activity"));
        assert!(rendered.contains("Thinking through the next step"));
    }

    #[test]
    fn red_spec_3582_details_drawer_exposes_context_sections_beyond_tools() {
        let mut app = App::new(AppConfig::default());
        app.show_tool_panel = true;
        let rendered = render_app(&mut app, 140, 32);

        assert!(rendered.contains("Memory"));
        assert!(rendered.contains("Cortex"));
        assert!(rendered.contains("Sessions"));
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
}
