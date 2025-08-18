use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use crate::tui::app::{TuiApp, AppMode};
use crate::tui::ui::widgets::{draw_test_list, draw_details_panel, draw_status_bar};
use crate::tui::modals::{
    draw_help_modal, draw_category_selector, draw_find_test_modal,
    draw_metadata_input_modal, draw_delete_confirmation_modal,
    draw_edit_expected_modal, draw_rename_test_modal, draw_create_test_modal,
};

pub fn draw(f: &mut Frame, app: &mut TuiApp) {
    let size = f.size();

    // Main layout - always use full screen, help will be a modal
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    // Top area layout - horizontal split
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Test list
            Constraint::Percentage(65), // Details/Output
        ])
        .split(main_chunks[0]);

    // Draw test list
    draw_test_list(f, top_chunks[0], app);

    // Right side - tabbed view for different content
    draw_details_panel(f, top_chunks[1], app);

    // Draw help modal if visible
    if app.show_help {
        draw_help_modal(f, size, app);
    }

    // Draw category selector modal if visible (for both category selection and move mode)
    if app.show_categories {
        draw_category_selector(f, size, app);
    }

    // Draw find test modal if in find mode
    if app.mode == AppMode::FindTest {
        draw_find_test_modal(f, size, app);
    }

    // Draw metadata input modal if adding metadata
    if app.mode == AppMode::AddingMetadata {
        draw_metadata_input_modal(f, size, app);
    }

    // Draw delete confirmation modal if confirming deletion
    if app.mode == AppMode::ConfirmDelete {
        draw_delete_confirmation_modal(f, size, app);
    }

    // Draw edit expected output modal if editing
    if app.mode == AppMode::EditingExpected {
        draw_edit_expected_modal(f, size, app);
    }
    
    // Draw rename test modal if renaming
    if app.mode == AppMode::RenamingTest {
        draw_rename_test_modal(f, size, app);
    }
    
    // Draw create test modal if creating
    if app.mode == AppMode::CreatingTest {
        draw_create_test_modal(f, size, app);
    }

    // Draw status bar at bottom
    draw_status_bar(f, size, app);
}

#[allow(dead_code)]
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}