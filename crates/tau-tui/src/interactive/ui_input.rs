use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::{App, FocusPanel, InputMode};

pub(crate) fn input_height(app: &App) -> u16 {
    let lines = app.input.lines().len() as u16;
    lines.clamp(3, 8) + 2
}

pub(crate) fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == FocusPanel::Input {
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
        .map(|line| Line::from(Span::raw(line.as_str())))
        .collect();

    frame.render_widget(Paragraph::new(Text::from(input_text)), inner);

    if app.focus == FocusPanel::Input && app.input_mode == InputMode::Insert {
        let cursor_x = inner.x + app.input.cursor_col() as u16;
        let cursor_y = inner.y + app.input.cursor_line() as u16;
        if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
