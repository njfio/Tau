//! UI rendering with ratatui — multi-panel layout.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};

use super::app::{App, FocusPanel, InputMode};
use super::chat::MessageRole;
use super::status::{AgentStateDisplay, CircuitBreakerDisplay};
use super::tools::ToolStatus;

/// Render the full application UI.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main vertical layout: status bar (1) | body | input (3-5) | help line (1)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                 // Status bar
            Constraint::Min(8),                    // Body (chat + tools)
            Constraint::Length(input_height(app)), // Input
            Constraint::Length(1),                 // Help/mode line
        ])
        .split(size);

    render_status_bar(frame, app, main_chunks[0]);
    render_body(frame, app, main_chunks[1]);
    render_input(frame, app, main_chunks[2]);
    render_help_line(frame, app, main_chunks[3]);

    // Overlays
    if app.show_help {
        render_help_overlay(frame, size);
    }

    if app.focus == FocusPanel::CommandPalette {
        render_command_palette(frame, app, size);
    }
}

fn input_height(app: &App) -> u16 {
    let lines = app.input.lines().len() as u16;
    lines.clamp(3, 8) + 2 // +2 for borders
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let model_span = Span::styled(
        format!(" {} ", app.status.model),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    );

    let profile_span = Span::styled(
        format!(" {} ", app.status.profile),
        Style::default().fg(Color::Black).bg(Color::Blue),
    );

    let tokens_span = Span::styled(
        format!(" Tokens: {} ", app.status.format_tokens()),
        Style::default().fg(Color::White).bg(Color::DarkGray),
    );

    let cost_span = Span::styled(
        format!(" Cost: {} ", app.status.format_cost()),
        Style::default().fg(Color::White).bg(Color::DarkGray),
    );

    let msgs_span = Span::styled(
        format!(" Msgs: {} ", app.status.total_messages),
        Style::default().fg(Color::White).bg(Color::DarkGray),
    );

    let cb_color = match app.status.circuit_breaker_state {
        CircuitBreakerDisplay::Closed => Color::Green,
        CircuitBreakerDisplay::Open => Color::Red,
        CircuitBreakerDisplay::HalfOpen => Color::Yellow,
    };
    let cb_span = Span::styled(
        format!(" CB: {} ", app.status.circuit_breaker_state.label()),
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
        format!(" {} ", app.status.agent_state.label()),
        Style::default().fg(Color::Black).bg(state_color),
    );

    let sep = Span::raw(" ");

    let line = Line::from(vec![
        model_span,
        sep.clone(),
        profile_span,
        sep.clone(),
        tokens_span,
        sep.clone(),
        cost_span,
        sep.clone(),
        msgs_span,
        sep.clone(),
        cb_span,
        sep,
        state_span,
    ]);

    let bar = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(bar, area);
}

fn render_body(frame: &mut Frame, app: &App, area: Rect) {
    if app.show_tool_panel {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        render_chat_panel(frame, app, chunks[0]);
        render_tool_panel(frame, app, chunks[1]);
    } else {
        render_chat_panel(frame, app, area);
    }
}

fn render_chat_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::Chat;
    let border_style = if is_focused {
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

    if app.chat.is_empty() {
        let empty = Paragraph::new("No messages yet. Type below and press Enter.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    // Build text lines from messages
    let mut lines: Vec<Line> = Vec::new();
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

    let total_lines = lines.len();
    let visible_height = inner.height as usize;

    // Auto-scroll: compute scroll so latest messages are visible
    let scroll = if total_lines > visible_height {
        // If user has scrolled to a specific message, use that
        let msg_idx = app.chat.scroll_offset();
        if msg_idx >= app.chat.len().saturating_sub(1) {
            // At bottom — show last lines
            (total_lines - visible_height) as u16
        } else {
            // Approximate: each message ~3 lines
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

    // Scrollbar
    if total_lines > visible_height {
        let mut scrollbar_state = ScrollbarState::new(total_lines).position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn render_tool_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::Tools;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let active = app.tools.active_count();
    let total = app.tools.total_count();
    let title = format!(" Tools ({active} active / {total} total) ");

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tools.entries().is_empty() {
        let empty =
            Paragraph::new("No tool executions yet.").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    // Show most recent entries that fit
    let visible = inner.height as usize;
    let entries = app.tools.entries();
    let start = entries.len().saturating_sub(visible);

    let items: Vec<ListItem> = entries[start..]
        .iter()
        .map(|entry| {
            let status_style = match entry.status {
                ToolStatus::Running => Style::default().fg(Color::Yellow),
                ToolStatus::Success => Style::default().fg(Color::Green),
                ToolStatus::Failed => Style::default().fg(Color::Red),
                ToolStatus::Timeout => Style::default().fg(Color::Magenta),
            };

            let line = Line::from(vec![
                Span::styled(format!("[{}] ", entry.status.label()), status_style),
                Span::styled(&entry.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    truncate(
                        &entry.detail,
                        (inner.width as usize).saturating_sub(entry.name.len() + 8),
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::Input;
    let border_color = if is_focused {
        match app.input_mode {
            InputMode::Insert => Color::Green,
            InputMode::Normal => Color::Cyan,
        }
    } else {
        Color::DarkGray
    };

    let mode_label = match app.input_mode {
        InputMode::Insert => " INSERT ",
        InputMode::Normal => " NORMAL ",
    };

    let block = Block::default()
        .title(Span::styled(
            " Input ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            mode_label,
            Style::default()
                .fg(Color::Black)
                .bg(if app.input_mode == InputMode::Insert {
                    Color::Green
                } else {
                    Color::Cyan
                }),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input_text: Vec<Line> = app
        .input
        .lines()
        .iter()
        .map(|l| Line::from(Span::raw(l.as_str())))
        .collect();

    let paragraph = Paragraph::new(Text::from(input_text));
    frame.render_widget(paragraph, inner);

    // Show cursor in insert mode
    if is_focused && app.input_mode == InputMode::Insert {
        let cursor_x = inner.x + app.input.cursor_col() as u16;
        let cursor_y = inner.y + app.input.cursor_line() as u16;
        if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn render_help_line(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.input_mode {
        InputMode::Normal => vec![
            ("i", "insert"),
            ("q", "quit"),
            ("?", "help"),
            ("Tab", "focus"),
            ("j/k", "scroll"),
            ("1-3", "panel"),
            ("C-p", "cmd"),
            ("C-t", "tools"),
        ],
        InputMode::Insert => vec![
            ("Esc", "normal"),
            ("Enter", "send"),
            ("S-Enter", "newline"),
            ("Tab", "focus"),
            ("C-c", "quit"),
            ("C-p", "cmd"),
        ],
    };

    let spans: Vec<Span> = hints
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {key} "),
                    Style::default().fg(Color::Black).bg(Color::DarkGray),
                ),
                Span::styled(format!("{desc} "), Style::default().fg(Color::DarkGray)),
            ]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let help_width = 60u16.min(area.width.saturating_sub(4));
    let help_height = 22u16.min(area.height.saturating_sub(4));
    let x = (area.width - help_width) / 2;
    let y = (area.height - help_height) / 2;
    let popup_area = Rect::new(x, y, help_width, help_height);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Tau Interactive TUI — Keyboard Reference",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Normal Mode",
            Style::default().add_modifier(Modifier::BOLD),
        )),
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
        Line::from(Span::styled(
            "Insert Mode",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Enter     Send message"),
        Line::from("  Shift+Enter / Alt+Enter  New line"),
        Line::from("  Esc       Back to normal mode"),
        Line::from(""),
        Line::from(Span::styled(
            "Global",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Ctrl+c    Quit"),
        Line::from("  Ctrl+l    Clear chat"),
        Line::from("  Ctrl+t    Toggle tool panel"),
        Line::from("  Ctrl+p    Command palette"),
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
