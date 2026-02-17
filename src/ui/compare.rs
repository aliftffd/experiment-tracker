use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Row, Table},
    Frame,
};

use crate::app::state::App;
use crate::ui::{gpu_bar, status_bar};
use crate::utils::color::chart_color;

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(1),  // GPU bar
        Constraint::Min(14),    // overlaid chart
        Constraint::Length(10), // comparison table
        Constraint::Length(1),  // status bar
    ])
    .split(frame.area());

    gpu_bar::render(app.gpu_stats.as_ref(), frame, chunks[0]);
    render_compare_chart(app, frame, chunks[1]);
    render_compare_table(app, frame, chunks[2]);
    status_bar::render(app, frame, chunks[3]);
}

fn render_compare_chart(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.compare_data.is_empty() {
        let empty = Paragraph::new("No runs selected for comparison. Press Space on dashboard to mark runs, then C to compare.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Compare ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let metric_name = app
        .compare_metric_names
        .get(app.compare_selected_metric)
        .cloned()
        .unwrap_or_else(|| "loss".to_string());

    // Build datasets for each run
    let mut datasets = Vec::new();
    let mut all_points: Vec<Vec<(f64, f64)>> = Vec::new();

    for (i, (run_name, metrics)) in app.compare_data.iter().enumerate() {
        let points: Vec<(f64, f64)> = metrics
            .iter()
            .filter(|m| m.name == metric_name)
            .map(|m| (m.x_value(), m.value))
            .collect();

        all_points.push(points);

        // We need the data to live long enough for the Chart widget
        // So we collect names for the legend
        let _name = format!("{} ({})", run_name, i + 1);
    }

    // Calculate global bounds across all runs
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for points in &all_points {
        for (x, y) in points {
            x_min = x_min.min(*x);
            x_max = x_max.max(*x);
            y_min = y_min.min(*y);
            y_max = y_max.max(*y);
        }
    }

    if x_min == f64::INFINITY {
        // No data points at all
        let empty = Paragraph::new(format!("No '{}' data in selected runs", metric_name))
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Compare ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let y_padding = (y_max - y_min) * 0.1;
    let y_min = y_min - y_padding;
    let y_max = y_max + y_padding;

    // Build datasets
    for (i, points) in all_points.iter().enumerate() {
        if points.is_empty() {
            continue;
        }
        let run_name = &app.compare_data[i].0;
        datasets.push(
            Dataset::default()
                .name(run_name.as_str())
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(chart_color(i)))
                .data(points),
        );
    }

    let title = if app.compare_metric_names.len() > 1 {
        format!(
            " {} [{}/{}] (Tab to cycle) ",
            metric_name,
            app.compare_selected_metric + 1,
            app.compare_metric_names.len()
        )
    } else {
        format!(" {} ", metric_name)
    };

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
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

fn render_compare_table(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.compare_data.is_empty() {
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(" Results ")
                .border_style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    // Collect all unique metric names across compared runs
    let mut all_metric_names: Vec<String> = Vec::new();
    for (_, metrics) in &app.compare_data {
        for m in metrics {
            if !all_metric_names.contains(&m.name) {
                all_metric_names.push(m.name.clone());
            }
        }
    }

    // Build header: "Metric | Run A | Run B | ..."
    let run_names: Vec<&str> = app.compare_data.iter().map(|(n, _)| n.as_str()).collect();
    let mut header_vec: Vec<&str> = vec!["Metric"];
    for name in &run_names {
        header_vec.push(name);
    }

    // Build rows: one per metric, showing latest value from each run
    let rows: Vec<Row> = all_metric_names
        .iter()
        .map(|metric_name| {
            let mut cells = vec![Line::from(Span::styled(
                metric_name.clone(),
                Style::default().fg(Color::Cyan),
            ))];

            // Find the latest value for this metric in each run
            let values: Vec<Option<f64>> = app
                .compare_data
                .iter()
                .map(|(_, metrics)| {
                    metrics
                        .iter()
                        .filter(|m| &m.name == metric_name)
                        .last()
                        .map(|m| m.value)
                })
                .collect();

            // Find the best value (min for loss-like, max for accuracy-like)
            let is_lower_better = metric_name.contains("loss") || metric_name.contains("error");
            let best = if is_lower_better {
                values
                    .iter()
                    .filter_map(|v| *v)
                    .fold(f64::INFINITY, f64::min)
            } else {
                values
                    .iter()
                    .filter_map(|v| *v)
                    .fold(f64::NEG_INFINITY, f64::max)
            };

            for val in &values {
                match val {
                    Some(v) => {
                        let is_best = (*v - best).abs() < 1e-10;
                        let style = if is_best {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        cells.push(Line::from(Span::styled(format!("{:.6}", v), style)));
                    }
                    None => {
                        cells.push(Line::from(Span::styled(
                            "—",
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }
            }

            Row::new(cells)
        })
        .collect();

    // Column widths
    let mut widths = vec![Constraint::Length(20)]; // metric name
    for _ in &app.compare_data {
        widths.push(Constraint::Min(15)); // each run
    }

    let header = Row::new(header_vec.iter().map(|h| {
        Line::from(Span::styled(
            h.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
    }))
    .bottom_margin(1);

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Final Metrics (best highlighted) ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(table, area);
}
