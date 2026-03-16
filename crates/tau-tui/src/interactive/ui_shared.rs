use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

use super::super::app::App;
use super::super::tools::{ToolEntry, ToolStatus};

pub(super) fn latest_running_tool(app: &App) -> Option<&ToolEntry> {
    app.tools
        .entries()
        .iter()
        .rev()
        .find(|entry| entry.status == ToolStatus::Running)
}

pub(super) fn badge(text: &str, background: Color) -> Span<'static> {
    Span::styled(
        text.to_string(),
        Style::default()
            .fg(Color::Black)
            .bg(background)
            .add_modifier(Modifier::BOLD),
    )
}

pub(super) fn action(text: &str, color: Color) -> Span<'static> {
    Span::styled(text.to_string(), Style::default().fg(color))
}
