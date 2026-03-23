use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::app::{App, InputMode};
use super::status::{AgentStateDisplay, CircuitBreakerDisplay};

pub(crate) fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let model_span = Span::styled(
        format!(" {} ", app.status.model),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    );
    let profile_span = Span::styled(
        format!(" {} ", app.status.profile),
        Style::default().fg(Color::Black).bg(Color::Blue),
    );
    let transport_span = Span::styled(
        format!(" {} ", app.status.transport.label()),
        Style::default().fg(Color::Black).bg(Color::Magenta),
    );
    let skills_span = app.status.format_active_skills().map(|skills| {
        Span::styled(
            format!(" Skills: {} ", skills),
            Style::default().fg(Color::Black).bg(Color::Yellow),
        )
    });
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
    let mut spans = vec![
        model_span,
        sep.clone(),
        profile_span,
        sep.clone(),
        transport_span,
    ];
    if let Some(skills_span) = skills_span {
        spans.push(sep.clone());
        spans.push(skills_span);
    }
    spans.extend([
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
    let line = Line::from(spans);

    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(Color::Black)),
        area,
    );
}

pub(crate) fn render_help_line(frame: &mut Frame, app: &App, area: Rect) {
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

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
