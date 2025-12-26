use crate::config::Config;
use crate::environment::EnvironmentManager;
use crate::tui::app::{Action, AppScreen};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct EnvironmentListScreen {
    pub selected: usize,
    pub env_keys: Vec<String>,
    pub list_state: ListState,
}

impl EnvironmentListScreen {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            selected: 0,
            env_keys: Vec::new(),
            list_state,
        }
    }

    pub fn refresh(&mut self, manager: &EnvironmentManager) {
        self.env_keys = manager
            .config()
            .environments
            .keys()
            .cloned()
            .collect();
        self.env_keys.sort();

        // Ensure selection is valid
        if !self.env_keys.is_empty() && self.selected >= self.env_keys.len() {
            self.selected = self.env_keys.len() - 1;
        }
        self.list_state.select(Some(self.selected));
    }

    pub fn render(&self, frame: &mut Frame, config: &Config) {
        let area = frame.area();

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // List
                Constraint::Length(3), // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Claude-Box Configuration")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(title, chunks[0]);

        // Environment list
        let default_env = config.default_environment.as_deref();

        let items: Vec<ListItem> = self
            .env_keys
            .iter()
            .map(|key| {
                let env_config = config.get_environment(key);
                let name = env_config.map(|c| c.name.as_str()).unwrap_or(key);

                let is_default = default_env == Some(key.as_str());
                let default_marker = if is_default { " (default)" } else { "" };

                let (r, g, b) = env_config
                    .map(|c| c.border_color_rgb())
                    .unwrap_or((255, 255, 255));

                let line = Line::from(vec![
                    Span::styled("● ", Style::default().fg(Color::Rgb(r, g, b))),
                    Span::raw(format!("{} ", name)),
                    Span::styled(
                        format!("[{}]", key),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(default_marker, Style::default().fg(Color::Yellow)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Environments ")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state.clone());

        // Help text
        let help = if self.env_keys.is_empty() {
            Paragraph::new("n: New environment | q: Quit")
        } else {
            Paragraph::new(
                "↑↓: Navigate | Enter: Settings | n: New | d: Delete | s: Set default | q: Quit",
            )
        };
        let help = help
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(help, chunks[2]);
    }

    pub fn handle_input(&mut self, key: KeyCode) -> Action {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('n') => Action::Goto(AppScreen::CreateEnvironment),
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.env_keys.is_empty() && self.selected > 0 {
                    self.selected -= 1;
                    self.list_state.select(Some(self.selected));
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.env_keys.is_empty() && self.selected < self.env_keys.len() - 1 {
                    self.selected += 1;
                    self.list_state.select(Some(self.selected));
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(key) = self.env_keys.get(self.selected) {
                    Action::Goto(AppScreen::EnvironmentSettings {
                        env_key: key.clone(),
                    })
                } else {
                    Action::None
                }
            }
            KeyCode::Char('d') => {
                if let Some(key) = self.env_keys.get(self.selected) {
                    Action::Goto(AppScreen::ConfirmDelete {
                        env_key: key.clone(),
                    })
                } else {
                    Action::None
                }
            }
            KeyCode::Char('s') => {
                if let Some(key) = self.env_keys.get(self.selected) {
                    Action::SetDefault { key: key.clone() }
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }
}
