use crate::config::ProviderConfig;
use crate::environment::EnvironmentManager;
use crate::tui::log_tui_error;
use crate::tui::screens::{
    ConfirmDeleteScreen, CreateEnvironmentScreen, EnvironmentListScreen, EnvironmentSettingsScreen,
};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::time::Duration;

/// Current screen/state in the TUI
#[derive(Debug, Clone)]
pub enum AppScreen {
    EnvironmentList,
    CreateEnvironment,
    EnvironmentSettings { env_key: String },
    ConfirmDelete { env_key: String },
}

/// Actions that screens can request
#[derive(Debug, Clone)]
pub enum Action {
    None,
    Quit,
    Goto(AppScreen),
    CreateEnv {
        key: String,
        name: String,
        color: String,
        provider: ProviderConfig,
    },
    DeleteEnv { key: String },
    SetDefault { key: String },
    UpdateEnv {
        key: String,
        name: String,
        color: String,
        provider: ProviderConfig,
    },
}

/// Main TUI application
pub struct App {
    pub manager: EnvironmentManager,
    pub current_screen: AppScreen,
    pub should_quit: bool,

    // Screen states
    pub list_screen: EnvironmentListScreen,
    pub create_screen: CreateEnvironmentScreen,
    pub settings_screen: EnvironmentSettingsScreen,
    pub delete_screen: ConfirmDeleteScreen,
}

impl App {
    pub fn new() -> Result<Self> {
        let manager = EnvironmentManager::new()?;
        Ok(Self {
            manager,
            current_screen: AppScreen::EnvironmentList,
            should_quit: false,
            list_screen: EnvironmentListScreen::new(),
            create_screen: CreateEnvironmentScreen::new(),
            settings_screen: EnvironmentSettingsScreen::new(),
            delete_screen: ConfirmDeleteScreen::new(),
        })
    }

    /// Navigate to a screen
    pub fn goto(&mut self, screen: AppScreen) {
        match &screen {
            AppScreen::CreateEnvironment => {
                self.create_screen.reset();
            }
            AppScreen::EnvironmentSettings { env_key } => {
                if let Some(config) = self.manager.config().get_environment(env_key) {
                    self.settings_screen.load(env_key.clone(), config.clone());
                }
            }
            AppScreen::ConfirmDelete { env_key } => {
                self.delete_screen.set_env_key(env_key.clone());
            }
            _ => {}
        }
        self.current_screen = screen;
    }

    /// Update list screen's environment count
    pub fn refresh_list(&mut self) {
        self.list_screen.refresh(&self.manager);
    }

    /// Handle an action from a screen
    fn handle_action(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Goto(screen) => {
                self.goto(screen);
            }
            Action::CreateEnv { key, name, color, provider } => {
                match self.manager.create_environment(&key, name, Some(color)) {
                    Ok(()) => {
                        // Apply provider config
                        if let Some(config) = self.manager.config_mut().environments.get_mut(&key) {
                            config.provider = provider;
                        }
                        if let Err(err) = self.manager.save() {
                            log_tui_error("save_config_after_create", err.as_ref());
                        }
                        self.refresh_list();
                        self.goto(AppScreen::EnvironmentList);
                    }
                    Err(err) => {
                        log_tui_error("create_environment", err.as_ref());
                    }
                }
            }
            Action::DeleteEnv { key } => {
                if let Err(err) = self.manager.delete_environment(&key) {
                    log_tui_error("delete_environment", err.as_ref());
                }
                self.refresh_list();
                self.goto(AppScreen::EnvironmentList);
            }
            Action::SetDefault { key } => {
                if let Err(err) = self.manager.set_default(Some(&key)) {
                    log_tui_error("set_default", err.as_ref());
                }
            }
            Action::UpdateEnv { key, name, color, provider } => {
                if let Some(config) = self.manager.config_mut().environments.get_mut(&key) {
                    config.name = name;
                    config.border_color = color;
                    config.provider = provider;
                }
                if let Err(err) = self.manager.save() {
                    log_tui_error("update_environment", err.as_ref());
                }
                self.refresh_list();
                self.goto(AppScreen::EnvironmentList);
            }
        }
    }
}

/// Run the configuration TUI
pub fn run_config_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    app.refresh_list();

    // Main loop
    let result = run_app_loop(&mut terminal, &mut app);
    if let Err(ref err) = result {
        log_tui_error("run_app_loop", err.as_ref());
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    while !app.should_quit {
        // Render current screen
        let current_screen = app.current_screen.clone();
        let config = app.manager.config().clone();

        terminal.draw(|frame| {
            match &current_screen {
                AppScreen::EnvironmentList => app.list_screen.render(frame, &config),
                AppScreen::CreateEnvironment => app.create_screen.render(frame),
                AppScreen::EnvironmentSettings { .. } => app.settings_screen.render(frame),
                AppScreen::ConfirmDelete { .. } => app.delete_screen.render(frame, &config),
            }
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let action = match &current_screen {
                        AppScreen::EnvironmentList => {
                            app.list_screen.handle_input(key.code)
                        }
                        AppScreen::CreateEnvironment => {
                            app.create_screen.handle_input(key.code)
                        }
                        AppScreen::EnvironmentSettings { .. } => {
                            app.settings_screen.handle_input(key.code)
                        }
                        AppScreen::ConfirmDelete { .. } => {
                            app.delete_screen.handle_input(key.code)
                        }
                    };
                    app.handle_action(action);
                }
            }
        }
    }

    Ok(())
}
