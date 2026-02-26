mod create_modal;
mod footer;
mod keybindings_overlay;
mod layout;

use crate::tui::app::{AppState, Mode};
use crate::tui::components::{log_panel, preview, task_list};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

/// Renders the full TUI frame, including modal overlays for active modes.
pub fn render(frame: &mut Frame, state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());
    let main_area = layout[0];
    let footer_area = layout[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_area);

    task_list::render(frame, chunks[0], &state.tasks, state.selected_index);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(78), Constraint::Percentage(22)])
        .split(chunks[1]);

    preview::render(frame, right[0], &state.preview_text);
    log_panel::render(frame, right[1], &state.log_entries);
    footer::render_key_hints(frame, footer_area);

    if let Mode::Creating(create_state) = &state.mode {
        create_modal::render_create_modal(frame, create_state, main_area);
    }
    if matches!(state.mode, Mode::Keybindings) {
        keybindings_overlay::render_keybindings_overlay(frame, main_area);
    }
}
