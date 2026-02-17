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
                .bg(Color::Green)
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
            " DOCKER ",
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let keyhints = match (&app.current_view, &app.input_mode) {
        (_, InputMode::Search) => " Esc:exit  Enter:confirm ",
        (_, InputMode::DeleteConfirm) => " y:confirm  n/Esc:cancel ",
        (_, InputMode::TagList) => " a:add  d:remove  j/k:nav  Esc:close ",
        (_, InputMode::TagInput) | (_, InputMode::NotesInput) => " Enter:save  Esc:cancel ",
        (_, InputMode::RunDialog) => " Tab:field  Space:toggle  Enter:run  Esc:cancel ",
        (View::Dashboard, _) => {
            " j/k:nav  Enter:open  Space:compare  /:search  R:run  g:gpu  ?:help  q:quit "
        }
        (View::RunDetail, _) => {
            " Esc:back  Tab:view  t:tags  n:notes  m:md  e:csv  x:tex  K:stop  ?:help "
        }
        (View::Compare, _) => " Esc:back  Tab:metric  m:md  e:csv  x:tex  ?:help ",
        (View::GpuMonitor, _) => " Esc/g:back  ?:help ",
        (View::Settings, _) => " Esc:back  ?:help  q:quit ",
        (View::Help, _) => " ?/Esc:close ",
        (View::Splash, _) => " Press any key... ",
        (View::Menu, _) => " j/k:nav  Enter:select  1-5:shortcut  q:quit ",
    };

    // Build status line with optional GPU mini-summary
    let mut spans = vec![
        mode_indicator,
        Span::styled(
            format!(" {} ", app.status_message),
            Style::default().fg(Color::Gray),
        ),
    ];

    // Add GPU mini-info on dashboard (not on GPU screen where it's redundant)
    if app.current_view != View::GpuMonitor {
        if let Some(stats) = &app.gpu_stats {
            let vram_color = if stats.vram_percent() > 85.0 {
                Color::Red
            } else if stats.vram_percent() > 60.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            spans.push(Span::styled(
                format!("GPU:{:>3}% ", stats.utilization_percent),
                Style::default().fg(vram_color),
            ));
            spans.push(Span::styled(
                format!("VRAM:{:.0}% ", stats.vram_percent()),
                Style::default().fg(vram_color),
            ));
            spans.push(Span::styled(
                format!("{}°C ", stats.temperature_celsius),
                Style::default().fg(if stats.temperature_celsius > 85 {
                    Color::Red
                } else {
                    Color::DarkGray
                }),
            ));
        }
    }

    spans.push(Span::styled(keyhints, Style::default().fg(Color::DarkGray)));

    let line = Line::from(spans);
    let status_bar = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(status_bar, area);
}

