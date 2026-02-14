use chrono::NaiveDateTime;

/// Format a datetime as relative time (e.g., "2m ago", "3h ago", "1d ago")
pub fn relative_time(dt: &NaiveDateTime) -> String {
    let now = chrono::Local::now().naive_local();
    let diff = now.signed_duration_since(*dt);

    if diff.num_seconds() < 60 {
        "just now".into()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_days() < 30 {
        format!("{}d ago", diff.num_days())
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

/// Format a duration in seconds to human-readable (e.g., "1h 23m 45s")
pub fn format_duration_secs(secs: i64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
