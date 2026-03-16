use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::app::{App, AppConfig};
use super::chat::MessageRole;
use super::ui;

pub fn run_interactive(config: AppConfig) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let tick_rate = Duration::from_millis(config.tick_rate_ms);
    let mut app = App::new(config.clone());
    app.push_message(
        MessageRole::System,
        format!(
            "Welcome to Tau Interactive Terminal. Model: {}. Press ? for help, Ctrl+C to quit.",
            config.model
        ),
    );
    let result = run_event_loop(&mut terminal, &mut app, tick_rate);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;
        if event::poll(tick_rate.saturating_sub(last_tick.elapsed()))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
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
