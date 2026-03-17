use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::super::app::App;
#[path = "ui_run_state_model.rs"]
mod model;

pub(super) fn run_state_height(app: &App) -> u16 {
    model::build_run_state_card(app)
        .map(|card| 5 + u16::from(card.meta.is_some()) + u16::from(card.preview.is_some()))
        .unwrap_or(0)
}

pub(super) fn render_run_state_card(frame: &mut Frame, app: &App, area: Rect) {
    let Some(card) = model::build_run_state_card(app) else {
        return;
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Rgb(12, 15, 20)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(run_state_paragraph(card), inner);
}

fn run_state_paragraph(card: model::RunStateCard<'static>) -> Paragraph<'static> {
    Paragraph::new(run_state_lines(card)).wrap(Wrap { trim: true })
}

fn run_state_lines(card: model::RunStateCard<'static>) -> Vec<Line<'static>> {
    let lines = vec![
        Line::from(vec![
            super::shared::badge(card.title, card.color),
            Span::raw(" "),
            Span::styled(card.context, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(Span::styled(
            card.primary,
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            card.secondary,
            Style::default().fg(Color::Gray),
        )),
    ];
    let lines = extend_with_meta(lines, card.meta);
    extend_with_preview(lines, card.preview)
}

fn extend_with_meta(mut lines: Vec<Line<'static>>, meta: Option<String>) -> Vec<Line<'static>> {
    if let Some(text) = meta {
        lines.push(Line::from(Span::styled(
            text,
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines
}

fn extend_with_preview(
    mut lines: Vec<Line<'static>>,
    preview: Option<String>,
) -> Vec<Line<'static>> {
    if let Some(text) = preview {
        lines.push(Line::from(Span::styled(
            text,
            Style::default().fg(Color::LightCyan),
        )));
    }
    lines
}
