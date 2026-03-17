use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::super::app::App;
use super::super::command_catalog::matching_commands;

pub(super) fn render_command_palette(frame: &mut Frame, app: &App, area: Rect) {
    let commands = matching_commands(&app.command_input);
    let popup_area = palette_area(area, commands.len() as u16);
    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .title(" Command Palette ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Rgb(8, 10, 14)));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    frame.render_widget(Paragraph::new(input_line(app)), chunks[0]);
    frame.render_widget(Paragraph::new(command_lines(commands)), chunks[1]);
    frame.render_widget(Paragraph::new(hint_line()), chunks[2]);
    frame.set_cursor_position((
        chunks[0].x + 1 + app.command_input.len() as u16,
        chunks[0].y,
    ));
}

fn palette_area(area: Rect, match_count: u16) -> Rect {
    let popup_width = 68u16.min(area.width.saturating_sub(4));
    let popup_height = (match_count.min(6) + 4)
        .max(6)
        .min(area.height.saturating_sub(4));
    Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    )
}

fn input_line(app: &App) -> Line<'static> {
    Line::from(vec![
        Span::styled("/", Style::default().fg(Color::LightGreen)),
        Span::raw(app.command_input.clone()),
    ])
}

fn command_lines(
    commands: Vec<&'static super::super::command_catalog::CommandSpec>,
) -> Vec<Line<'static>> {
    if commands.is_empty() {
        return vec![Line::from("No matching commands")];
    }
    commands
        .into_iter()
        .take(6)
        .map(|command| {
            Line::from(vec![
                Span::styled(
                    format!("/{:<16}", command.name),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(command.summary, Style::default().fg(Color::Gray)),
            ])
        })
        .collect()
}

fn hint_line() -> Line<'static> {
    Line::from(vec![
        Span::styled("Esc close", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("Enter run", Style::default().fg(Color::Green)),
    ])
}
