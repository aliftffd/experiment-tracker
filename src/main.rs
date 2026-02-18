mod app;
mod config;
mod db;
mod docker;
mod export;
mod gpu;
mod models;
mod platform;
mod ui;
mod utils;
mod watcher;

use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::handler::{execute_action, handle_key_event};
use crate::app::state::App;
use crate::config::AppConfig;
use crate::db::Database;
use crate::watcher::directory::{DirectoryWatcher, WatchEvent};

#[derive(Parser, Debug)]
#[command(name = "experiment-tracker")]
#[command(about = "A TUI experiment tracker for ML workflows")]
#[command(version)]
struct Cli {
    /// Path to config file
    #[arg(short, long)]
    config: Option<String>,

    /// Watch directory (overrides config)
    #[arg(short, long)]
    watch: Option<Vec<String>>,

    /// Database path (overrides config)
    #[arg(short, long)]
    db: Option<String>,

    /// Seed database with sample data for testing
    #[arg(long)]
    seed: bool,

    /// Skip initial file scan on startup
    #[arg(long)]
    no_scan: bool,

    /// Skip splash screen, go directly to menu
    #[arg(long)]
    no_splash: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config
    let mut config = AppConfig::load(cli.config.as_deref().map(std::path::Path::new))?;

    // CLI overrides
    if let Some(watch_dirs) = cli.watch {
        config.general.watch_dirs = watch_dirs;
    }
    if let Some(db_path) = cli.db {
        config.general.db_path = db_path;
    }

    // Open database
    let db_path = config.resolved_db_path();
    let db = Database::open(&db_path)
        .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

    // Seed sample data if requested
    if cli.seed {
        seed_sample_data(&db)?;
        eprintln!("Seeded sample data into {}", db_path.display());
    }

    // Create app
    let mut app = App::new(config.clone(), db)?;

    // Skip splash if requested
    if cli.no_splash {
        app.current_view = crate::app::state::View::Menu;
    }

    // Initial scan: import existing log files
    if !cli.no_scan {
        match app.import_existing_files() {
            Ok(count) => {
                if count > 0 {
                    app.set_status(format!("Imported {} existing log files", count));
                }
            }
            Err(e) => {
                app.set_status(format!("Scan warning: {}", e));
            }
        }
    }

    // Start the file watcher
    let watch_dirs = config.resolved_watch_dirs();
    let watcher = match DirectoryWatcher::new(&watch_dirs) {
        Ok(w) => Some(w),
        Err(e) => {
            app.set_status(format!("Watcher error: {} — running without live updates", e));
            None
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run event loop
    let result = run_event_loop(&mut terminal, &mut app, &config, watcher.as_ref());

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    config: &AppConfig,
    watcher: Option<&DirectoryWatcher>,
) -> Result<()> {
    let tick_rate = Duration::from_millis(config.general.refresh_rate_ms);

    loop {
        // Render
        terminal.draw(|frame| {
            ui::render(app, frame);
        })?;

        // Handle keyboard events
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                let action = handle_key_event(app, key);
                execute_action(app, action);
            }
        }

        // Handle file watcher events
        if let Some(w) = watcher {
            for event in w.drain_events() {
                match event {
                    WatchEvent::FileChanged(path) => {
                        if let Err(e) = app.import_log_file(&path) {
                            app.set_status(format!(
                                "Import error: {} — {}",
                                path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown"),
                                e
                            ));
                        }
                    }
                    WatchEvent::FileRemoved(path) => {
                        app.set_status(format!(
                            "File removed: {}",
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                        ));
                        // We don't auto-delete runs when files are removed
                        // User might have just moved the file
                    }
                    WatchEvent::Error(msg) => {
                        app.set_status(format!("Watcher: {}", msg));
                    }
                }
            }
        }

        // Poll GPU stats
        app.poll_gpu_if_needed();

        // Check quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Seed the database with sample experiment data for testing
fn seed_sample_data(db: &Database) -> Result<()> {
    // Skip if already seeded
    let existing = db.get_all_runs().unwrap_or_default();
    if !existing.is_empty() {
        eprintln!("Database already contains runs, skipping seed.");
        return Ok(());
    }

    // Run 1: A completed training run
    let run1 = db.insert_run(
        "tiny-trans-baseline-v1",
        "./experiments/run_001/metrics.jsonl",
    )?;
    db.update_run_status(run1.id, &models::RunStatus::Completed)?;
    db.add_tag(run1.id, "baseline")?;
    db.add_tag(run1.id, "transformer")?;

    for epoch in 0i64..50 {
        let loss = 2.5 * (-0.05 * epoch as f64).exp() + 0.3 + 0.05 * (epoch as f64 * 0.1).sin();
        let acc = 0.45 + 0.45 * (1.0 - (-0.08 * epoch as f64).exp());
        let lr = 0.001 * (0.95_f64).powi(epoch as i32);

        db.insert_metric(run1.id, "loss", Some(epoch), Some(epoch * 100), loss)?;
        db.insert_metric(run1.id, "accuracy", Some(epoch), Some(epoch * 100), acc)?;
        db.insert_metric(run1.id, "learning_rate", Some(epoch), Some(epoch * 100), lr)?;
    }

    // Run 2: A running experiment
    let run2 = db.insert_run(
        "tiny-trans-optuna-trial-7",
        "./experiments/run_002/metrics.jsonl",
    )?;
    db.add_tag(run2.id, "optuna")?;
    db.add_tag(run2.id, "hpo")?;

    for epoch in 0i64..25 {
        let loss = 3.0 * (-0.03 * epoch as f64).exp() + 0.5;
        let acc = 0.30 + 0.35 * (1.0 - (-0.06 * epoch as f64).exp());

        db.insert_metric(run2.id, "loss", Some(epoch), Some(epoch * 100), loss)?;
        db.insert_metric(run2.id, "accuracy", Some(epoch), Some(epoch * 100), acc)?;
    }

    // Run 3: A failed run
    let run3 = db.insert_run(
        "bci-motor-imagery-v2",
        "./experiments/run_003/metrics.jsonl",
    )?;
    db.update_run_status(run3.id, &models::RunStatus::Failed)?;
    db.add_tag(run3.id, "bci")?;
    db.update_run_notes(run3.id, "OOM at epoch 15 - reduce batch size")?;

    for epoch in 0i64..15 {
        let loss = 1.8 * (-0.02 * epoch as f64).exp() + 0.8;
        let acc = 0.33 + 0.15 * (1.0 - (-0.04 * epoch as f64).exp());

        db.insert_metric(run3.id, "loss", Some(epoch), Some(epoch * 100), loss)?;
        db.insert_metric(run3.id, "accuracy", Some(epoch), Some(epoch * 100), acc)?;
    }

    // Run 4: Another completed run with different hyperparams
    let run4 = db.insert_run(
        "tiny-trans-large-dim",
        "./experiments/run_004/metrics.jsonl",
    )?;
    db.update_run_status(run4.id, &models::RunStatus::Completed)?;
    db.add_tag(run4.id, "transformer")?;
    db.add_tag(run4.id, "large")?;

    for epoch in 0i64..40 {
        let loss = 2.0 * (-0.06 * epoch as f64).exp() + 0.25;
        let acc = 0.50 + 0.42 * (1.0 - (-0.1 * epoch as f64).exp());

        db.insert_metric(run4.id, "loss", Some(epoch), Some(epoch * 100), loss)?;
        db.insert_metric(run4.id, "accuracy", Some(epoch), Some(epoch * 100), acc)?;
    }

    // Run 5: Stopped run
    let run5 = db.insert_run("amc-vit-experiment", "./experiments/run_005/metrics.jsonl")?;
    db.update_run_status(run5.id, &models::RunStatus::Stopped)?;
    db.add_tag(run5.id, "vit")?;
    db.update_run_notes(run5.id, "Stopped - accuracy plateaued")?;

    for epoch in 0i64..20 {
        let loss = 2.2 * (-0.04 * epoch as f64).exp() + 0.6;
        let acc = 0.40 + 0.20 * (1.0 - (-0.05 * epoch as f64).exp());

        db.insert_metric(run5.id, "loss", Some(epoch), Some(epoch * 100), loss)?;
        db.insert_metric(run5.id, "accuracy", Some(epoch), Some(epoch * 100), acc)?;
    }

    Ok(())
}
