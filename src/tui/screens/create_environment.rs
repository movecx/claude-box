use crate::config::{ProviderConfig, ProviderModels, ProviderPreset};
use crate::tui::app::{Action, AppScreen};
use crate::tui::widgets::ColorPicker;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    Key,
    Name,
    Color,
    Provider,
    ApiKey,
    CustomUrl,
    OpusModel,
    SonnetModel,
    HaikuModel,
}

impl Field {
    fn all() -> &'static [Field] {
        &[
            Field::Key,
            Field::Name,
            Field::Color,
            Field::Provider,
            Field::ApiKey,
            Field::CustomUrl,
            Field::OpusModel,
            Field::SonnetModel,
            Field::HaikuModel,
        ]
    }

    fn next(&self) -> Field {
        let fields = Self::all();
        let idx = fields.iter().position(|f| f == self).unwrap_or(0);
        fields[(idx + 1) % fields.len()]
    }

    fn prev(&self) -> Field {
        let fields = Self::all();
        let idx = fields.iter().position(|f| f == self).unwrap_or(0);
        if idx == 0 {
            fields[fields.len() - 1]
        } else {
            fields[idx - 1]
        }
    }
}

pub struct CreateEnvironmentScreen {
    pub key_input: String,
    pub name_input: String,
    pub color_picker: ColorPicker,
    current_field: Field,
    error: Option<String>,
    confirming: bool,

    // Provider settings
    provider_preset: ProviderPreset,
    api_key: String,
    custom_url: String,
    opus_model: String,
    sonnet_model: String,
    haiku_model: String,
}

impl CreateEnvironmentScreen {
    pub fn new() -> Self {
        Self {
            key_input: String::new(),
            name_input: String::new(),
            color_picker: ColorPicker::new(),
            current_field: Field::Key,
            error: None,
            confirming: false,
            provider_preset: ProviderPreset::Anthropic,
            api_key: String::new(),
            custom_url: String::new(),
            opus_model: String::new(),
            sonnet_model: String::new(),
            haiku_model: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.key_input.clear();
        self.name_input.clear();
        self.color_picker = ColorPicker::new();
        self.current_field = Field::Key;
        self.error = None;
        self.confirming = false;
        self.provider_preset = ProviderPreset::Anthropic;
        self.api_key.clear();
        self.custom_url.clear();
        self.opus_model.clear();
        self.sonnet_model.clear();
        self.haiku_model.clear();
    }

    fn build_provider_config(&self) -> ProviderConfig {
        let models = if self.provider_preset != ProviderPreset::Anthropic
            && (!self.opus_model.is_empty()
                || !self.sonnet_model.is_empty()
                || !self.haiku_model.is_empty())
        {
            Some(ProviderModels {
                opus: self.opus_model.clone(),
                sonnet: self.sonnet_model.clone(),
                haiku: self.haiku_model.clone(),
            })
        } else {
            None
        };

        ProviderConfig {
            preset: self.provider_preset,
            api_key: if self.api_key.is_empty() {
                None
            } else {
                Some(self.api_key.clone())
            },
            custom_base_url: if self.custom_url.is_empty() || self.provider_preset != ProviderPreset::Custom {
                None
            } else {
                Some(self.custom_url.clone())
            },
            models,
        }
    }

    fn is_field_visible(&self, field: Field) -> bool {
        match field {
            Field::Key | Field::Name | Field::Color | Field::Provider => true,
            Field::ApiKey => self.provider_preset != ProviderPreset::Anthropic,
            Field::CustomUrl => self.provider_preset == ProviderPreset::Custom,
            Field::OpusModel | Field::SonnetModel | Field::HaikuModel => {
                self.provider_preset != ProviderPreset::Anthropic
            }
        }
    }

    fn update_fields_for_preset(&mut self) {
        if let Some(defaults) = self.provider_preset.default_models() {
            self.opus_model = defaults.opus;
            self.sonnet_model = defaults.sonnet;
            self.haiku_model = defaults.haiku;
        } else {
            // Clear fields for Custom or Anthropic
            self.opus_model.clear();
            self.sonnet_model.clear();
            self.haiku_model.clear();
        }
        // Clear custom URL when not using Custom preset
        if self.provider_preset != ProviderPreset::Custom {
            self.custom_url.clear();
        }
    }

    fn is_text_field(&self, field: Field) -> bool {
        matches!(
            field,
            Field::Key | Field::Name | Field::ApiKey | Field::CustomUrl | Field::OpusModel | Field::SonnetModel | Field::HaikuModel
        )
    }

    fn is_last_visible_text_field(&self) -> bool {
        // Find all visible text fields
        let visible_text_fields: Vec<Field> = Field::all()
            .iter()
            .copied()
            .filter(|f| self.is_field_visible(*f) && self.is_text_field(*f))
            .collect();

        // Check if current field is the last visible text field
        visible_text_fields.last() == Some(&self.current_field)
    }

    fn next_visible_field(&self) -> Field {
        let mut field = self.current_field.next();
        while !self.is_field_visible(field) && field != self.current_field {
            field = field.next();
        }
        field
    }

    fn prev_visible_field(&self) -> Field {
        let mut field = self.current_field.prev();
        while !self.is_field_visible(field) && field != self.current_field {
            field = field.prev();
        }
        field
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Min(10),   // Content
                Constraint::Length(2), // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Create New Environment")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(title, chunks[0]);

        // Content area
        self.render_fields(frame, chunks[1]);

        // Help
        let help_text = if self.current_field == Field::Provider {
            "←/→: Change provider | Tab: Next field | Enter: Create | Esc: Cancel"
        } else if self.current_field == Field::Color {
            "←/→: Change color | Tab: Next field | Enter: Create | Esc: Cancel"
        } else {
            "Tab: Next field | Enter: Create | Esc: Cancel"
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);

        // Error message overlay
        if let Some(ref error) = self.error {
            let error_area = Rect::new(
                area.x + 2,
                area.y + area.height - 4,
                area.width - 4,
                1,
            );
            let error_text = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_text, error_area);
        }

        // Confirmation dialog overlay
        if self.confirming {
            let dialog_width = 40;
            let dialog_height = 5;
            let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
            let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

            let dialog_area = Rect::new(
                area.x + dialog_x,
                area.y + dialog_y,
                dialog_width,
                dialog_height,
            );

            let dialog = Block::default()
                .title(" Confirm ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));

            let inner = dialog.inner(dialog_area);
            frame.render_widget(ratatui::widgets::Clear, dialog_area);
            frame.render_widget(dialog, dialog_area);

            let text = Paragraph::new("Create environment?\n\n[Y]es  [N]o / Cancel")
                .style(Style::default().fg(Color::White))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(text, inner);
        }
    }

    fn render_fields(&self, frame: &mut Frame, area: Rect) {
        let mut constraints = vec![
            Constraint::Length(3), // Key
            Constraint::Length(3), // Name
            Constraint::Length(3), // Color
            Constraint::Length(3), // Provider
        ];

        if self.provider_preset != ProviderPreset::Anthropic {
            constraints.push(Constraint::Length(3)); // API Key
            if self.provider_preset == ProviderPreset::Custom {
                constraints.push(Constraint::Length(3)); // Custom URL
            }
            constraints.push(Constraint::Length(3)); // Opus
            constraints.push(Constraint::Length(3)); // Sonnet
            constraints.push(Constraint::Length(3)); // Haiku
        }
        constraints.push(Constraint::Min(0)); // Spacer

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut chunk_idx = 0;

        // Key input
        let key_style = self.field_style(Field::Key);
        let key_block = Block::default()
            .title(" Key (identifier) ")
            .borders(Borders::ALL)
            .border_style(key_style);
        let key_text = Paragraph::new(self.key_input.as_str()).block(key_block);
        frame.render_widget(key_text, chunks[chunk_idx]);
        chunk_idx += 1;

        // Name input
        let name_style = self.field_style(Field::Name);
        let name_block = Block::default()
            .title(" Display Name ")
            .borders(Borders::ALL)
            .border_style(name_style);
        let name_text = Paragraph::new(self.name_input.as_str()).block(name_block);
        frame.render_widget(name_text, chunks[chunk_idx]);
        chunk_idx += 1;

        // Color picker
        let color_style = self.field_style(Field::Color);
        self.color_picker.render(frame, chunks[chunk_idx], color_style);
        chunk_idx += 1;

        // Provider selector
        let provider_style = self.field_style(Field::Provider);
        self.render_provider_selector(frame, chunks[chunk_idx], provider_style);
        chunk_idx += 1;

        // Provider-specific fields
        if self.provider_preset != ProviderPreset::Anthropic {
            // API Key
            let api_style = self.field_style(Field::ApiKey);
            let api_block = Block::default()
                .title(" API Key ")
                .borders(Borders::ALL)
                .border_style(api_style);
            let masked_key = if self.api_key.is_empty() {
                String::new()
            } else {
                format!("{}...", &self.api_key.chars().take(8).collect::<String>())
            };
            let api_text = Paragraph::new(
                if self.current_field == Field::ApiKey {
                    self.api_key.as_str()
                } else {
                    masked_key.as_str()
                }
            ).block(api_block);
            frame.render_widget(api_text, chunks[chunk_idx]);
            chunk_idx += 1;

            // Custom URL (only for Custom provider)
            if self.provider_preset == ProviderPreset::Custom {
                let url_style = self.field_style(Field::CustomUrl);
                let url_block = Block::default()
                    .title(" Base URL ")
                    .borders(Borders::ALL)
                    .border_style(url_style);
                let url_text = Paragraph::new(self.custom_url.as_str()).block(url_block);
                frame.render_widget(url_text, chunks[chunk_idx]);
                chunk_idx += 1;
            }

            // Model fields
            let opus_style = self.field_style(Field::OpusModel);
            let opus_block = Block::default()
                .title(" Opus Model ")
                .borders(Borders::ALL)
                .border_style(opus_style);
            let opus_text = Paragraph::new(self.opus_model.as_str()).block(opus_block);
            frame.render_widget(opus_text, chunks[chunk_idx]);
            chunk_idx += 1;

            let sonnet_style = self.field_style(Field::SonnetModel);
            let sonnet_block = Block::default()
                .title(" Sonnet Model ")
                .borders(Borders::ALL)
                .border_style(sonnet_style);
            let sonnet_text = Paragraph::new(self.sonnet_model.as_str()).block(sonnet_block);
            frame.render_widget(sonnet_text, chunks[chunk_idx]);
            chunk_idx += 1;

            let haiku_style = self.field_style(Field::HaikuModel);
            let haiku_block = Block::default()
                .title(" Haiku Model ")
                .borders(Borders::ALL)
                .border_style(haiku_style);
            let haiku_text = Paragraph::new(self.haiku_model.as_str()).block(haiku_block);
            frame.render_widget(haiku_text, chunks[chunk_idx]);
        }
    }

    fn field_style(&self, field: Field) -> Style {
        if self.current_field == field {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        }
    }

    fn render_provider_selector(&self, frame: &mut Frame, area: Rect, border_style: Style) {
        let block = Block::default()
            .title(" Provider (←/→ to change) ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut spans = Vec::new();
        for preset in ProviderPreset::all() {
            let is_selected = *preset == self.provider_preset;
            let style = if is_selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            if is_selected {
                spans.push(Span::styled(format!(" [{}] ", preset.name()), style));
            } else {
                spans.push(Span::styled(format!("  {}  ", preset.name()), style));
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, inner);
    }

    pub fn handle_input(&mut self, key: KeyCode) -> Action {
        // Handle confirmation prompt
        if self.confirming {
            match key {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.confirming = false;
                    let name = if self.name_input.is_empty() {
                        self.key_input.clone()
                    } else {
                        self.name_input.clone()
                    };
                    return Action::CreateEnv {
                        key: self.key_input.clone(),
                        name,
                        color: self.color_picker.selected_hex(),
                        provider: self.build_provider_config(),
                    };
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirming = false;
                    return Action::Goto(AppScreen::EnvironmentList);
                }
                _ => return Action::None,
            }
        }

        match key {
            KeyCode::Esc => Action::Goto(AppScreen::EnvironmentList),
            KeyCode::Tab => {
                self.current_field = self.next_visible_field();
                Action::None
            }
            KeyCode::BackTab => {
                self.current_field = self.prev_visible_field();
                Action::None
            }
            KeyCode::Enter => {
                // If on a text field that's not the last one, move to next field
                if self.is_text_field(self.current_field) && !self.is_last_visible_text_field() {
                    self.current_field = self.next_visible_field();
                    return Action::None;
                }

                // Otherwise, validate and show confirmation
                if self.key_input.is_empty() {
                    self.error = Some("Key cannot be empty".to_string());
                    return Action::None;
                }
                self.confirming = true;
                Action::None
            }
            KeyCode::Up => {
                self.current_field = self.prev_visible_field();
                Action::None
            }
            KeyCode::Down => {
                self.current_field = self.next_visible_field();
                Action::None
            }
            KeyCode::Left => {
                match self.current_field {
                    Field::Color => self.color_picker.prev(),
                    Field::Provider => {
                        let presets = ProviderPreset::all();
                        let idx = presets.iter().position(|p| *p == self.provider_preset).unwrap_or(0);
                        self.provider_preset = if idx == 0 {
                            presets[presets.len() - 1]
                        } else {
                            presets[idx - 1]
                        };
                        self.update_fields_for_preset();
                    }
                    _ => {}
                }
                Action::None
            }
            KeyCode::Right => {
                match self.current_field {
                    Field::Color => self.color_picker.next(),
                    Field::Provider => {
                        let presets = ProviderPreset::all();
                        let idx = presets.iter().position(|p| *p == self.provider_preset).unwrap_or(0);
                        self.provider_preset = presets[(idx + 1) % presets.len()];
                        self.update_fields_for_preset();
                    }
                    _ => {}
                }
                Action::None
            }
            KeyCode::Char(c) => {
                match self.current_field {
                    Field::Key => {
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            self.key_input.push(c);
                            self.error = None;
                        }
                    }
                    Field::Name => {
                        self.name_input.push(c);
                        self.error = None;
                    }
                    Field::ApiKey => {
                        self.api_key.push(c);
                    }
                    Field::CustomUrl => {
                        self.custom_url.push(c);
                    }
                    Field::OpusModel => {
                        self.opus_model.push(c);
                    }
                    Field::SonnetModel => {
                        self.sonnet_model.push(c);
                    }
                    Field::HaikuModel => {
                        self.haiku_model.push(c);
                    }
                    _ => {}
                }
                Action::None
            }
            KeyCode::Backspace => {
                match self.current_field {
                    Field::Key => { self.key_input.pop(); }
                    Field::Name => { self.name_input.pop(); }
                    Field::ApiKey => { self.api_key.pop(); }
                    Field::CustomUrl => { self.custom_url.pop(); }
                    Field::OpusModel => { self.opus_model.pop(); }
                    Field::SonnetModel => { self.sonnet_model.pop(); }
                    Field::HaikuModel => { self.haiku_model.pop(); }
                    _ => {}
                }
                Action::None
            }
            _ => Action::None,
        }
    }
}
