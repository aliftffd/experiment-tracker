use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::state::RunDialogState;

/// show the docker dialog
pub fn render(dialog: &RunDialogState, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 14, area);

    let fields = [
        ("Image", &dialog.image, 0),
        ("Command", &dialog.command, 1),
        ("Output Dir", &dialog.output_dir, 2),
    ];

    let mut lines = vec![Line::from("")];

    for (label, value, idx) in &fields {
        let is_active = dialog.active_field == *idx;
        let label_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let cursor = if is_active { "█" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(format!(" {:<12}", label), label_style),
            Span::styled(
                format!("{}{}", value, cursor),
                Style::default().fg(Color::White),
            ),
        ]));
        lines.push(Line::from("")); // spacing
    }

    // Gpu Toggle
    let gpu_style = if dialog.active_field == 3 {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let gpu_check = if dialog.use_gpu { "✓" } else { " " };
    lines.push(Line::from(vec![
            Span::styled(" GPU      ", gpu_style),
            Span::styled(
                format!("[{}] enabled", gpu_check),
                Style::default().fg(if dialog.use_gpu {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
                ),
    ]));

    lines.push(Line::from(""));

    // error message if any
    if !dialog.error_message.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  ⚠ {}", dialog.error_message),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(""));
    }

    // Controls
    lines.push(Line::from(vec![
        Span::styled("  Tab", Style::default().fg(Color::Cyan)),
        Span::raw(":next field  "),
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::raw(":toggle GPU  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(":run  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(":cancel"),
    ]));

    let popup = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Run Experiment (Docker) ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
