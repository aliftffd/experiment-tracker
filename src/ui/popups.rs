use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Calculate a centered rect for popups
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);

    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

/// Render a delete confirmation popup
pub fn render_delete_confirm(run_name: &str, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 7, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Delete "),
            Span::styled(
                run_name,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  This will remove all metrics, tags, and notes.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [y] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Confirm    "),
            Span::styled(
                "[n/Esc] ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Cancel"),
        ]),
    ];

    let popup = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Delete ")
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

/// Render a text input popup (used for tags and notes)
pub fn render_input_popup(title: &str, prompt: &str, value: &str, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 6, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", prompt),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{}{}", value, "█"),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let popup = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title))
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

/// Render a tag list popup (view/remove tags)
pub fn render_tag_list(tags: &[String], selected: usize, frame: &mut Frame, area: Rect) {
    let height = (tags.len() as u16 + 4).min(15);
    let popup_area = centered_rect(40, height, area);

    let mut lines = vec![Line::from("")];

    if tags.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No tags",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, tag) in tags.iter().enumerate() {
            let marker = if i == selected { "▶ " } else { "  " };
            let style = if i == selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!("  {}{}", marker, tag),
                style,
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [a] ", Style::default().fg(Color::Green)),
        Span::raw("Add  "),
        Span::styled("[d] ", Style::default().fg(Color::Red)),
        Span::raw("Remove  "),
        Span::styled("[Esc] ", Style::default().fg(Color::DarkGray)),
        Span::raw("Close"),
    ]));

    let popup = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Tags ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

/// Render a notes editor popup
pub fn render_notes_editor(
    current_notes: &str,
    editing_value: &str,
    frame: &mut Frame,
    area: Rect,
) {
    let popup_area = centered_rect(60, 10, area);

    let mut lines = vec![Line::from("")];

    if !current_notes.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  Current: {}", current_notes),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "  New note:",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{}{}", editing_value, "█"),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Enter ", Style::default().fg(Color::Green)),
        Span::raw("Save  "),
        Span::styled("Esc ", Style::default().fg(Color::Red)),
        Span::raw("Cancel"),
    ]));

    let popup = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Edit Notes ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}
