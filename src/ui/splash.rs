use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::App;
use crate::platform;
use super::ascii_art;

pub fn render(app: &App, frame: &mut Frame) {
    let use_unicode = platform::supports_unicode();

    let mut lines: Vec<Line> = Vec::new();

    // Large block-letter art
    if use_unicode {
        for line in ascii_art::SPLASH_ART {
            lines.push(Line::from(Span::styled(
                *line,
                Style::default().fg(Color::Cyan),
            )));
        }
    } else {
        for line in ascii_art::SPLASH_ART_ASCII {
            lines.push(Line::from(Span::styled(
                *line,
                Style::default().fg(Color::Cyan),
            )));
        }
    }

    // Blank line
    lines.push(Line::from(""));

    // Version string centered
    lines.push(Line::from(Span::styled(
        ascii_art::version_string(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));

    // Blank line
    lines.push(Line::from(""));

    // System info line: GPU | Docker | runs
    let gpu_part = if let Some(stats) = &app.gpu_stats {
        format!("GPU: {}", stats.gpu_name)
    } else {
        "GPU: not detected".to_string()
    };

    let docker_part = if let Some(info) = &app.docker_info {
        if info.running {
            "Docker: Ready".to_string()
        } else {
            "Docker: not running".to_string()
        }
    } else {
        "Docker: not found".to_string()
    };

    let runs_part = format!("{} runs", app.runs.len());

    let status_line = if use_unicode {
        format!("\u{26A1} {} \u{2502} \u{1F433} {} \u{2502} {}", gpu_part, docker_part, runs_part)
    } else {
        format!("* {} | {} | {}", gpu_part, docker_part, runs_part)
    };

    lines.push(Line::from(Span::styled(
        status_line,
        Style::default().fg(Color::DarkGray),
    )));

    // Blank line
    lines.push(Line::from(""));

    // Press any key prompt
    lines.push(Line::from(Span::styled(
        "Press any key to continue...",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    let art_width = if use_unicode {
        ascii_art::SPLASH_ART_WIDTH
    } else {
        ascii_art::SPLASH_ART_ASCII_WIDTH
    };

    let total_height = lines.len() as u16;
    let area = frame.area();
    let centered = centered_rect(art_width + 4, total_height, area);

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, centered);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
