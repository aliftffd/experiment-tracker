use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs as RatatuiTabs},
};

/// build a styled tabs widgets
pub fn styled_tabs<'a>(titles: &[String], selected: usize) -> RatatuiTabs<'a> {
    let tab_titles: Vec<Line> = titles.iter().map(|t| Line::from(t.clone())).collect();

    RatatuiTabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|")
}
