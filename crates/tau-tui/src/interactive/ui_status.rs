use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::super::app::App;
use super::super::status::{AgentStateDisplay, CircuitBreakerDisplay};

pub(super) fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
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
    let cb_span = Span::styled(
        format!(
            " health={} ",
            app.status.circuit_breaker_state.label().to_lowercase()
        ),
        Style::default()
            .fg(Color::Black)
            .bg(circuit_breaker_color(app.status.circuit_breaker_state)),
    );
    let state_span = Span::styled(
        format!(" active={} ", app.status.agent_state.label().to_lowercase()),
        Style::default()
            .fg(Color::Black)
            .bg(agent_state_color(app.status.agent_state)),
    );
    let sep = Span::raw(" ");
    let line = Line::from(vec![
        model_span,
        sep.clone(),
        session_span,
        sep.clone(),
        workspace_span,
        sep.clone(),
        profile_span,
        sep.clone(),
        approval_span,
        sep.clone(),
        tokens_span,
        sep.clone(),
        cb_span,
        sep,
        state_span,
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(Color::Black)),
        area,
    );
}

fn circuit_breaker_color(state: CircuitBreakerDisplay) -> Color {
    match state {
        CircuitBreakerDisplay::Closed => Color::Green,
        CircuitBreakerDisplay::Open => Color::Red,
        CircuitBreakerDisplay::HalfOpen => Color::Yellow,
    }
}

fn agent_state_color(state: AgentStateDisplay) -> Color {
    match state {
        AgentStateDisplay::Idle => Color::DarkGray,
        AgentStateDisplay::Thinking => Color::Yellow,
        AgentStateDisplay::ToolExec => Color::Cyan,
        AgentStateDisplay::Streaming => Color::Green,
        AgentStateDisplay::Error => Color::Red,
    }
}
