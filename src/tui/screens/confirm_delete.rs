use crate::config::Config;
use crate::tui::app::{Action, AppScreen};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct ConfirmDeleteScreen {
    env_key: String,
    selected_yes: bool,
}

impl ConfirmDeleteScreen {
    pub fn new() -> Self {
        Self {
            env_key: String::new(),
            selected_yes: false,
        }
    }

    pub fn set_env_key(&mut self, key: String) {
        self.env_key = key;
        self.selected_yes = false;
    }

    pub fn render(&self, frame: &mut Frame, config: &Config) {
        let area = frame.area();

        // Center the dialog
        let dialog_width = 50;
        let dialog_height = 10;
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Length(2), // Message
                Constraint::Length(2), // Buttons
            ])
            .split(dialog_area);

        // Dialog box
        let block = Block::default()
            .title(" Confirm Delete ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        frame.render_widget(block, dialog_area);

        // Message
        let name = config
            .get_environment(&self.env_key)
            .map(|c| c.name.as_str())
            .unwrap_or(&self.env_key);

        let message = Paragraph::new(format!("Delete environment \"{}\"?", name))
            .alignment(Alignment::Center);
        frame.render_widget(message, chunks[1]);

        // Buttons
        let yes_style = if self.selected_yes {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let no_style = if !self.selected_yes {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let buttons = Line::from(vec![
            Span::styled("  [ Yes ]  ", yes_style),
            Span::raw("    "),
            Span::styled("  [ No ]  ", no_style),
        ]);
        let buttons = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons, chunks[2]);
    }

    pub fn handle_input(&mut self, key: KeyCode) -> Action {
        match key {
            KeyCode::Esc | KeyCode::Char('n') => Action::Goto(AppScreen::EnvironmentList),
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.selected_yes = !self.selected_yes;
                Action::None
            }
            KeyCode::Enter => {
                if self.selected_yes {
                    Action::DeleteEnv {
                        key: self.env_key.clone(),
                    }
                } else {
                    Action::Goto(AppScreen::EnvironmentList)
                }
            }
            KeyCode::Char('y') => Action::DeleteEnv {
                key: self.env_key.clone(),
            },
            _ => Action::None,
        }
    }
}
