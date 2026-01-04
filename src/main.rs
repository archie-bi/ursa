mod app;
mod tmux;
mod ui;

use std::time::Duration;

use app::{App, AppAction};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();

    // Handle post-TUI actions (attaching to session)
    if let Ok(Some(AppAction::AttachSession(name))) = result {
        if let Err(e) = tmux::attach_session(&name) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> Result<Option<AppAction>> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Poll for events with a timeout to allow for potential refresh
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }

        if app.should_quit {
            return Ok(Some(AppAction::Quit));
        }

        if let AppAction::AttachSession(name) = &app.action {
            return Ok(Some(AppAction::AttachSession(name.clone())));
        }
    }
}
