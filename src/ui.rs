use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, AppState, SessionAction};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(5),    // Main content
        Constraint::Length(3), // Help bar
    ])
    .split(frame.area());

    render_title(frame, chunks[0]);
    render_session_list(frame, chunks[1], app);
    render_help_bar(frame, chunks[2], app);

    // Render input popup if creating session
    if app.state == AppState::CreatingSession {
        render_create_popup(frame, app);
    }

    // Render rename popup if renaming session
    if matches!(app.state, AppState::RenamingSession { .. }) {
        render_rename_popup(frame, app);
    }

    // Render error message if any
    if let Some(ref error) = app.error_message {
        render_error_popup(frame, error);
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("  Ursa - Tmux Session Manager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn render_session_list(frame: &mut Frame, area: Rect, app: &App) {
    let mut items: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let attached_indicator = if session.attached { " (attached)" } else { "" };
            let is_selected = i == app.selected_index;

            // Build action buttons for existing sessions
            let enter_style = if is_selected && app.selected_action == SessionAction::Enter {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let rename_style = if is_selected && app.selected_action == SessionAction::Rename {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let delete_style = if is_selected && app.selected_action == SessionAction::Delete {
                Style::default().fg(Color::Black).bg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::raw(&session.name),
                Span::styled(
                    format!(
                        " [{} window{}]{}",
                        session.windows,
                        if session.windows == 1 { "" } else { "s" },
                        attached_indicator
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled("[Enter]", enter_style),
                Span::raw(" "),
                Span::styled("[Rename]", rename_style),
                Span::raw(" "),
                Span::styled("[Delete]", delete_style),
            ]))
        })
        .collect();

    // Add "Create new session" option
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  + ", Style::default().fg(Color::Green)),
        Span::styled("Create new session", Style::default().fg(Color::Green)),
    ])));

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Sessions ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_help_bar(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.state {
        AppState::SessionList => {
            vec![
                Span::styled(" ↑↓/jk ", Style::default().fg(Color::Yellow)),
                Span::raw("Navigate  "),
                Span::styled("←→/hl ", Style::default().fg(Color::Yellow)),
                Span::raw("Action  "),
                Span::styled("Enter ", Style::default().fg(Color::Yellow)),
                Span::raw("Confirm  "),
                Span::styled("r ", Style::default().fg(Color::Yellow)),
                Span::raw("Refresh  "),
                Span::styled("q/Esc ", Style::default().fg(Color::Yellow)),
                Span::raw("Quit"),
            ]
        }
        AppState::CreatingSession => {
            vec![
                Span::styled("Enter ", Style::default().fg(Color::Yellow)),
                Span::raw("Create  "),
                Span::styled("Esc ", Style::default().fg(Color::Yellow)),
                Span::raw("Cancel"),
            ]
        }
        AppState::RenamingSession { .. } => {
            vec![
                Span::styled("Enter ", Style::default().fg(Color::Yellow)),
                Span::raw("Rename  "),
                Span::styled("Esc ", Style::default().fg(Color::Yellow)),
                Span::raw("Cancel"),
            ]
        }
    };

    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(help, area);
}

fn render_input_popup(frame: &mut Frame, app: &App, title: &str, border_color: Color) {
    let area = centered_rect(50, 20, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let input_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = input_block.inner(area);

    frame.render_widget(input_block, area);

    // Render the input text with cursor
    let input_text = format!("{}_", app.input_buffer);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White));
    frame.render_widget(input, inner);
}

fn render_create_popup(frame: &mut Frame, app: &App) {
    render_input_popup(frame, app, " New Session Name ", Color::Cyan);
}

fn render_rename_popup(frame: &mut Frame, app: &App) {
    render_input_popup(frame, app, " Rename Session ", Color::Yellow);
}

fn render_error_popup(frame: &mut Frame, error: &str) {
    let area = centered_rect(60, 15, frame.area());

    frame.render_widget(Clear, area);

    let error_block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = error_block.inner(area);
    frame.render_widget(error_block, area);

    let error_text = Paragraph::new(error)
        .style(Style::default().fg(Color::Red));
    frame.render_widget(error_text, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
