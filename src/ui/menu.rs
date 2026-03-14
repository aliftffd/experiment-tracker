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

pub fn render(app: &mut App, frame: &mut Frame) {
    let use_unicode = platform::supports_unicode();

    let mut lines: Vec<Line> = Vec::new();

    // Compact art header
    let (art, separator) = if use_unicode {
        (ascii_art::MENU_ART, ascii_art::MENU_SEPARATOR_UNICODE)
    } else {
        (ascii_art::MENU_ART_ASCII, ascii_art::MENU_SEPARATOR_ASCII)
    };

    for line in art {
        lines.push(Line::from(Span::styled(
            *line,
            Style::default().fg(Color::Cyan),
        )));
    }
    lines.push(Line::from(Span::styled(
        separator,
        Style::default().fg(Color::Cyan),
    )));

    // Separator line (empty row in box)
    let left = if use_unicode { "║" } else { "|" };
    let right = if use_unicode { "║" } else { "|" };

    lines.push(Line::from(Span::styled(
        format!("{}                                               {}", left, right),
        Style::default().fg(Color::Cyan),
    )));

    // Menu items
    let menu_items = build_menu_items(app);

    for (i, (label, shortcut, context)) in menu_items.iter().enumerate() {
        let is_selected = app.menu_selected == i;
        let cursor = if is_selected { "▶" } else { " " };

        let cursor_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let label_style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let context_style = Style::default().fg(Color::DarkGray);

        lines.push(Line::from(vec![
            Span::styled(left, Style::default().fg(Color::Cyan)),
            Span::styled(format!("   {} ", cursor), cursor_style),
            Span::styled(format!("[{}]  ", shortcut), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:<18}", label), label_style),
            Span::styled(format!("{:<15}", context), context_style),
            Span::styled(right, Style::default().fg(Color::Cyan)),
        ]));
    }

    // Empty row
    lines.push(Line::from(Span::styled(
        format!("{}                                               {}", left, right),
        Style::default().fg(Color::Cyan),
    )));

    // Bottom separator
    lines.push(Line::from(Span::styled(
        separator,
        Style::default().fg(Color::Cyan),
    )));

    // Status footer
    let gpu_status = if let Some(stats) = &app.gpu_stats {
        format!(
            "GPU: {} {}{}C",
            if use_unicode { "\u{2713}" } else { "+" },
            stats.gpu_name,
            stats.temperature_celsius,
        )
    } else {
        format!("GPU: {} not detected", if use_unicode { "\u{2717}" } else { "-" })
    };

    let docker_status = if let Some(info) = &app.docker_info {
        if info.running {
            format!("Docker: {} Running", if use_unicode { "\u{2713}" } else { "+" })
        } else {
            format!("Docker: {} Stopped", if use_unicode { "\u{2717}" } else { "-" })
        }
    } else {
        format!("Docker: {} N/A", if use_unicode { "\u{2717}" } else { "-" })
    };

    let status_line1 = format!("  {}  {}", gpu_status, docker_status);
    let watch_dirs: Vec<String> = app.config.general.watch_dirs.clone();
    let watch_str = watch_dirs.first().map(|s| s.as_str()).unwrap_or(".");
    let status_line2 = format!("  DB: {} runs  Watch: {}", app.runs.len(), watch_str);

    let footer1 = format!("{} {:<46}{}", left, status_line1, right);
    let footer2 = format!("{} {:<46}{}", left, status_line2, right);

    lines.push(Line::from(Span::styled(footer1, Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(Span::styled(footer2, Style::default().fg(Color::DarkGray))));

    // Bottom border
    let bottom = if use_unicode {
        ascii_art::MENU_BOTTOM_UNICODE
    } else {
        ascii_art::MENU_BOTTOM_ASCII
    };

    {
        lines.push(Line::from(Span::styled(
            bottom,
            Style::default().fg(Color::Cyan),
        )));
    }

    let total_height = lines.len() as u16;
    let area = frame.area();
    let centered = centered_rect(ascii_art::MENU_ART_WIDTH + 4, total_height, area);

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(paragraph, centered);
}

fn build_menu_items(app: &App) -> Vec<(&'static str, &'static str, String)> {
    let run_count = app.runs.len();

    let docker_context = if app.docker.is_some() {
        "(Docker)".to_string()
    } else {
        "(unavailable)".to_string()
    };

    let gpu_context = if let Some(stats) = &app.gpu_stats {
        format!("({}{}C)", stats.gpu_name, stats.temperature_celsius)
    } else {
        "(not detected)".to_string()
    };

    let compare_context = if app.compare_run_ids.is_empty() {
        String::new()
    } else {
        format!("({} selected)", app.compare_run_ids.len())
    };

    vec![
        ("Dashboard", "1", format!("({} experiments)", run_count)),
        ("Run Experiment", "2", docker_context),
        ("GPU Monitor", "3", gpu_context),
        ("Compare Runs", "4", compare_context),
        ("Settings", "5", String::new()),
        ("Quit", "q", String::new()),
    ]
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
