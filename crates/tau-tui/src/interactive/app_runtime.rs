use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use super::app::{App, AppConfig};
use super::app_mouse::handle_mouse;
use super::chat::{ChatMessage, MessageRole};
use super::ui;

/// Run the interactive TUI application.
pub fn run_interactive(config: AppConfig) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config.clone());
    let tick_rate = Duration::from_millis(config.tick_rate_ms);

    app.chat.add_message(ChatMessage {
        role: MessageRole::System,
        content: format!(
            "Welcome to Tau Interactive Terminal. Model: {}. Press ? for help, Ctrl+C to quit.",
            config.model
        ),
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
    });

    let result = run_event_loop(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => app.handle_key(key),
                Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    let area = Rect::new(0, 0, size.width, size.height);
                    handle_mouse(app, mouse, area);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
