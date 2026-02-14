pub mod components;
pub mod dashboard;
pub mod run_detail;
pub mod status_bar;

use ratatui::Frame;

use crate::app::state::{App, View};

/// Main render function - routes to the appropriate view
pub fn render(app: &mut App, frame: &mut Frame) {
    match app.current_view {
        View::Dashboard => dashboard::render(app, frame),
        View::RunDetail => run_detail::render(app, frame),
        View::Compare => dashboard::render(app, frame), // placeholder until Day 3
        View::Help => render_help(app, frame),
    }
}

fn render_help(app: &mut App, frame: &mut Frame) {
    use ratatui::layout::{Constraint, Layout};

    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.area());

    let help_text = vec![
        "",
        "  ╔══════════════════════════════════════════╗",
        "  ║       Experiment Tracker - Help           ║",
        "  ╠══════════════════════════════════════════╣",
        "  ║                                          ║",
        "  ║  Navigation                              ║",
        "  ║  ─────────                               ║",
        "  ║  j/↓       Move down                     ║",
        "  ║  k/↑       Move up                       ║",
        "  ║  Enter/l   Select / Open detail           ║",
        "  ║  Esc/h     Go back                       ║",
        "  ║  Tab       Next tab / Cycle metric        ║",
        "  ║                                          ║",
        "  ║  Actions                                 ║",
        "  ║  ───────                                 ║",
        "  ║  /         Search runs                    ║",
        "  ║  d         Delete run                     ║",
        "  ║  s         Toggle status (in detail)      ║",
        "  ║  t         Add tag (in detail)            ║",
        "  ║  c         Compare mode                   ║",
        "  ║  r         Refresh                        ║",
        "  ║  m         Export markdown                 ║",
        "  ║  e         Export CSV                      ║",
        "  ║                                          ║",
        "  ║  General                                 ║",
        "  ║  ───────                                 ║",
        "  ║  ?         Toggle this help               ║",
        "  ║  q         Quit                           ║",
        "  ║  Ctrl+C    Force quit                     ║",
        "  ║                                          ║",
        "  ╚══════════════════════════════════════════╝",
        "",
    ];

    let text = help_text.join("\n");
    let paragraph = ratatui::widgets::Paragraph::new(text)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));

    frame.render_widget(paragraph, chunks[0]);
    status_bar::render(app, frame, chunks[1]);
}
