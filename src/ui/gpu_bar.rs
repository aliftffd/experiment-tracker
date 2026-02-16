use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::gpu::GpuStats;

/// Render the always-visible GPU bar (1 row)
pub fn render(stats: Option<&GpuStats>, frame: &mut Frame, area: Rect) {
    let line = match stats {
        Some(s) => build_gpu_line(s),
        None => Line::from(Span::styled(
            " GPU: not detected ",
            Style::default().fg(Color::DarkGray),
        )),
    };

    let bar = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(bar, area);
}

fn build_gpu_line(stats: &GpuStats) -> Line<'static> {
    let util_color = level_color(stats.utilization_percent as f32, 60.0, 85.0);
    let vram_color = level_color(stats.vram_percent(), 60.0, 85.0);
    let temp_color = temp_level_color(stats.temperature_celsius, 70, 85);

    let vram_bar = mini_bar(stats.vram_percent(), 10);

    let mut spans = vec![
        Span::styled(
            " GPU ",
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{}%", stats.utilization_percent),
            Style::default().fg(util_color),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("VRAM ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}/{} MB ", stats.vram_used_mb, stats.vram_total_mb),
            Style::default().fg(vram_color),
        ),
        Span::styled(vram_bar, Style::default().fg(vram_color)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}°C", stats.temperature_celsius),
            Style::default().fg(temp_color),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:.0}W", stats.power_draw_watts),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    // Add fan speed if available (not on laptops)
    if let Some(fan) = stats.fan_speed_percent {
        spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(
            format!("Fan {}%", fan),
            Style::default().fg(Color::DarkGray),
        ));
    }

    Line::from(spans)
}

/// Color based on threshold levels
/// Green → Yellow → Red
fn level_color(value: f32, warn: f32, critical: f32) -> Color {
    if value > critical {
        Color::Red
    } else if value > warn {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Temperature-specific thresholds
fn temp_level_color(temp: u32, warn: u32, critical: u32) -> Color {
    if temp > critical {
        Color::Red
    } else if temp > warn {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Mini progress bar using Unicode blocks
/// Returns something like "████████░░" for 80%
fn mini_bar(percent: f32, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
