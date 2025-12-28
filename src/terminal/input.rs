use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Get xterm-style modifier code for key combinations
/// Returns None if no modifiers are active, otherwise returns the modifier code (2-8)
fn get_modifier_code(modifiers: KeyModifiers) -> Option<u8> {
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    let alt = modifiers.contains(KeyModifiers::ALT);
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);

    match (ctrl, alt, shift) {
        (false, false, false) => None,
        (false, false, true) => Some(2),  // Shift
        (false, true, false) => Some(3),  // Alt
        (false, true, true) => Some(4),   // Shift+Alt
        (true, false, false) => Some(5),  // Ctrl
        (true, false, true) => Some(6),   // Ctrl+Shift
        (true, true, false) => Some(7),   // Ctrl+Alt
        (true, true, true) => Some(8),    // Ctrl+Alt+Shift
    }
}

/// Build CSI sequence for arrow keys with optional modifiers
/// Format: ESC[1;{mod}{letter} with modifiers, ESC[{letter} without
fn arrow_key_sequence(letter: u8, modifiers: KeyModifiers) -> Vec<u8> {
    match get_modifier_code(modifiers) {
        Some(m) => vec![0x1b, b'[', b'1', b';', m + b'0', letter],
        None => vec![0x1b, b'[', letter],
    }
}

/// Build CSI sequence for Home/End with optional modifiers
/// Format: ESC[1;{mod}{letter} with modifiers, ESC[{letter} without
fn home_end_sequence(letter: u8, modifiers: KeyModifiers) -> Vec<u8> {
    match get_modifier_code(modifiers) {
        Some(m) => vec![0x1b, b'[', b'1', b';', m + b'0', letter],
        None => vec![0x1b, b'[', letter],
    }
}

/// Build CSI sequence for keys that use ~-terminated format (PageUp, PageDown, Delete, Insert)
/// Format: ESC[{num};{mod}~ with modifiers, ESC[{num}~ without
fn tilde_key_sequence(num: u8, modifiers: KeyModifiers) -> Vec<u8> {
    match get_modifier_code(modifiers) {
        Some(m) => vec![0x1b, b'[', num + b'0', b';', m + b'0', b'~'],
        None => vec![0x1b, b'[', num + b'0', b'~'],
    }
}

/// Build sequence for F1-F4 (use SS3 format without modifiers, CSI format with modifiers)
fn f1_f4_sequence(letter: u8, modifiers: KeyModifiers) -> Vec<u8> {
    match get_modifier_code(modifiers) {
        Some(m) => vec![0x1b, b'[', b'1', b';', m + b'0', letter],
        None => vec![0x1b, b'O', letter],
    }
}

/// Build sequence for F5-F12 (use CSI ~-terminated format)
fn f5_plus_sequence(num_str: &[u8], modifiers: KeyModifiers) -> Vec<u8> {
    let mut seq = vec![0x1b, b'['];
    seq.extend_from_slice(num_str);
    if let Some(m) = get_modifier_code(modifiers) {
        seq.push(b';');
        seq.push(m + b'0');
    }
    seq.push(b'~');
    seq
}

/// Convert crossterm key events to bytes for PTY input
pub fn key_event_to_bytes(event: &KeyEvent) -> Option<Vec<u8>> {
    let modifiers = event.modifiers;

    let bytes = match event.code {
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                // Convert Ctrl+letter to control character
                let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a' - 1);
                vec![ctrl_char]
            } else if modifiers.contains(KeyModifiers::ALT) {
                // Alt key sends ESC followed by the character
                vec![0x1b, c as u8]
            } else {
                c.to_string().into_bytes()
            }
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => vec![0x1b, b'[', b'Z'], // Shift+Tab
        KeyCode::Esc => vec![0x1b],

        // Arrow keys with modifier support
        KeyCode::Up => arrow_key_sequence(b'A', modifiers),
        KeyCode::Down => arrow_key_sequence(b'B', modifiers),
        KeyCode::Right => arrow_key_sequence(b'C', modifiers),
        KeyCode::Left => arrow_key_sequence(b'D', modifiers),

        // Home/End with modifier support
        KeyCode::Home => home_end_sequence(b'H', modifiers),
        KeyCode::End => home_end_sequence(b'F', modifiers),

        // PageUp(5), PageDown(6), Insert(2), Delete(3) with modifier support
        KeyCode::PageUp => tilde_key_sequence(5, modifiers),
        KeyCode::PageDown => tilde_key_sequence(6, modifiers),
        KeyCode::Insert => tilde_key_sequence(2, modifiers),
        KeyCode::Delete => tilde_key_sequence(3, modifiers),

        // Function keys with modifier support
        KeyCode::F(n) => {
            match n {
                1 => f1_f4_sequence(b'P', modifiers),
                2 => f1_f4_sequence(b'Q', modifiers),
                3 => f1_f4_sequence(b'R', modifiers),
                4 => f1_f4_sequence(b'S', modifiers),
                5 => f5_plus_sequence(b"15", modifiers),
                6 => f5_plus_sequence(b"17", modifiers),
                7 => f5_plus_sequence(b"18", modifiers),
                8 => f5_plus_sequence(b"19", modifiers),
                9 => f5_plus_sequence(b"20", modifiers),
                10 => f5_plus_sequence(b"21", modifiers),
                11 => f5_plus_sequence(b"23", modifiers),
                12 => f5_plus_sequence(b"24", modifiers),
                _ => return None,
            }
        }
        _ => return None,
    };

    Some(bytes)
}

