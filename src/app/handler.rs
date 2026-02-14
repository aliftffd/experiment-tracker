use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::actions::Action;
use crate::app::state::{App, InputMode, View};

/// map a key event to an action based on current state
pub fn handle_key_event(app: &App, key: KeyEvent) -> Action {
    // Global keybinding (always active)
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
        KeyCode::Char('q') if app.input_mode == InputMode::Normal => return Action::Quit,
        _ => {}
    }

    // Mode-specific keybinding
    match app.input_mode {
        InputMode::Search => handle_search_keys(key),
        InputMode::Normal => match app.current_view {
            View::Dashboard => handle_dashboard_keys(key),
            View::RunDetail => handle_run_detail_keys(key),
            View::Compare => handle_compare_keys(key),
            View::Help => handle_help_keys(key),
        },

        InputMode::TagInput => handle_tag_input_keys(key),
    }
}

fn handle_dashboard_keys(key: KeyEvent) -> Action {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
        KeyCode::Enter | KeyCode::Char('l') => Action::Select,
        KeyCode::Tab => Action::NextTab,
        KeyCode::BackTab => Action::PrevTab,

        // Actions
        KeyCode::Char('/') => Action::EnterSearchMode,
        KeyCode::Char('d') => Action::DeleteRun,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('c') => Action::GoToCompare,

        _ => Action::None,
    }
}

fn handle_run_detail_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Backspace => Action::Back,
        KeyCode::Tab => Action::CycleMetric,
        KeyCode::Char('s') => Action::ToggleRunStatus,
        KeyCode::Char('?') => Action::ToggleHelp,
        _ => Action::None,
    }
}

fn handle_compare_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => Action::Back,
        KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
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

fn handle_tag_input_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::ExitSearchMode,
        KeyCode::Enter => Action::ExitSearchMode,
        KeyCode::Backspace => Action::SearchBackspace,
        KeyCode::Char(c) => Action::SearchInput(c),
        _ => Action::None,
    }
}

/// Execute an action, mutating app state
pub fn execute_action(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.should_quit = true,
        Action::MoveUp => app.move_up(),
        Action::MoveDown => app.move_down(),

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
            app.navigate_to(View::Compare);
            app.set_status("Compare mode");
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

        Action::EnterSearchMode => {
            app.input_mode = InputMode::Search;
            app.set_status("Search: type to filter runs");
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

        Action::DeleteRun => {
            if let Some(run) = app.selected_run().cloned() {
                if app.db.delete_run(run.id).is_ok() {
                    let _ = app.refresh_runs();
                    app.set_status(format!("Deleted: {}", run.name));
                }
            }
        }

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
                    app.set_status(format!("Status → {}", new_status));
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
