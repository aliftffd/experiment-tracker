use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Row,
    Frame,
};

use crate::app::state::{App, InputMode};
use crate::ui::components::{input, table};
use crate::ui::{gpu_bar, status_bar};
use crate::utils::{color::status_color, time::relative_time};

pub fn render(app: &mut App, frame: &mut Frame) {
    let show_search = app.input_mode == InputMode::Search || !app.search_query.is_empty();

    let mut constraints = vec![Constraint::Length(1)]; // GPU bar

    if show_search {
        constraints.push(Constraint::Length(3)); // search bar
    }
    constraints.push(Constraint::Min(0)); // table
    constraints.push(Constraint::Length(1)); // status bar

    let chunks = Layout::vertical(constraints).split(frame.area());

    let mut chunk_idx = 0;

    // GPU bar
    gpu_bar::render(app.gpu_stats.as_ref(), frame, chunks[chunk_idx]);
    chunk_idx += 1;

    // Search bar (conditional)
    if show_search {
        let search_widget = input::text_input(
            "Search",
            &app.search_query,
            app.input_mode == InputMode::Search,
        );
        frame.render_widget(search_widget, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    let table_area = chunks[chunk_idx];
    let status_area = chunks[chunk_idx + 1];

    // Build table rows
    let runs = app.visible_runs().clone();
    let rows: Vec<Row> = runs
        .iter()
        .map(|run| {
            let status_style = Style::default().fg(status_color(&run.status));

            // Compare marker
            let compare_marker = if app.is_selected_for_compare(run.id) {
                Span::styled("* ", Style::default().fg(Color::Yellow))
            } else {
                Span::styled("  ", Style::default())
            };

            // Docker indicator
            let docker_indicator = if app
                .docker
                .as_ref()
                .map(|d| d.is_running(run.id))
                .unwrap_or(false)
            {
                Span::styled("D", Style::default().fg(Color::Cyan))
            } else {
                Span::raw(" ")
            };

            Row::new(vec![
                Line::from(compare_marker),
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
                Line::from(docker_indicator),
            ])
        })
        .collect();

    let run_count = runs.len();
    let compare_count = app.compare_run_ids.len();
    let title = if compare_count > 0 {
        format!(
            "Experiments ({}) -- {} selected for compare",
            run_count, compare_count
        )
    } else {
        format!("Experiments ({})", run_count)
    };

    let widths = vec![
        Constraint::Length(3),  // compare marker
        Constraint::Length(3),  // status icon
        Constraint::Min(20),   // name
        Constraint::Length(12), // status text
        Constraint::Length(10), // updated
        Constraint::Length(4),  // docker indicator
    ];

    let headers = vec!["", "", "Name", "Status", "Updated", ""];

    let (table_widget, state) =
        table::styled_table(&title, headers, rows, widths, Some(app.selected_run_index));

    if let Some(mut table_state) = state {
        frame.render_stateful_widget(table_widget, table_area, &mut table_state);
    } else {
        frame.render_widget(table_widget, table_area);
    }

    // Empty state
    if run_count == 0 {
        let empty_msg = if app.search_query.is_empty() {
            "No experiments found. Place .jsonl or .csv files in your watch directory, or press R to run an experiment."
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

    status_bar::render(app, frame, status_area);
}
