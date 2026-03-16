use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::super::app::App;
use super::drawer;

pub(super) fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let help_width = 60u16.min(area.width.saturating_sub(4));
    let help_height = 22u16.min(area.height.saturating_sub(4));
    let popup_area = Rect::new(
        (area.width - help_width) / 2,
        (area.height - help_height) / 2,
        help_width,
        help_height,
    );
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Tau Interactive TUI — Keyboard Reference",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
        Line::from("  Ctrl+t    Toggle details drawer"),
        Line::from("  Ctrl+p    Command palette"),
        Line::from(""),
        Line::from(Span::styled(
            "Slash Commands",
            Style::default().add_modifier(Modifier::BOLD),
        )),
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

pub(super) fn render_command_palette(frame: &mut Frame, app: &App, area: Rect) {
    let palette_width = 50u16.min(area.width.saturating_sub(4));
    let popup_area = Rect::new((area.width - palette_width) / 2, 2, palette_width, 3);
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
    frame.set_cursor_position((popup_area.x + 1 + app.command_input.len() as u16, popup_area.y + 1));
}

pub(super) fn render_detail_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = area.width.saturating_sub(6).min(52);
    let popup_height = area.height.saturating_sub(6).min(16);
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );
    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .title(" Quick details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Rgb(8, 10, 14)));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);
    drawer::render_detail_contents(frame, app, inner);
}
