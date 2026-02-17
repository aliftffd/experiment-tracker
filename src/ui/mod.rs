pub mod ascii_art;
pub mod compare;
pub mod components;
pub mod dashboard;
pub mod gpu_bar;
pub mod gpu_screen;
pub mod menu;
pub mod popups;
pub mod run_detail;
pub mod run_dialog;
pub mod settings;
pub mod splash;
pub mod status_bar;

use ratatui::Frame;

use crate::app::state::{App, InputMode, View};

pub fn render(app: &mut App, frame: &mut Frame) {
    match app.current_view {
        View::Splash => splash::render(app, frame),
        View::Menu => menu::render(app, frame),
        View::Dashboard => dashboard::render(app, frame),
        View::RunDetail => run_detail::render(app, frame),
        View::Compare => compare::render(app, frame),
        View::GpuMonitor => gpu_screen::render(app, frame),
        View::Settings => settings::render(app, frame),
        View::Help => render_help(app, frame),
    }

    render_popup_overlay(app, frame);
}

fn render_popup_overlay(app: &App, frame: &mut Frame) {
    let area = frame.area();

    match app.input_mode {
        InputMode::DeleteConfirm => {
            if let Some(run) = app.selected_run() {
                popups::render_delete_confirm(&run.name, frame, area);
            }
        }
        InputMode::TagList => {
            popups::render_tag_list(&app.current_tags, app.tag_list_selected, frame, area);
        }
        InputMode::TagInput => {
            popups::render_input_popup(
                "Add Tag",
                "Enter tag name:",
                &app.input_buffer,
                frame,
                area,
            );
        }
        InputMode::NotesInput => {
            let current_notes = app
                .current_run
                .as_ref()
                .map(|r| r.notes.as_str())
                .unwrap_or("");
            popups::render_notes_editor(current_notes, &app.input_buffer, frame, area);
        }
        InputMode::RunDialog => {
            if let Some(dialog) = &app.run_dialog {
                run_dialog::render(dialog, frame, area);
            }
        }
        _ => {}
    }
}

fn render_help(app: &mut App, frame: &mut Frame) {
    use ratatui::layout::{Constraint, Layout};

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

    gpu_bar::render(app.gpu_stats.as_ref(), frame, chunks[0]);

    let help_text = vec![
        "",
        "  ╔══════════════════════════════════════════════════╗",
        "  ║         Experiment Tracker — Help                 ║",
        "  ╠══════════════════════════════════════════════════╣",
        "  ║                                                  ║",
        "  ║  Dashboard                                       ║",
        "  ║  ─────────                                       ║",
        "  ║  j/k ↑/↓    Navigate runs                        ║",
        "  ║  Enter/l     Open run detail                      ║",
        "  ║  Space       Mark/unmark for comparison           ║",
        "  ║  /           Search runs                          ║",
        "  ║  d           Delete run (with confirmation)       ║",
        "  ║  c           Compare marked runs                  ║",
        "  ║  g           GPU monitor                          ║",
        "  ║  R           Run experiment (Docker)              ║",
        "  ║  r           Refresh                              ║",
        "  ║  Esc         Back to menu                         ║",
        "  ║                                                  ║",
        "  ║  Run Detail                                      ║",
        "  ║  ──────────                                      ║",
        "  ║  Esc/h       Back to dashboard                    ║",
        "  ║  Tab         Cycle sub-view (Chart/Params/Logs)   ║",
        "  ║  j/k         Cycle metrics (in chart view)        ║",
        "  ║  s           Toggle run status                    ║",
        "  ║  t           Manage tags                          ║",
        "  ║  n           Edit notes                           ║",
        "  ║  K           Stop Docker container                ║",
        "  ║  m           Export as Markdown                    ║",
        "  ║  e           Export as CSV                         ║",
        "  ║  x           Export as LaTeX                       ║",
        "  ║                                                  ║",
        "  ║  Compare View                                    ║",
        "  ║  ────────────                                    ║",
        "  ║  Tab         Cycle metrics                        ║",
        "  ║  m/e/x       Export comparison (md/csv/tex)       ║",
        "  ║  Esc         Back to menu                         ║",
        "  ║                                                  ║",
        "  ║  General                                         ║",
        "  ║  ───────                                         ║",
        "  ║  ?           Toggle help                          ║",
        "  ║  q           Quit                                 ║",
        "  ║  Ctrl+C      Force quit                           ║",
        "  ║                                                  ║",
        "  ╚══════════════════════════════════════════════════╝",
    ];

    let text = help_text.join("\n");
    let paragraph = ratatui::widgets::Paragraph::new(text)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));

    frame.render_widget(paragraph, chunks[1]);
    status_bar::render(app, frame, chunks[2]);
}
