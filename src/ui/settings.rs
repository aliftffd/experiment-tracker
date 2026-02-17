use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::App;
use crate::platform;
use super::{gpu_bar, status_bar};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(1),  // GPU bar
        Constraint::Min(0),    // Settings content
        Constraint::Length(1), // Status bar
    ])
    .split(frame.area());

    gpu_bar::render(app.gpu_stats.as_ref(), frame, chunks[0]);
    render_settings_content(app, frame, chunks[1]);
    status_bar::render(app, frame, chunks[2]);
}

fn render_settings_content(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let use_unicode = platform::supports_unicode();

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White);
    let ok_style = Style::default().fg(Color::Green);
    let err_style = Style::default().fg(Color::Red);
    let separator = if use_unicode { "────────────────────" } else { "--------------------" };

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    // Watch Directories
    lines.push(Line::from(Span::styled("  Watch Directories", header_style)));
    lines.push(Line::from(Span::styled(
        format!("  {}", separator),
        Style::default().fg(Color::DarkGray),
    )));
    for (i, dir) in app.config.general.watch_dirs.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(format!("  {}. ", i + 1), label_style),
            Span::styled(dir.clone(), value_style),
        ]));
    }

    lines.push(Line::from(""));

    // Database
    lines.push(Line::from(Span::styled("  Database", header_style)));
    lines.push(Line::from(Span::styled(
        format!("  {}", separator),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Path: ", label_style),
        Span::styled(app.config.general.db_path.clone(), value_style),
    ]));
    let db_size = app.db_file_size();
    let run_count = app.runs.len();
    lines.push(Line::from(vec![
        Span::styled(format!("  Runs: {}", run_count), value_style),
        Span::styled("  |  ", label_style),
        Span::styled(format!("Size: {}", db_size), value_style),
    ]));

    lines.push(Line::from(""));

    // Docker
    lines.push(Line::from(Span::styled("  Docker", header_style)));
    lines.push(Line::from(Span::styled(
        format!("  {}", separator),
        Style::default().fg(Color::DarkGray),
    )));
    if let Some(info) = &app.docker_info {
        let check = if use_unicode { "\u{2713}" } else { "+" };
        let cross = if use_unicode { "\u{2717}" } else { "x" };

        if info.running {
            lines.push(Line::from(vec![
                Span::styled("  Status: ", label_style),
                Span::styled(format!("{} Running (v{})", check, info.version), ok_style),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  Status: ", label_style),
                Span::styled(format!("{} Not running", cross), err_style),
            ]));
        }

        let gpu_str = if info.gpu_support {
            Span::styled(format!("{} nvidia-container-toolkit", check), ok_style)
        } else {
            Span::styled(format!("{} not available", cross), err_style)
        };
        lines.push(Line::from(vec![
            Span::styled("  GPU Support: ", label_style),
            gpu_str,
        ]));
    } else {
        let cross = if use_unicode { "\u{2717}" } else { "x" };
        lines.push(Line::from(vec![
            Span::styled("  Status: ", label_style),
            Span::styled(format!("{} Not found", cross), err_style),
        ]));
    }

    if let Some(docker_cfg) = &app.config.docker {
        lines.push(Line::from(vec![
            Span::styled("  Default Image: ", label_style),
            Span::styled(docker_cfg.default_image.clone(), value_style),
        ]));
    }

    lines.push(Line::from(""));

    // GPU
    lines.push(Line::from(Span::styled("  GPU", header_style)));
    lines.push(Line::from(Span::styled(
        format!("  {}", separator),
        Style::default().fg(Color::DarkGray),
    )));
    if let Some(stats) = &app.gpu_stats {
        lines.push(Line::from(vec![
            Span::styled("  Device: ", label_style),
            Span::styled(stats.gpu_name.clone(), value_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Driver: ", label_style),
            Span::styled(stats.driver_version.clone(), value_style),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  Device: ", label_style),
            Span::styled("Not detected", Style::default().fg(Color::DarkGray)),
        ]));
    }

    if let Some(gpu_cfg) = &app.config.gpu {
        lines.push(Line::from(vec![
            Span::styled("  Poll Interval: ", label_style),
            Span::styled(format!("{}s", gpu_cfg.poll_interval_secs), value_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  Temp Warning: {}{}C", gpu_cfg.temp_warning, if use_unicode { "\u{00B0}" } else { "" }),
                value_style,
            ),
            Span::styled("  |  ", label_style),
            Span::styled(
                format!("Critical: {}{}C", gpu_cfg.temp_critical, if use_unicode { "\u{00B0}" } else { "" }),
                value_style,
            ),
        ]));
    }

    lines.push(Line::from(""));

    // Config File
    lines.push(Line::from(Span::styled("  Config File", header_style)));
    lines.push(Line::from(Span::styled(
        format!("  {}", separator),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Location: ", label_style),
        Span::styled(
            "~/.config/experiment-tracker/config.toml",
            value_style,
        ),
    ]));

    lines.push(Line::from(""));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Settings ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}
