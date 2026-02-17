use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Wrap},
    Frame,
};

use crate::app::state::{App, DetailSubView};
use crate::ui::{gpu_bar, status_bar};
use crate::utils::color::{chart_color, status_color};
use crate::utils::time::relative_time;

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // GPU bar
        Constraint::Length(5), // run info header
        Constraint::Length(1), // sub-view tabs
        Constraint::Min(10),   // active sub-view
        Constraint::Length(6), // latest metrics
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    gpu_bar::render(app.gpu_stats.as_ref(), frame, chunks[0]);
    render_header(app, frame, chunks[1]);
    render_sub_view_tabs(app, frame, chunks[2]);

    match app.detail_sub_view {
        DetailSubView::Chart => render_chart(app, frame, chunks[3]),
        DetailSubView::Hyperparams => render_hyperparams(app, frame, chunks[3]),
        DetailSubView::Logs => render_logs(app, frame, chunks[3]),
    }

    render_metrics_summary(app, frame, chunks[4]);
    status_bar::render(app, frame, chunks[5]);
}

fn render_header(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let run = match &app.current_run {
        Some(r) => r,
        None => return,
    };

    let status_style = Style::default().fg(status_color(&run.status));

    let tags_str = if app.current_tags.is_empty() {
        "none".to_string()
    } else {
        app.current_tags.join(", ")
    };

    // Docker indicator
    let docker_status = if app
        .docker
        .as_ref()
        .map(|d| d.is_running(run.id))
        .unwrap_or(false)
    {
        " 🐳 container running"
    } else {
        ""
    };

    let notes_preview = if run.notes.is_empty() {
        String::new()
    } else {
        let preview = if run.notes.len() > 50 {
            format!("{}…", &run.notes[..50])
        } else {
            run.notes.clone()
        };
        format!("  Notes: {}", preview)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(" Name: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &run.name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(run.status.symbol(), status_style),
            Span::styled(format!(" {}", run.status), status_style),
            Span::styled(docker_status, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(" Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&run.log_path, Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled(" Tags: ", Style::default().fg(Color::DarkGray)),
            Span::styled(tags_str, Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("  Created: {}", relative_time(&run.created_at)),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(notes_preview, Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let header = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Run Detail ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(header, area);
}

fn render_sub_view_tabs(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let tabs = vec![
        ("Chart", DetailSubView::Chart),
        ("Hyperparams", DetailSubView::Hyperparams),
        ("Logs", DetailSubView::Logs),
    ];

    let spans: Vec<Span> = tabs
        .iter()
        .map(|(label, view)| {
            if *view == app.detail_sub_view {
                Span::styled(
                    format!(" {} ", label),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                )
            } else {
                Span::styled(format!(" {} ", label), Style::default().fg(Color::DarkGray))
            }
        })
        .collect();

    let mut all_spans = vec![Span::raw(" ")];
    for (i, span) in spans.into_iter().enumerate() {
        all_spans.push(span);
        if i < tabs.len() - 1 {
            all_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }
    }
    all_spans.push(Span::styled(
        "  (Tab to switch)",
        Style::default().fg(Color::DarkGray),
    ));

    let line = Line::from(all_spans);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_chart(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.current_metric_names.is_empty() {
        let empty = Paragraph::new("No metrics recorded yet")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Chart ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let metric_name = &app.current_metric_names[app.selected_metric_index];

    let data_points: Vec<(f64, f64)> = app
        .current_metrics
        .iter()
        .filter(|m| &m.name == metric_name)
        .map(|m| (m.x_value(), m.value))
        .collect();

    if data_points.is_empty() {
        let empty = Paragraph::new(format!("No data for: {}", metric_name))
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", metric_name))
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let x_min = data_points
        .iter()
        .map(|p| p.0)
        .fold(f64::INFINITY, f64::min);
    let x_max = data_points
        .iter()
        .map(|p| p.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = data_points
        .iter()
        .map(|p| p.1)
        .fold(f64::INFINITY, f64::min);
    let y_max = data_points
        .iter()
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_padding = (y_max - y_min) * 0.1;
    let y_min = y_min - y_padding;
    let y_max = y_max + y_padding;

    let dataset = Dataset::default()
        .name(metric_name.as_str())
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(chart_color(app.selected_metric_index)))
        .data(&data_points);

    let tab_hint = if app.current_metric_names.len() > 1 {
        format!(
            " {} [{}/{}] (j/k to cycle) ",
            metric_name,
            app.selected_metric_index + 1,
            app.current_metric_names.len()
        )
    } else {
        format!(" {} ", metric_name)
    };

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(tab_hint)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_axis(
            Axis::default()
                .title("Step")
                .style(Style::default().fg(Color::DarkGray))
                .bounds([x_min, x_max])
                .labels::<Vec<Line>>(vec![
                    format!("{:.0}", x_min).into(),
                    format!("{:.0}", x_max).into(),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(metric_name.as_str())
                .style(Style::default().fg(Color::DarkGray))
                .bounds([y_min, y_max])
                .labels::<Vec<Line>>(vec![
                    format!("{:.4}", y_min).into(),
                    format!("{:.4}", (y_min + y_max) / 2.0).into(),
                    format!("{:.4}", y_max).into(),
                ]),
        );

    frame.render_widget(chart, area);
}

fn render_hyperparams(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.current_hyperparams.is_empty() {
        let empty = Paragraph::new("No hyperparameters recorded.\n\nAdd a hyperparams line to your log file:\n{\"hyperparams\": {\"lr\": 0.001, \"batch_size\": 64}}")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Hyperparameters ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let mut lines: Vec<Line> = vec![
        Line::from(vec![Span::styled(
            format!("  {:<30} {}", "Parameter", "Value"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(Span::styled(
            format!("  {}", "─".repeat(50)),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    for hp in &app.current_hyperparams {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<30}", hp.key),
                Style::default().fg(Color::White),
            ),
            Span::styled(&hp.value, Style::default().fg(Color::Yellow)),
        ]));
    }

    let params = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Hyperparameters ({}) ",
                app.current_hyperparams.len()
            ))
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(params, area);
}

fn render_logs(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let is_running = app
        .current_run
        .as_ref()
        .map(|r| {
            app.docker
                .as_ref()
                .map(|d| d.is_running(r.id))
                .unwrap_or(false)
        })
        .unwrap_or(false);

    let content = if app.container_logs.is_empty() {
        if is_running {
            "Waiting for container output...".to_string()
        } else {
            "No active Docker container for this run.\n\nLogs appear here when a container is running.\nPress R on the dashboard to launch an experiment.".to_string()
        }
    } else {
        app.container_logs.clone()
    };

    let title = if is_running {
        " Container Logs (live) "
    } else {
        " Container Logs "
    };

    let style = if app.container_logs.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let logs = Paragraph::new(content)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(if is_running {
                    Color::Green
                } else {
                    Color::DarkGray
                })),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(logs, area);
}

fn render_metrics_summary(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let latest = &app.current_latest_metrics;

    if latest.is_empty() {
        let empty = Paragraph::new("No metrics")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Latest Metrics ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let lines: Vec<Line> = latest
        .iter()
        .enumerate()
        .map(|(i, (name, value))| {
            Line::from(vec![
                Span::styled(
                    format!(
                        "  {} ",
                        if i == app.selected_metric_index {
                            "▶"
                        } else {
                            " "
                        }
                    ),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(format!("{:<20}", name), Style::default().fg(chart_color(i))),
                Span::styled(format!("{:.6}", value), Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let summary = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Latest Metrics ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(summary, area);
}

