use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::{App, InputMode, View};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let mode_indicator = match app.input_mode {
        InputMode::Normal => Span::styled(
            " NORMAL ",
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),

        InputMode::Search => Span::styled(
            " SEARCH ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),

        InputMode::TagInput => Span::styled(
            " TAG ",
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let keyhints = match (&app.current_view, &app.input_mode) {
        (_, InputMode::Search) => " Esc:exit  Enter:confirm ",
        (View::Dashboard, _) => " j/k:navigate  Enter:open  /:search  d:delete  ?:help  q:quit ",
        (View::RunDetail, _) => " Esc:back  Tab:metric  s:status  t:tag  m:export  ?:help ",
        (View::Compare, _) => " Esc:back  j/k:navigate  ?:help ",
        (View::Help, _) => " ?/Esc:close help ",
    };

    let line = Line::from(vec![
        mode_indicator,
        Span::styled(
            format!(" {} ", app.status_message),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(keyhints, Style::default().fg(Color::DarkGray)),
    ]);

    let status_bar = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(status_bar, area);
}
