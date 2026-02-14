use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
};

/// build a syled table widgets

pub fn styled_table<'a>(
    title: &'a str,
    headers: Vec<&'a str>,
    rows: Vec<Row<'a>>,
    widths: Vec<Constraint>,
    selected: Option<usize>,
) -> (Table<'a>, Option<ratatui::widgets::TableState>) {
    let header_cells: Vec<ratatui::text::Line> = headers
        .iter()
        .map(|h| ratatui::text::Line::from(*h))
        .collect();

    let header = Row::new(header_cells)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{}", title))
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶  ");

    let state = selected.map(|i| {
        let mut s = ratatui::widgets::TableState::default();
        s.select(Some(i));
        s
    });

    (table, state)
}
