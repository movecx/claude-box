use ratatui::style::Color;

/// Border style configuration
#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub color: Color,
    pub title: String,
}

impl BorderStyle {
    pub fn new(color: Color, title: String) -> Self {
        Self { color, title }
    }

    /// Create from hex color string
    pub fn from_hex(hex: &str, title: String) -> Self {
        let color = parse_hex_to_color(hex);
        Self { color, title }
    }
}

/// Parse hex color string to ratatui Color
fn parse_hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Blue; // Default fallback
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(52);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(152);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(219);

    Color::Rgb(r, g, b)
}
