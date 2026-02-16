use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::actions::Action;
use crate::app::state::{App, InputMode, View};

/// Map a key event to an action based on current state
pub fn handle_key_event(app: &App, key: KeyEvent) -> Action {
    // Global: Ctrl+C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    // Route based on input mode (popups take priority)
    match app.input_mode {
        InputMode::Search => handle_search_keys(key),
        InputMode::TagInput | InputMode::NotesInput | InputMode::RunDialog => {
            handle_input_keys(key)
        }
        InputMode::TagList => handle_tag_list_keys(key),
        InputMode::DeleteConfirm => handle_delete_confirm_keys(key),
        InputMode::Normal => {
            // q only quits in Normal mode
            if key.code == KeyCode::Char('q') {
                return Action::Quit;
            }
            match app.current_view {
                View::Dashboard => handle_dashboard_keys(key),
                View::RunDetail => handle_run_detail_keys(key),
                View::Compare => handle_compare_keys(key),
                View::GpuMonitor => handle_gpu_keys(key),
                View::Help => handle_help_keys(key),
            }
        }
    }
}

fn handle_dashboard_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
        KeyCode::Enter | KeyCode::Char('l') => Action::Select,
        KeyCode::Tab => Action::NextTab,
        KeyCode::BackTab => Action::PrevTab,

        KeyCode::Char('/') => Action::EnterSearchMode,
        KeyCode::Char('d') => Action::DeleteRun,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char(' ') => Action::ToggleCompareSelection,
        KeyCode::Char('c') => Action::GoToCompare,
        KeyCode::Char('g') => Action::GoToGpuMonitor,
        KeyCode::Char('R') => Action::OpenRunDialog,

        _ => Action::None,
    }
}

fn handle_run_detail_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Backspace => Action::Back,
        KeyCode::Tab => Action::CycleMetric,
        KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,

        KeyCode::Char('s') => Action::ToggleRunStatus,
        KeyCode::Char('t') => Action::OpenTagList,
        KeyCode::Char('n') => Action::OpenNotesEditor,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('g') => Action::GoToGpuMonitor,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('K') => Action::StopContainer,

        KeyCode::Char('m') => Action::ExportMarkdown,
        KeyCode::Char('e') => Action::ExportCsv,

        _ => Action::None,
    }
}

fn handle_compare_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => Action::Back,
        KeyCode::Tab => Action::CycleCompareMetric,
        KeyCode::Char('g') => Action::GoToGpuMonitor,
        KeyCode::Char('?') => Action::ToggleHelp,
        _ => Action::None,
    }
}

fn handle_gpu_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('g') => Action::Back,
        KeyCode::Char('?') => Action::ToggleHelp,
        _ => Action::None,
    }
}

fn handle_help_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => Action::ToggleHelp,
        _ => Action::None,
    }
}

fn handle_search_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::ExitSearchMode,
        KeyCode::Enter => Action::ExitSearchMode,
        KeyCode::Backspace => Action::SearchBackspace,
        KeyCode::Char(c) => Action::SearchInput(c),
        _ => Action::None,
    }
}

fn handle_input_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::InputCancel,
        KeyCode::Enter => Action::InputConfirm,
        KeyCode::Backspace => Action::InputBackspace,
        KeyCode::Char(c) => Action::InputChar(c),
        _ => Action::None,
    }
}

fn handle_tag_list_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::InputCancel,
        KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
        KeyCode::Char('a') => Action::OpenTagInput,
        KeyCode::Char('d') | KeyCode::Delete => Action::RemoveSelectedTag,
        _ => Action::None,
    }
}

fn handle_delete_confirm_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmDelete,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::CancelDelete,
        _ => Action::None,
    }
}

/// Execute an action, mutating app state
pub fn execute_action(app: &mut App, action: Action) {
    match action {
        Action::Quit => {
            // Stop all Docker containers before quitting
            if let Some(docker) = &mut app.docker {
                docker.stop_all();
            }
            app.should_quit = true;
        }

        Action::MoveUp => match app.input_mode {
            InputMode::TagList => {
                if app.tag_list_selected > 0 {
                    app.tag_list_selected -= 1;
                }
            }
            _ => app.move_up(),
        },

        Action::MoveDown => match app.input_mode {
            InputMode::TagList => {
                let max = app.current_tags.len().saturating_sub(1);
                if app.tag_list_selected < max {
                    app.tag_list_selected += 1;
                }
            }
            _ => app.move_down(),
        },

        Action::Select => {
            if let Some(run) = app.selected_run().cloned() {
                if app.load_run_detail(run.id).is_ok() {
                    app.navigate_to(View::RunDetail);
                    app.set_status(format!("Viewing: {}", run.name));
                } else {
                    app.set_status("Failed to load run details");
                }
            }
        }

        Action::Back => {
            app.go_back();
            app.set_status("Ready");
        }

        Action::GoToDashboard => {
            app.navigate_to(View::Dashboard);
            app.set_status("Ready");
        }

        Action::GoToCompare => {
            if app.compare_run_ids.is_empty() {
                app.set_status("No runs selected. Press Space to mark runs first.");
            } else if app.load_compare_data().is_ok() {
                app.navigate_to(View::Compare);
                app.set_status(format!("Comparing {} runs", app.compare_run_ids.len()));
            }
        }

        Action::GoToGpuMonitor => {
            app.navigate_to(View::GpuMonitor);
            app.set_status("GPU Monitor");
        }

        Action::NextTab => {
            app.selected_tab = (app.selected_tab + 1) % app.tab_titles.len();
        }

        Action::PrevTab => {
            app.selected_tab = if app.selected_tab == 0 {
                app.tab_titles.len() - 1
            } else {
                app.selected_tab - 1
            };
        }

        Action::CycleMetric => {
            app.cycle_metric();
            if let Some(name) = app.current_metric_names.get(app.selected_metric_index) {
                app.set_status(format!("Metric: {}", name));
            }
        }

        Action::CycleCompareMetric => {
            app.cycle_compare_metric();
            if let Some(name) = app.compare_metric_names.get(app.compare_selected_metric) {
                app.set_status(format!("Compare metric: {}", name));
            }
        }

        // ─── Search ──────────────────────────────────
        Action::EnterSearchMode => {
            app.input_mode = InputMode::Search;
            app.set_status("Search: type to filter");
        }

        Action::ExitSearchMode => {
            app.input_mode = InputMode::Normal;
            app.set_status("Ready");
        }

        Action::SearchInput(c) => {
            let mut query = app.search_query.clone();
            query.push(c);
            app.update_search(query);
        }

        Action::SearchBackspace => {
            let mut query = app.search_query.clone();
            query.pop();
            app.update_search(query);
        }

        Action::SearchClear => {
            app.update_search(String::new());
        }

        // ─── Delete ──────────────────────────────────
        Action::DeleteRun => {
            if app.selected_run().is_some() {
                app.input_mode = InputMode::DeleteConfirm;
            }
        }

        Action::ConfirmDelete => {
            if let Some(run) = app.selected_run().cloned() {
                if app.db.delete_run(run.id).is_ok() {
                    app.compare_run_ids.retain(|&id| id != run.id);
                    let _ = app.refresh_runs();
                    app.set_status(format!("Deleted: {}", run.name));
                }
            }
            app.input_mode = InputMode::Normal;
        }

        Action::CancelDelete => {
            app.input_mode = InputMode::Normal;
            app.set_status("Delete cancelled");
        }

        // ─── Tags ────────────────────────────────────
        Action::OpenTagList => {
            app.input_mode = InputMode::TagList;
            app.tag_list_selected = 0;
        }

        Action::OpenTagInput => {
            app.input_buffer.clear();
            app.input_mode = InputMode::TagInput;
        }

        Action::RemoveSelectedTag => {
            if let Some(run) = app.current_run.clone() {
                if let Some(tag) = app.current_tags.get(app.tag_list_selected) {
                    let tag = tag.clone();
                    if app.db.remove_tag(run.id, &tag).is_ok() {
                        app.current_tags.retain(|t| t != &tag);
                        if app.tag_list_selected > 0
                            && app.tag_list_selected >= app.current_tags.len()
                        {
                            app.tag_list_selected = app.current_tags.len().saturating_sub(1);
                        }
                        app.set_status(format!("Removed tag: {}", tag));
                    }
                }
            }
        }

        // ─── Notes ───────────────────────────────────
        Action::OpenNotesEditor => {
            app.input_buffer.clear();
            app.input_mode = InputMode::NotesInput;
        }

        // ─── Shared Input ────────────────────────────
        Action::InputChar(c) => {
            app.input_buffer.push(c);
        }

        Action::InputBackspace => {
            app.input_buffer.pop();
        }

        Action::InputConfirm => {
            let buffer = app.input_buffer.clone();
            match app.input_mode {
                InputMode::TagInput => {
                    if !buffer.is_empty() {
                        if let Some(run) = app.current_run.clone() {
                            if app.db.add_tag(run.id, &buffer).is_ok() {
                                app.current_tags.push(buffer.clone());
                                app.current_tags.sort();
                                app.set_status(format!("Added tag: {}", buffer));
                            }
                        }
                    }
                    app.input_buffer.clear();
                    app.input_mode = InputMode::TagList;
                }
                InputMode::NotesInput => {
                    if let Some(run) = app.current_run.clone() {
                        if app.db.update_run_notes(run.id, &buffer).is_ok() {
                            if let Some(r) = &mut app.current_run {
                                r.notes = buffer.clone();
                            }
                            app.set_status("Notes saved");
                        }
                    }
                    app.input_buffer.clear();
                    app.input_mode = InputMode::Normal;
                }
                _ => {
                    app.input_mode = InputMode::Normal;
                }
            }
        }

        Action::InputCancel => {
            app.input_buffer.clear();
            match app.input_mode {
                InputMode::TagInput => app.input_mode = InputMode::TagList,
                _ => app.input_mode = InputMode::Normal,
            }
            app.set_status("Cancelled");
        }

        // ─── Compare ────────────────────────────────
        Action::ToggleCompareSelection => {
            if let Some(run) = app.selected_run().cloned() {
                app.toggle_compare(run.id);
            }
        }

        // ─── Status Toggle ──────────────────────────
        Action::ToggleRunStatus => {
            if let Some(run) = app.current_run.clone() {
                use crate::models::RunStatus;
                let new_status = match run.status {
                    RunStatus::Running => RunStatus::Completed,
                    RunStatus::Completed => RunStatus::Stopped,
                    RunStatus::Stopped => RunStatus::Running,
                    RunStatus::Failed => RunStatus::Running,
                };
                if app.db.update_run_status(run.id, &new_status).is_ok() {
                    let _ = app.load_run_detail(run.id);
                    app.set_status(format!("Status -> {}", new_status));
                }
            }
        }

        // ─── Docker ─────────────────────────────────
        Action::StopContainer => {
            if let Some(run) = app.current_run.clone() {
                if let Some(docker) = &mut app.docker {
                    if docker.is_running(run.id) {
                        if docker.stop_container(run.id).is_ok() {
                            let _ = app.db.update_run_status(
                                run.id,
                                &crate::models::RunStatus::Stopped,
                            );
                            let _ = app.load_run_detail(run.id);
                            app.set_status("Container stopped");
                        }
                    } else {
                        app.set_status("No running container for this run");
                    }
                }
            }
        }

        Action::Refresh => {
            if app.refresh_runs().is_ok() {
                app.set_status("Refreshed");
            }
        }

        Action::ToggleHelp => {
            app.show_help = !app.show_help;
            if app.show_help {
                app.navigate_to(View::Help);
            } else {
                app.go_back();
            }
        }

        _ => {}
    }
}
