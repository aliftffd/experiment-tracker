use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph},
    Frame,
};

use crate::app::state::App;
use crate::gpu::{GpuHistory, GpuStats};
use crate::ui::{gpu_bar, status_bar};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // GPU bar
        Constraint::Length(8), // gauges (utilization, VRAM, temp, power)
        Constraint::Min(10),   // history chart
        Constraint::Length(8), // process list
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    gpu_bar::render(app.gpu_stats.as_ref(), frame, chunks[0]);
    render_gauges(app.gpu_stats.as_ref(), frame, chunks[1]);
    render_history_chart(&app.gpu_history, frame, chunks[2]);
    render_process_list(&app.gpu_processes, frame, chunks[3]);
    status_bar::render(app, frame, chunks[4]);
}

fn render_gauges(stats: Option<&GpuStats>, frame: &mut Frame, area: ratatui::layout::Rect) {
    let stats = match stats {
        Some(s) => s,
        None => {
            let msg = Paragraph::new("No GPU detected")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" GPU Stats ")
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
            frame.render_widget(msg, area);
            return;
        }
    };

    let inner = Layout::vertical([
        Constraint::Length(1), // GPU name
        Constraint::Length(2), // utilization gauge
        Constraint::Length(2), // VRAM gauge
        Constraint::Length(2), // temperature + power
    ])
    .split(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " {} — Driver {} ",
                stats.gpu_name, stats.driver_version
            ))
            .inner(area),
    );

    // Render the block border
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " {} — Driver {} ",
                stats.gpu_name, stats.driver_version
            ))
            .border_style(Style::default().fg(Color::DarkGray)),
        area,
    );

    // GPU Utilization gauge
    let util_gauge = Gauge::default()
        .block(Block::default().title(" Utilization "))
        .gauge_style(gauge_style(stats.utilization_percent as f32, 60.0, 85.0))
        .percent(stats.utilization_percent.min(100) as u16)
        .label(format!("{}%", stats.utilization_percent));
    frame.render_widget(util_gauge, inner[1]);

    // VRAM gauge
    let vram_pct = stats.vram_percent();
    let vram_gauge = Gauge::default()
        .block(Block::default().title(" VRAM "))
        .gauge_style(gauge_style(vram_pct, 60.0, 85.0))
        .ratio((vram_pct / 100.0).min(1.0) as f64)
        .label(format!(
            "{} / {} MB ({:.0}%)",
            stats.vram_used_mb, stats.vram_total_mb, vram_pct
        ));
    frame.render_widget(vram_gauge, inner[2]);

    // Temperature and power on the same line
    let temp_power = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner[3]);

    let temp_line = Line::from(vec![
        Span::styled("  Temp: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}°C", stats.temperature_celsius),
            Style::default().fg(temp_color(stats.temperature_celsius)),
        ),
        Span::styled(
            format!("  Clock: {} MHz", stats.clock_speed_mhz),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(temp_line), temp_power[0]);

    let power_line = Line::from(vec![
        Span::styled("  Power: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                "{:.0} / {:.0} W",
                stats.power_draw_watts, stats.power_limit_watts
            ),
            Style::default().fg(Color::DarkGray),
        ),
        if let Some(fan) = stats.fan_speed_percent {
            Span::styled(
                format!("  Fan: {}%", fan),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::styled("  Fan: N/A", Style::default().fg(Color::DarkGray))
        },
    ]);
    frame.render_widget(Paragraph::new(power_line), temp_power[1]);
}

fn render_history_chart(history: &GpuHistory, frame: &mut Frame, area: ratatui::layout::Rect) {
    if history.is_empty() {
        let empty = Paragraph::new("Collecting GPU data...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" History ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let util_data: Vec<(f64, f64)> = history
        .utilization_series()
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let vram_data: Vec<(f64, f64)> = history
        .vram_series()
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let temp_data: Vec<(f64, f64)> = history
        .temp_series()
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let x_max = history.len() as f64;

    let datasets = vec![
        Dataset::default()
            .name("Util %")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&util_data),
        Dataset::default()
            .name("VRAM %")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&vram_data),
        Dataset::default()
            .name("Temp °C")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&temp_data),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" History (last 5 min) — Cyan:Util  Magenta:VRAM  Yellow:Temp ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, x_max]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, 100.0])
                .labels::<Vec<Line>>(vec!["0".into(), "50".into(), "100".into()]),
        );

    frame.render_widget(chart, area);
}

fn render_process_list(
    processes: &[crate::gpu::GpuProcess],
    frame: &mut Frame,
    area: ratatui::layout::Rect,
) {
    let mut lines: Vec<Line> = vec![Line::from(vec![Span::styled(
        format!("  {:<8} {:<40} {:>10}", "PID", "Process", "VRAM"),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )])];

    if processes.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No GPU processes running",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for proc in processes {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<8}", proc.pid),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<40}", truncate_name(&proc.name, 38)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>8} MB", proc.vram_used_mb),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
        }
    }

    let list = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" GPU Processes ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(list, area);
}

fn gauge_style(value: f32, warn: f32, critical: f32) -> Style {
    let color = if value > critical {
        Color::Red
    } else if value > warn {
        Color::Yellow
    } else {
        Color::Green
    };
    Style::default().fg(color)
}

fn temp_color(temp: u32) -> Color {
    if temp > 85 {
        Color::Red
    } else if temp > 70 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn truncate_name(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
