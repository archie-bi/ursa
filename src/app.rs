use crate::tmux::{self, TmuxSession};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    SessionList,
    CreatingSession,
    RenamingSession { original_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SessionAction {
    #[default]
    Enter,
    Rename,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppAction {
    None,
    AttachSession(String),
    Quit,
}

pub struct App {
    pub state: AppState,
    pub sessions: Vec<TmuxSession>,
    pub selected_index: usize,
    pub selected_action: SessionAction,
    pub input_buffer: String,
    pub should_quit: bool,
    pub action: AppAction,
    pub error_message: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let sessions = tmux::list_sessions();
        Self {
            state: AppState::SessionList,
            sessions,
            selected_index: 0,
            selected_action: SessionAction::default(),
            input_buffer: String::new(),
            should_quit: false,
            action: AppAction::None,
            error_message: None,
        }
    }

    pub fn refresh_sessions(&mut self) {
        self.sessions = tmux::list_sessions();
        // Ensure selected index is within bounds (max is sessions.len() for "Create new")
        let max_index = self.sessions.len(); // "Create new" is at this index
        if self.selected_index > max_index {
            self.selected_index = max_index;
        }
    }

    /// Total items = sessions + "Create new session" option
    pub fn total_items(&self) -> usize {
        self.sessions.len() + 1
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Clear error on any keypress
        self.error_message = None;

        match &self.state {
            AppState::SessionList => self.handle_session_list_key(key),
            AppState::CreatingSession => self.handle_creating_session_key(key),
            AppState::RenamingSession { .. } => self.handle_renaming_session_key(key),
        }
    }

    fn handle_session_list_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    // Reset action to Enter when changing selection
                    self.selected_action = SessionAction::Enter;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.total_items() - 1 {
                    self.selected_index += 1;
                    // Reset action to Enter when changing selection
                    self.selected_action = SessionAction::Enter;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                // Only allow action cycling for existing sessions (not "Create new")
                if self.selected_index < self.sessions.len() {
                    self.selected_action = match self.selected_action {
                        SessionAction::Enter => SessionAction::Rename,
                        SessionAction::Rename => SessionAction::Delete,
                        SessionAction::Delete => SessionAction::Delete, // Stop at edge
                    };
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                // Only allow action cycling for existing sessions (not "Create new")
                if self.selected_index < self.sessions.len() {
                    self.selected_action = match self.selected_action {
                        SessionAction::Enter => SessionAction::Enter, // Stop at edge
                        SessionAction::Rename => SessionAction::Enter,
                        SessionAction::Delete => SessionAction::Rename,
                    };
                }
            }
            KeyCode::Enter => {
                self.select_current();
            }
            KeyCode::Char('r') => {
                self.refresh_sessions();
            }
            _ => {}
        }
    }

    fn handle_creating_session_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::SessionList;
                self.input_buffer.clear();
            }
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    self.create_and_attach_session();
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                // Only allow valid tmux session name characters
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    self.input_buffer.push(c);
                }
            }
            _ => {}
        }
    }

    fn select_current(&mut self) {
        if self.selected_index == self.sessions.len() {
            // "Create new session" selected
            self.state = AppState::CreatingSession;
            self.input_buffer.clear();
        } else if let Some(session) = self.sessions.get(self.selected_index) {
            match self.selected_action {
                SessionAction::Enter => {
                    // Attach to session
                    self.action = AppAction::AttachSession(session.name.clone());
                }
                SessionAction::Rename => {
                    // Enter rename mode
                    self.state = AppState::RenamingSession {
                        original_name: session.name.clone(),
                    };
                    self.input_buffer = session.name.clone();
                }
                SessionAction::Delete => {
                    // Delete the session
                    self.delete_current_session();
                }
            }
        }
    }

    fn delete_current_session(&mut self) {
        let Some(session) = self.sessions.get(self.selected_index) else {
            return;
        };
        let name = session.name.clone();

        match tmux::kill_session(&name) {
            Ok(()) => {
                self.refresh_sessions();
                self.selected_action = SessionAction::Enter;
            }
            Err(e) => {
                self.error_message = Some(e);
            }
        }
    }

    fn create_and_attach_session(&mut self) {
        let name = self.input_buffer.trim().to_string();
        if name.is_empty() {
            return;
        }

        match tmux::create_session(&name) {
            Ok(()) => {
                self.action = AppAction::AttachSession(name);
            }
            Err(e) => {
                self.error_message = Some(e);
                self.state = AppState::SessionList;
                self.input_buffer.clear();
            }
        }
    }

    fn handle_renaming_session_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::SessionList;
                self.input_buffer.clear();
                self.selected_action = SessionAction::Enter;
            }
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    self.rename_current_session();
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                // Only allow valid tmux session name characters
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    self.input_buffer.push(c);
                }
            }
            _ => {}
        }
    }

    fn rename_current_session(&mut self) {
        let new_name = self.input_buffer.trim().to_string();
        if new_name.is_empty() {
            return;
        }

        // Extract original_name from the state
        let original_name = if let AppState::RenamingSession { original_name } = &self.state {
            original_name.clone()
        } else {
            return;
        };

        match tmux::rename_session(&original_name, &new_name) {
            Ok(()) => {
                self.state = AppState::SessionList;
                self.input_buffer.clear();
                self.selected_action = SessionAction::Enter;
                self.refresh_sessions();
            }
            Err(e) => {
                self.error_message = Some(e);
                self.state = AppState::SessionList;
                self.input_buffer.clear();
                self.selected_action = SessionAction::Enter;
            }
        }
    }
}
