use crate::config::EnvironmentConfig;
use crate::pty::{spawn_claude_code, ClaudeProcess};
use crate::terminal::border::BorderStyle;
use crate::terminal::input::key_event_to_bytes;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::PtySize;
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::Color,
    style::Style,
    widgets::{Block, Borders, BorderType, Paragraph},
    Terminal,
};
use std::io::{stdout, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tui_term::widget::PseudoTerminal;

/// Main terminal wrapper that runs Claude Code with a border
pub struct TerminalWrapper {
    border_style: BorderStyle,
    claude_config_dir: std::path::PathBuf,
    env_config: EnvironmentConfig,
    working_dir: std::path::PathBuf,
}

impl TerminalWrapper {
    pub fn new(env_config: &EnvironmentConfig, claude_config_dir: &Path, working_dir: &Path) -> Self {
        let border_style = BorderStyle::from_hex(
            &env_config.border_color,
            format!(" {} ", env_config.name),
        );

        Self {
            border_style,
            claude_config_dir: claude_config_dir.to_path_buf(),
            env_config: env_config.clone(),
            working_dir: working_dir.to_path_buf(),
        }
    }

    /// Run the terminal wrapper (blocking)
    pub fn run(&self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Get terminal size (subtract 2 for border)
        let size = terminal.size()?;
        let pty_size = PtySize {
            rows: size.height.saturating_sub(2).max(1),
            cols: size.width.saturating_sub(2).max(1),
            pixel_width: 0,
            pixel_height: 0,
        };

        // Spawn Claude Code with provider configuration
        let mut process = spawn_claude_code(&self.claude_config_dir, &self.env_config, pty_size, &self.working_dir)?;

        // Create vt100 parser
        let mut parser = vt100::Parser::new(pty_size.rows, pty_size.cols, 1000);

        // Setup PTY reader in separate thread
        let mut reader = process.master.try_clone_reader()?;
        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Get writer for PTY
        let mut writer = process.master.take_writer()?;

        // Main loop
        let result = self.run_loop(&mut terminal, &mut parser, &rx, &mut writer, &mut process);

        // Cleanup
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

        result
    }

    fn run_loop(
        &self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
        parser: &mut vt100::Parser,
        rx: &mpsc::Receiver<Vec<u8>>,
        writer: &mut Box<dyn Write + Send>,
        process: &mut ClaudeProcess,
    ) -> Result<()> {
        // Scrollback offset: 0 = live view (bottom), >0 = scrolled up into history
        let mut scroll_offset: usize = 0;

        loop {
            // Check if process has exited
            if let Ok(Some(_status)) = process.child.try_wait() {
                break;
            }

            // Read PTY output
            while let Ok(data) = rx.try_recv() {
                parser.process(&data);
            }

            // Handle resize
            let size = terminal.size()?;
            let new_pty_size = PtySize {
                rows: size.height.saturating_sub(2).max(1),
                cols: size.width.saturating_sub(2).max(1),
                pixel_width: 0,
                pixel_height: 0,
            };

            if new_pty_size.rows != parser.screen().size().0
                || new_pty_size.cols != parser.screen().size().1
            {
                process.master.resize(new_pty_size)?;
                parser.set_size(new_pty_size.rows, new_pty_size.cols);
            }

            // Max scrollback is the configured buffer size (1000 lines)
            // set_scrollback() will clamp to actual available content
            let max_scroll: usize = 1000;

            // Set scrollback position for rendering
            // set_scrollback clamps to actual available content, so read back the real value
            parser.set_scrollback(scroll_offset);
            scroll_offset = parser.screen().scrollback();

            // Render
            let current_offset = scroll_offset;
            let border_title = self.border_style.title.clone();
            let border_color = self.border_style.color;

            terminal.draw(|frame| {
                let area = frame.area();

                // Create border block - show scroll indicator if scrolled up
                let title = if current_offset > 0 {
                    format!("{} [↑ {} lines] ", border_title.trim_end(), current_offset)
                } else {
                    border_title.clone()
                };

                let block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(border_color));

                let inner = block.inner(area);
                frame.render_widget(block, area);

                if inner.width == 0 || inner.height == 0 {
                    let message = Paragraph::new("Terminal too small. Resize to continue.")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Red));
                    frame.render_widget(message, area);
                } else {
                    // Render terminal content (will show scrolled view due to set_scrollback)
                    let pseudo_term = PseudoTerminal::new(parser.screen());
                    frame.render_widget(pseudo_term, inner);
                }
            })?;

            // Reset scrollback to 0 so new PTY output appears correctly
            parser.set_scrollback(0);

            // Handle input events
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::PageUp => {
                                // Scroll up into history
                                scroll_offset = (scroll_offset + 10).min(max_scroll);
                            }
                            KeyCode::PageDown => {
                                // Scroll down toward live view
                                scroll_offset = scroll_offset.saturating_sub(10);
                            }
                            _ => {
                                // Any other keypress that sends input resets scroll to live view
                                if let Some(bytes) = key_event_to_bytes(&key) {
                                    scroll_offset = 0;
                                    writer.write_all(&bytes)?;
                                    writer.flush()?;
                                }
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        match mouse.kind {
                            MouseEventKind::ScrollUp => {
                                // Scroll up into history (increase offset)
                                scroll_offset = (scroll_offset + 3).min(max_scroll);
                            }
                            MouseEventKind::ScrollDown => {
                                // Scroll down toward live view (decrease offset)
                                scroll_offset = scroll_offset.saturating_sub(3);
                            }
                            _ => {}
                        }
                    }
                    Event::Resize(_, _) => {
                        // Resize handled above
                    }
                    Event::Paste(text) => {
                        scroll_offset = 0;
                        writer.write_all(text.as_bytes())?;
                        writer.flush()?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
