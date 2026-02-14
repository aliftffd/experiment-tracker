use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Row,
    Frame,
};

use crate::app::state::{App, InputMode};
use crate::ui::components::{input, table};
use crate::ui::status_bar;
use crate::utils::{color::status_color, time::relative_time};

pub fn render(app: &mut App, frame: &mut Frame) {
    let show_search = app.input_mode == InputMode::Search || !app.search_query.is_empty();

    // Layout
    let constraints = if show_search {
        vec![
            Constraint::Length(3), // search bar
            Constraint::Min(0),    // table
            Constraint::Length(1), // status bar
        ]
    } else {
        vec![
            Constraint::Min(0),    // table
            Constraint::Length(1), // status bar
        ]
    };

    let chunks = Layout::vertical(constraints).split(frame.area());

    let (table_area, status_area) = if show_search {
        // Render search bar
        let search_widget = input::text_input(
            "Search",
            &app.search_query,
            app.input_mode == InputMode::Search,
        );
        frame.render_widget(search_widget, chunks[0]);
        (chunks[1], chunks[2])
    } else {
        (chunks[0], chunks[1])
    };

    // Build table rows
    let runs = app.visible_runs().clone();
    let rows: Vec<Row> = runs
        .iter()
        .map(|run| {
            let status_style = Style::default().fg(status_color(&run.status));

            Row::new(vec![
                Line::from(Span::styled(run.status.symbol(), status_style)),
                Line::from(Span::styled(
                    run.name.clone(),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(run.status.to_string(), status_style)),
                Line::from(Span::styled(
                    relative_time(&run.updated_at),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    truncate(&run.log_path, 30),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let run_count = runs.len();
    let title = format!("Experiments ({})", run_count);

    let widths = vec![
        Constraint::Length(3),  // status icon
        Constraint::Min(20),    // name
        Constraint::Length(12), // status text
        Constraint::Length(10), // updated
        Constraint::Length(32), // log path
    ];

    let headers = vec!["", "Name", "Status", "Updated", "Path"];

    let (table_widget, state) =
        table::styled_table(&title, headers, rows, widths, Some(app.selected_run_index));

    if let Some(mut table_state) = state {
        frame.render_stateful_widget(table_widget, table_area, &mut table_state);
    } else {
        frame.render_widget(table_widget, table_area);
    }

    // Render empty state if no runs
    if run_count == 0 {
        let empty_msg = if app.search_query.is_empty() {
            "No experiments found. Logs will appear when detected in watch directories."
        } else {
            "No runs match your search."
        };

        let empty = ratatui::widgets::Paragraph::new(empty_msg)
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )
            .alignment(ratatui::layout::Alignment::Center);

        let inner = ratatui::layout::Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Length(1),
            Constraint::Percentage(40),
        ])
        .split(table_area);
        frame.render_widget(empty, inner[1]);
    }

    // Status bar
    status_bar::render(app, frame, status_area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("…{}", &s[s.len() - max_len + 1..])
    }
}
