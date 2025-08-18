use anyhow::Result;
use crossterm::event::KeyCode;
use crate::tui::app::{TuiApp, AppMode, MetadataField};
use crate::tui::event::KeyEvent;

pub fn handle_find_test_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.close_find_test();
        }
        KeyCode::Enter => {
            // Jump to selected test and close finder
            app.jump_to_selected_search_result();
            app.close_find_test();
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            if !app.search_results.is_empty() {
                app.search_selected_index = 
                    (app.search_selected_index + 1) % app.search_results.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.search_results.is_empty() {
                if app.search_selected_index > 0 {
                    app.search_selected_index -= 1;
                } else {
                    app.search_selected_index = app.search_results.len() - 1;
                }
            }
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.update_search_results();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.update_search_results();
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_running_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            // Cancel execution
            app.test_receiver = None;  // Drop receiver to stop checking for messages
            app.append_output("\n\n[Test execution cancelled]\n");
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_category_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.show_categories = false;
            app.mode = AppMode::Normal;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_category_selection_down();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_category_selection_up();
        }
        KeyCode::Enter => {
            app.select_current_category();
            app.show_categories = false;
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_delete_confirmation(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Confirm deletion
            if let Err(e) = app.confirm_delete_test() {
                app.append_output(&format!("Failed to delete test: {e}\n"));
            } else {
                app.append_output("Test deleted successfully!\n");
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            // Cancel deletion
            app.cancel_delete();
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_edit_expected_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel editing
            app.cancel_edit_expected();
        }
        KeyCode::Enter => {
            // Save expected output
            if let Err(e) = app.save_expected_output() {
                app.append_output(&format!("Failed to save expected output: {e}\n"));
            }
        }
        KeyCode::Char(c) => {
            // Handle special escape sequences
            if c == '\\' && !app.editing_expected.ends_with('\\') {
                app.editing_expected.push('\\');
            } else if app.editing_expected.ends_with('\\') {
                let _ = app.editing_expected.pop();
                match c {
                    'n' => app.editing_expected.push('\n'),
                    't' => app.editing_expected.push('\t'),
                    'r' => app.editing_expected.push('\r'),
                    '\\' => app.editing_expected.push('\\'),
                    _ => {
                        app.editing_expected.push('\\');
                        app.editing_expected.push(c);
                    }
                }
            } else {
                app.editing_expected.push(c);
            }
        }
        KeyCode::Backspace => {
            app.editing_expected.pop();
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_metadata_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel metadata input
            app.metadata_input = crate::tui::app::MetadataInput::default();
            app.mode = AppMode::Normal;
        }
        KeyCode::Enter => {
            // Save metadata
            if let Err(e) = app.save_metadata() {
                app.append_output(&format!("Failed to save metadata: {e}\n"));
            } else {
                app.append_output("Metadata saved successfully!\n");
            }
        }
        KeyCode::Tab => {
            // Move to next field
            app.metadata_input.focused_field = match app.metadata_input.focused_field {
                MetadataField::ExpectedOutput => MetadataField::Description,
                MetadataField::Description => MetadataField::UseRuntime,
                MetadataField::UseRuntime => MetadataField::IsKnownFailure,
                MetadataField::IsKnownFailure => MetadataField::ExpectedOutput,
            };
        }
        KeyCode::BackTab => {
            // Move to previous field (Shift+Tab)
            app.metadata_input.focused_field = match app.metadata_input.focused_field {
                MetadataField::ExpectedOutput => MetadataField::IsKnownFailure,
                MetadataField::Description => MetadataField::ExpectedOutput,
                MetadataField::UseRuntime => MetadataField::Description,
                MetadataField::IsKnownFailure => MetadataField::UseRuntime,
            };
        }
        KeyCode::Char(' ') => {
            // Toggle checkbox fields
            match app.metadata_input.focused_field {
                MetadataField::UseRuntime => {
                    app.metadata_input.use_runtime = !app.metadata_input.use_runtime;
                }
                MetadataField::IsKnownFailure => {
                    app.metadata_input.is_known_failure = !app.metadata_input.is_known_failure;
                }
                _ => {}
            }
        }
        KeyCode::Char(c) => {
            // Add character to text fields
            match app.metadata_input.focused_field {
                MetadataField::ExpectedOutput => {
                    // Handle special escape sequences
                    if c == '\\' && !app.metadata_input.expected_output.ends_with('\\') {
                        app.metadata_input.expected_output.push('\\');
                    } else if app.metadata_input.expected_output.ends_with('\\') {
                        let _ = app.metadata_input.expected_output.pop();
                        match c {
                            'n' => app.metadata_input.expected_output.push('\n'),
                            't' => app.metadata_input.expected_output.push('\t'),
                            'r' => app.metadata_input.expected_output.push('\r'),
                            '\\' => app.metadata_input.expected_output.push('\\'),
                            _ => {
                                app.metadata_input.expected_output.push('\\');
                                app.metadata_input.expected_output.push(c);
                            }
                        }
                    } else {
                        app.metadata_input.expected_output.push(c);
                    }
                }
                MetadataField::Description => {
                    app.metadata_input.description.push(c);
                }
                _ => {}
            }
        }
        KeyCode::Backspace => {
            // Remove character from text fields
            match app.metadata_input.focused_field {
                MetadataField::ExpectedOutput => {
                    app.metadata_input.expected_output.pop();
                }
                MetadataField::Description => {
                    app.metadata_input.description.pop();
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_rename_test_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel renaming
            app.cancel_rename();
        }
        KeyCode::Enter => {
            // Save new name
            if let Err(e) = app.save_rename_test() {
                app.append_output(&format!("Failed to rename test: {e}\n"));
            }
        }
        KeyCode::Char(c) => {
            // Only allow alphanumeric, underscore, and hyphen in test names
            if c.is_alphanumeric() || c == '_' || c == '-' {
                app.rename_new_name.push(c);
            }
        }
        KeyCode::Backspace => {
            app.rename_new_name.pop();
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_move_test_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    // In move mode, we show the category selector
    // So we delegate to the category input handler
    match key.code {
        KeyCode::Esc => {
            // Cancel moving
            app.cancel_move();
        }
        KeyCode::Up => {
            app.move_category_selection_up();
        }
        KeyCode::Down => {
            app.move_category_selection_down();
        }
        KeyCode::Enter => {
            // Get selected category
            let target_category = if app.selected_category_index == 0 {
                // "All Tests" option - use Uncategorized
                "Uncategorized".to_string()
            } else {
                // Get the category at the current index
                let category_names: Vec<String> = app.categories.keys().cloned().collect();
                category_names.get(app.selected_category_index - 1)
                    .cloned()
                    .unwrap_or_else(|| "Uncategorized".to_string())
            };
            
            // Save the move with the selected category
            if let Err(e) = app.save_move_test(target_category) {
                app.append_output(&format!("Failed to move test: {e}\n"));
            }
        }
        _ => {}
    }
    Ok(true)
}

pub fn handle_create_test_input(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel creating
            app.cancel_create_test();
        }
        KeyCode::Enter => {
            // Save new test
            if let Err(e) = app.save_new_test() {
                app.append_output(&format!("Failed to create test: {e}\n"));
            }
        }
        KeyCode::Tab => {
            // Switch between name and description fields
            app.new_test_focused_field = !app.new_test_focused_field;
        }
        KeyCode::Char(c) => {
            if !app.new_test_focused_field {
                // Editing name field - only allow alphanumeric, underscore, and hyphen
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    app.new_test_name.push(c);
                }
            } else {
                // Editing description field
                app.new_test_description.push(c);
            }
        }
        KeyCode::Backspace => {
            if !app.new_test_focused_field {
                app.new_test_name.pop();
            } else {
                app.new_test_description.pop();
            }
        }
        _ => {}
    }
    Ok(true)
}