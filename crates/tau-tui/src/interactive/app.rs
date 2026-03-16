//! Core application state and event loop for the interactive TUI.

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::chat::{ChatMessage, ChatPanel, MessageRole};
use super::gateway::GatewayInteractiveConfig;
use super::input::InputEditor;
use super::status::StatusBar;
use super::tools::{ToolEntry, ToolPanel, ToolStatus};
use super::ui;

/// Configuration for the interactive TUI application.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model: String,
    pub profile: String,
    pub tick_rate_ms: u64,
    pub gateway: Option<GatewayInteractiveConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: "openai/gpt-5.2".to_string(),
            profile: "local-dev".to_string(),
            tick_rate_ms: 100,
            gateway: None,
        }
    }
}

/// Which panel is currently focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Chat,
    Input,
    Tools,
    CommandPalette,
}

/// Input mode for the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

/// Main application state.
pub struct App {
    pub config: AppConfig,
    pub chat: ChatPanel,
    pub input: InputEditor,
    pub status: StatusBar,
    pub tools: ToolPanel,
    pub focus: FocusPanel,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub show_help: bool,
    pub command_input: String,
    pub show_tool_panel: bool,
    pub pending_assistant: String,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let status = StatusBar::new(config.model.clone(), config.profile.clone());
        Self {
            config,
            chat: ChatPanel::new(),
            input: InputEditor::new(),
            status,
            tools: ToolPanel::new(),
            focus: FocusPanel::Input,
            input_mode: InputMode::Insert,
            should_quit: false,
            show_help: false,
            command_input: String::new(),
            show_tool_panel: true,
            pending_assistant: String::new(),
        }
    }

    /// Process a key event and update app state.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global shortcuts that work in any mode
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.should_quit = true;
                return;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                // Clear chat
                self.chat.clear();
                return;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
                self.show_tool_panel = !self.show_tool_panel;
                return;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                if self.focus == FocusPanel::CommandPalette {
                    self.focus = FocusPanel::Input;
                } else {
                    self.focus = FocusPanel::CommandPalette;
                    self.command_input.clear();
                }
                return;
            }
            _ => {}
        }

        if self.focus == FocusPanel::CommandPalette {
            self.handle_command_palette_key(key);
            return;
        }

        if self.show_help {
            self.show_help = false;
            return;
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Insert => self.handle_insert_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('i') => {
                self.input_mode = InputMode::Insert;
                self.focus = FocusPanel::Input;
            }
            KeyCode::Char('a') => {
                self.input_mode = InputMode::Insert;
                self.focus = FocusPanel::Input;
                self.input.move_end();
            }
            KeyCode::Char('o') => {
                self.input_mode = InputMode::Insert;
                self.focus = FocusPanel::Input;
                self.input.new_line();
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.focus == FocusPanel::Chat {
                    self.chat.scroll_down(1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.focus == FocusPanel::Chat {
                    self.chat.scroll_up(1);
                }
            }
            KeyCode::Char('G') => {
                if self.focus == FocusPanel::Chat {
                    self.chat.scroll_to_bottom();
                }
            }
            KeyCode::Char('g') => {
                if self.focus == FocusPanel::Chat {
                    self.chat.scroll_to_top();
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.focus == FocusPanel::Chat {
                    self.chat.scroll_down(10);
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.focus == FocusPanel::Chat {
                    self.chat.scroll_up(10);
                }
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    FocusPanel::Chat => FocusPanel::Input,
                    FocusPanel::Input => {
                        if self.show_tool_panel {
                            FocusPanel::Tools
                        } else {
                            FocusPanel::Chat
                        }
                    }
                    FocusPanel::Tools => FocusPanel::Chat,
                    FocusPanel::CommandPalette => FocusPanel::Input,
                };
            }
            KeyCode::Char('1') => self.focus = FocusPanel::Chat,
            KeyCode::Char('2') => self.focus = FocusPanel::Input,
            KeyCode::Char('3') if self.show_tool_panel => self.focus = FocusPanel::Tools,
            _ => {}
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => {
                self.input_mode = InputMode::Normal;
            }
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Enter) => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.input.new_line();
                } else {
                    self.submit_input();
                }
            }
            (KeyModifiers::ALT, KeyCode::Enter) => {
                self.input.new_line();
            }
            (_, KeyCode::Char(c)) => {
                self.input.insert_char(c);
            }
            (_, KeyCode::Backspace) => {
                self.input.delete_backward();
            }
            (_, KeyCode::Delete) => {
                self.input.delete_forward();
            }
            (_, KeyCode::Left) => {
                self.input.move_left();
            }
            (_, KeyCode::Right) => {
                self.input.move_right();
            }
            (_, KeyCode::Up) => {
                self.input.move_up();
            }
            (_, KeyCode::Down) => {
                self.input.move_down();
            }
            (_, KeyCode::Home) => {
                self.input.move_home();
            }
            (_, KeyCode::End) => {
                self.input.move_end();
            }
            (_, KeyCode::Tab) => {
                // Tab cycles focus in insert mode too
                self.focus = match self.focus {
                    FocusPanel::Input => FocusPanel::Chat,
                    FocusPanel::Chat => {
                        if self.show_tool_panel {
                            FocusPanel::Tools
                        } else {
                            FocusPanel::Input
                        }
                    }
                    FocusPanel::Tools => FocusPanel::Input,
                    FocusPanel::CommandPalette => FocusPanel::Input,
                };
            }
            _ => {}
        }
    }

    fn handle_command_palette_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.focus = FocusPanel::Input;
                self.command_input.clear();
            }
            KeyCode::Enter => {
                let cmd = self.command_input.clone();
                self.focus = FocusPanel::Input;
                self.command_input.clear();
                self.execute_command(&cmd);
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, cmd: &str) {
        match cmd.trim() {
            "quit" | "q" => self.should_quit = true,
            "clear" => self.chat.clear(),
            "help" => self.show_help = true,
            "tools" => self.show_tool_panel = !self.show_tool_panel,
            _ => {
                self.chat.add_message(ChatMessage {
                    role: MessageRole::System,
                    content: format!("Unknown command: {cmd}"),
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                });
            }
        }
    }

    fn submit_input(&mut self) {
        let text = self.input.get_text();
        if text.trim().is_empty() {
            return;
        }

        // Handle slash commands
        if text.starts_with('/') {
            let cmd = text.trim_start_matches('/');
            self.execute_command(cmd);
            self.input.clear();
            return;
        }

        self.chat.add_message(ChatMessage {
            role: MessageRole::User,
            content: text.clone(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        self.status.total_messages += 1;
        self.status.total_tokens += text.len() as u64 / 4;
        self.chat.scroll_to_bottom();
        self.input.clear();

        if self.config.gateway.is_some() {
            self.pending_assistant.clear();
            self.status.agent_state = super::status::AgentStateDisplay::Thinking;
            return;
        }

        self.chat.add_message(ChatMessage {
            role: MessageRole::Assistant,
            content: format!(
                "Received your message. (Model: {}, {} chars)",
                self.config.model,
                text.len()
            ),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        self.status.total_messages += 1;
    }

    /// Push a chat message externally (for agent integration).
    pub fn push_message(&mut self, role: MessageRole, content: String) {
        self.chat.add_message(ChatMessage {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        self.chat.scroll_to_bottom();
    }

    /// Push a tool execution event externally.
    pub fn push_tool_event(&mut self, name: String, status: ToolStatus, detail: String) {
        self.tools.add_entry(ToolEntry {
            name,
            status,
            detail,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
    }
}

/// Run the interactive TUI application.
pub fn run_interactive(config: AppConfig) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config.clone());
    let tick_rate = Duration::from_millis(config.tick_rate_ms);

    // Welcome message
    app.chat.add_message(ChatMessage {
        role: MessageRole::System,
        content: format!(
            "Welcome to Tau Interactive Terminal. Model: {}. Press ? for help, Ctrl+C to quit.",
            config.model
        ),
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
    });

    let result = run_event_loop(&mut terminal, &mut app, tick_rate);

    // Restore terminal
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
