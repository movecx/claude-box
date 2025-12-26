use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Special value for no border
const NO_BORDER: &str = "none";

/// Predefined colors for selection (None first, then colors)
const COLORS: &[(&str, &str)] = &[
    ("none", "None"),
    ("#3498db", "Blue"),
    ("#27ae60", "Green"),
    ("#e74c3c", "Red"),
    ("#9b59b6", "Purple"),
    ("#f39c12", "Orange"),
    ("#1abc9c", "Teal"),
    ("#e91e63", "Pink"),
    ("#00bcd4", "Cyan"),
    ("#ff5722", "Deep Orange"),
    ("#607d8b", "Gray"),
];

#[derive(Debug, Clone)]
pub struct ColorPicker {
    selected: usize,
}

impl ColorPicker {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % COLORS.len();
    }

    pub fn prev(&mut self) {
        if self.selected == 0 {
            self.selected = COLORS.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn selected_hex(&self) -> String {
        COLORS[self.selected].0.to_string()
    }

    pub fn set_from_hex(&mut self, hex: &str) {
        for (i, (color_hex, _)) in COLORS.iter().enumerate() {
            if color_hex.eq_ignore_ascii_case(hex) {
                self.selected = i;
                return;
            }
        }
        // If not found, keep current selection
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, border_style: Style) {
        let block = Block::default()
            .title(" Border Color (←/→ to change) ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build color swatches
        let mut spans = Vec::new();
        for (i, (hex, name)) in COLORS.iter().enumerate() {
            let is_none = *hex == NO_BORDER;

            if i == self.selected {
                if is_none {
                    // "None" option - show without color swatch
                    spans.push(Span::styled(
                        format!(" [{}] ", name),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ));
                } else {
                    let (r, g, b) = parse_hex(hex);
                    let color = Color::Rgb(r, g, b);
                    spans.push(Span::styled(
                        format!(" [●] {} ", name),
                        Style::default().fg(color),
                    ));
                }
            } else {
                if is_none {
                    spans.push(Span::styled(
                        "  ○  ",
                        Style::default().fg(Color::DarkGray),
                    ));
                } else {
                    let (r, g, b) = parse_hex(hex);
                    let color = Color::Rgb(r, g, b);
                    spans.push(Span::styled(
                        "  ●  ",
                        Style::default().fg(color),
                    ));
                }
            }
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, inner);
    }
}

fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}
