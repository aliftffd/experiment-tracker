use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::state::App;
use crate::ui::status_bar;
use crate::utils::color::{chart_color, status_color};
use crate::utils::time::relative_time;

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(5), // run info header
        Constraint::Min(12), // chart
        Constraint::Length(8), // latest metrics table
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    render_header(app, frame, chunks[0]);
    render_chart(app, frame, chunks[1]);
    render_metrics_summary(app, frame, chunks[2]);
    status_bar::render(app, frame, chunks[3]);
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

    let lines = vec![
        Line::from(vec![
            Span::styled(" Name: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&run.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(run.status.symbol(), status_style),
            Span::styled(format!("{}", run.status), status_style),
        ]),

        Line::from(vec![
            Span::styled(" Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&run.log_path, Style::default().fg(Color::Gray)),
        ]),

        Line::from(vec![
            Span::styled(" Tags: ", Style::default().fg(Color::DarkGray)),
            Span::styled(tags_str, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                format!("Created: {}", relative_time(&run.created_at)),
                Style::default().fg(Color::DarkGray),
                ),
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

    // collect data points for the selected metric
    let data_points: Vec<(f64, f64)> = app
        .current_metrics
        .iter()
        .filter(|m| &m.name == metric_name)
        .map(|m| (m.x_value(), m.value))
        .collect();

    if data_points.is_empty() {
        let empty = Paragraph::new(format!("No data for metric: {}", metric_name))
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

    // Calculate bounds
    let x_min = data_points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let x_max = data_points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let y_min = data_points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let y_max = data_points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);

    // add padding to y-axis
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
            " {} [{}/{}] (Tab to cycle) ",
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
                .labels(vec![
                    Line::from(format!("{:.0}", x_min)),
                    Line::from(format!("{:.0}", (x_min + x_max) / 2.0)),
                    Line::from(format!("{:.0}", x_max)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(metric_name.as_str())
                .style(Style::default().fg(Color::DarkGray))
                .bounds([y_min, y_max])
                .labels(vec![
                    Line::from(format!("{:.4}", y_min)),
                    Line::from(format!("{:.4}", (y_min + y_max) / 2.0)),
                    Line::from(format!("{:.4}", y_max)),
                ]),
        );
    frame.render_widget(chart, area);
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
                    .border_style(Style::default().fg(Color::DarkGray))
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
                    format!(" {} ", if i == app.selected_metric_index { "▶" } else { " " }),
                    Style::default().fg(Color::Cyan),
                    ),
                Span::styled(
                    format!("{:<20}", name),
                    Style::default().fg(chart_color(i)),
                    ),
                Span::styled(
                    format!("{:.6}", value),
                    Style::default().fg(Color::White),
                    ),
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
