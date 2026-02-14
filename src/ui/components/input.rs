use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

/// Build a styled text input widget
pub fn text_input<'a>(label: &'a str, value: &'a str, active: bool) -> Paragraph<'a> {
    let border_color = if active { Color::Cyan } else { Color::DarkGray };

    let display = if active && value.is_empty() {
        "type to search..."
    } else {
        value
    };

    let cursor = if active { "█" } else { "" };

    Paragraph::new(format!("{}{}", display, cursor)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", label))
            .border_style(Style::default().fg(border_color)),
    )
}
