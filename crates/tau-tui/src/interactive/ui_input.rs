use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
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

    // Enable wrapping so long input lines don't scroll off screen
    frame.render_widget(
        Paragraph::new(Text::from(input_text)).wrap(Wrap { trim: false }),
        inner,
    );

    // Compute cursor position accounting for line wrapping
    if app.focus == FocusPanel::Input && app.input_mode == InputMode::Insert {
        let width = inner.width as usize;
        if width > 0 {
            let mut visual_y = 0u16;
            for (i, line) in app.input.lines().iter().enumerate() {
                if i == app.input.cursor_line() {
                    break;
                }
                let line_len = line.chars().count().max(1);
                visual_y += ((line_len + width - 1) / width) as u16;
            }

            let col = app.input.cursor_col();
            let wrap_row = (col / width) as u16;
            let wrap_col = (col % width) as u16;
            visual_y += wrap_row;

            let cursor_x = inner.x + wrap_col;
            let cursor_y = inner.y + visual_y;
            if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
}
