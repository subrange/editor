use anyhow::Result;
use crossterm::event::KeyCode;
use crate::tui::app::{TuiApp, AppMode};
use crate::tui::event::KeyEvent;
use crate::tui::handlers::modal::{
    handle_find_test_input, handle_running_input, handle_category_input,
    handle_metadata_input, handle_delete_confirmation, handle_edit_expected_input,
    handle_rename_test_input, handle_move_test_input, handle_create_test_input,
};

pub fn handle_input(
    app: &mut TuiApp,
    key: KeyEvent,
) -> Result<bool> {
    // Handle help scrolling first if help is open
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                app.show_help = false;
                app.help_scroll = 0;
                return Ok(true);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.help_scroll = app.help_scroll.saturating_add(1);
                return Ok(true);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
                return Ok(true);
            }
            KeyCode::PageDown => {
                app.help_scroll = app.help_scroll.saturating_add(10);
                return Ok(true);
            }
            KeyCode::PageUp => {
                app.help_scroll = app.help_scroll.saturating_sub(10);
                return Ok(true);
            }
            _ => return Ok(true), // Ignore other keys when help is open
        }
    }

    match app.mode {
        AppMode::Normal => Ok(true), // Normal mode is handled in runner.rs
        AppMode::FindTest => handle_find_test_input(app, key),
        AppMode::Running => handle_running_input(app, key),
        AppMode::SelectCategory => handle_category_input(app, key),
        AppMode::AddingMetadata => handle_metadata_input(app, key),
        AppMode::ConfirmDelete => handle_delete_confirmation(app, key),
        AppMode::EditingExpected => handle_edit_expected_input(app, key),
        AppMode::RenamingTest => handle_rename_test_input(app, key),
        AppMode::MovingTest => handle_move_test_input(app, key),
        AppMode::CreatingTest => handle_create_test_input(app, key),
    }
}