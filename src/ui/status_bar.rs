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

        InputMode::TagInput | InputMode::TagList => Span::styled(
            " TAG ",
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::NotesInput => Span::styled(
            " NOTES ",
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::DeleteConfirm => Span::styled(
            " DELETE? ",
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::RunDialog => Span::styled(
            " RUN ",
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let keyhints = match (&app.current_view, &app.input_mode) {
        (_, InputMode::Search) => " Esc:exit  Enter:confirm ",
        (_, InputMode::DeleteConfirm) => " y:confirm  n/Esc:cancel ",
        (_, InputMode::TagList) => " a:add  d:remove  j/k:nav  Esc:close ",
        (_, InputMode::TagInput) => " Enter:save  Esc:cancel ",
        (_, InputMode::NotesInput) => " Enter:save  Esc:cancel ",
        (_, InputMode::RunDialog) => " Enter:confirm  Esc:cancel ",
        (View::Dashboard, _) => " j/k:nav  Enter:open  Space:compare  /:search  d:delete  g:gpu  ?:help  q:quit ",
        (View::RunDetail, _) => " Esc:back  Tab:metric  s:status  t:tags  n:notes  m:md  e:csv  K:stop  ?:help ",
        (View::Compare, _) => " Esc:back  Tab:cycle metric  g:gpu  ?:help ",
        (View::GpuMonitor, _) => " Esc/g:back  ?:help ",
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
