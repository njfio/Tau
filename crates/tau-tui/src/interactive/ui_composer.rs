use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use super::super::app::{App, FocusPanel, InputMode};

pub(super) fn input_height(app: &App) -> u16 {
    let lines = app.input.lines().len() as u16;
    lines.clamp(2, 6) + 3
}

pub(super) fn render_input(frame: &mut Frame, app: &App, area: Rect) {
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

    let composer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    render_input_lines(frame, app, composer_chunks[0]);
    render_footer(frame, app, composer_chunks[1]);
    render_cursor(frame, app, composer_chunks[0]);
}

fn render_input_lines(frame: &mut Frame, app: &App, area: Rect) {
    let lines = app
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
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let mode = match app.input_mode {
        InputMode::Insert => "insert",
        InputMode::Normal => "normal",
    };
    let actions =
        format!("Press / for commands  Enter send  Shift+Enter newline  Tab focus  {mode} mode");
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Compose ", Style::default().fg(Color::Black).bg(Color::Green)),
        Span::raw(" "),
        Span::styled(actions, Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(footer, area);
}

fn render_cursor(frame: &mut Frame, app: &App, area: Rect) {
    if app.focus != FocusPanel::Input || app.input_mode != InputMode::Insert {
        return;
    }
    let cursor_x = area.x + 2 + app.input.cursor_col() as u16;
    let cursor_y = area.y + app.input.cursor_line() as u16;
    if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
