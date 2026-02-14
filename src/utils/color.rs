use ratatui::style::Color;

use crate::models::RunStatus;

/// Rotating color palette for chart lines
const CHART_COLORS: &[Color] = &[
    Color::Cyan,
    Color::Magenta,
    Color::Yellow,
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::LightCyan,
    Color::LightMagenta,
    Color::LightYellow,
    Color::LightGreen,
];

/// Get a color for a chart line by index
pub fn chart_color(index: usize) -> Color {
    CHART_COLORS[index % CHART_COLORS.len()]
}

/// Get color for a run status
pub fn status_color(status: &RunStatus) -> Color {
    match status {
        RunStatus::Running => Color::Yellow,
        RunStatus::Completed => Color::Green,
        RunStatus::Failed => Color::Red,
        RunStatus::Stopped => Color::Gray,
    }
}
