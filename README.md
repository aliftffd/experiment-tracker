# ExpTrack — ML Experiment Tracker

A terminal UI (TUI) application for tracking machine learning experiments, built with Rust using [ratatui](https://github.com/ratatui-org/ratatui) and [crossterm](https://github.com/crossterm-rs/crossterm).

```
╔═══════════════════════════════════════════════╗
║                                               ║
║   ▄▄▄ ▄   ▄ ▄▄▄  ▄▄▄▄ ▄▄▄  ▄▄▄  ▄▄▄ ▄  ▄  ║
║   █    ▀▄▀  █▄▄█   █   █▄▄▀ █▄▄█ █   █▄▄▀   ║
║   ▀▀▀ ▀ ▀  █      ▀   ▀  ▀ ▀  ▀ ▀▀▀ ▀  ▀   ║
║                                               ║
║        ML Experiment Tracker v0.1.0           ║
║                                               ║
╚═══════════════════════════════════════════════╝
```

---

## Features

- **Dashboard** — browse all experiment runs with status, timestamps, and inline sparklines
- **Run Detail** — view metric charts, hyperparameters, and live Docker container logs per run
- **Compare Runs** — side-by-side metric comparison across up to 5 runs
- **GPU Monitor** — live GPU utilization, VRAM, temperature, power draw, and process list via `nvidia-smi`
- **Docker Integration** — launch training containers directly from the TUI with GPU passthrough support
- **Live File Watcher** — automatically imports new metrics as `.jsonl` or `.csv` log files are written
- **Export** — export any run or comparison to Markdown, CSV, or LaTeX
- **Settings** — read-only view of current config, database stats, Docker status, and GPU info
- **Search** — fuzzy filter runs by name, status, or notes

---

## Requirements

- Rust 1.75+
- Linux or Windows (cross-platform via crossterm)
- **Optional:** `nvidia-smi` for GPU monitoring
- **Optional:** Docker for launching training containers

---

## Installation

```bash
git clone <repo>
cd experiment-tracker
cargo build --release
```

The binary is at `target/release/experiment-tracker`.

---

## Usage

```bash
# Run normally (shows splash → menu)
experiment-tracker

# Skip splash screen, go straight to the menu
experiment-tracker --no-splash

# Seed the database with 5 sample runs for testing
experiment-tracker --seed

# Use a custom config file
experiment-tracker --config path/to/config.toml

# Override watch directory
experiment-tracker --watch ./my-experiments

# Override database path
experiment-tracker --db /tmp/test.db

# Skip the initial file scan on startup
experiment-tracker --no-scan
```

---

## Log File Format

Drop `.jsonl` or `.csv` files into your watched directory and the app picks them up automatically.

**JSON Lines (`.jsonl`):**
```json
{"epoch": 1, "step": 100, "loss": 0.542, "accuracy": 0.73}
{"epoch": 2, "step": 200, "loss": 0.431, "accuracy": 0.81}
```

Hyperparameters can be embedded in a special line:
```json
{"hyperparams": {"lr": 0.001, "batch_size": 32, "model": "transformer"}}
```

**CSV (`.csv`):**
```
epoch,step,loss,accuracy
1,100,0.542,0.73
2,200,0.431,0.81
```

Format is auto-detected by file extension and content.

---

## Navigation

```
App Start
  │
  ▼
Splash (2s or any key)
  │
  ▼
Menu ◄──────────────────────────────┐
  ├── [1] Dashboard ── Run Detail   │
  │        │ Esc                    │
  │        └────────────────────────┤
  ├── [2] Run Dialog (popup)        │
  │        │ Esc cancels            │
  │        └────────────────────────┤
  ├── [3] GPU Monitor               │
  │        │ Esc                    │
  │        └────────────────────────┤
  ├── [4] Compare                   │
  │        │ Esc                    │
  │        └────────────────────────┤
  ├── [5] Settings                  │
  │        │ Esc                    │
  │        └────────────────────────┘
  └── [q] Quit
```

`Esc` from any top-level view returns to the Menu.

---

## Keybindings

### Menu
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Select item |
| `1`–`5` | Jump to item |
| `q` | Quit |

### Dashboard
| Key | Action |
|-----|--------|
| `j` / `↓` | Next run |
| `k` / `↑` | Previous run |
| `Enter` / `l` | Open run detail |
| `Space` | Toggle run for comparison |
| `c` | Go to Compare view |
| `g` | Go to GPU Monitor |
| `R` | Open Run Dialog (Docker) |
| `/` | Search / filter runs |
| `d` | Delete selected run |
| `r` | Refresh |
| `?` | Toggle help |
| `Esc` | Back to Menu |

### Run Detail
| Key | Action |
|-----|--------|
| `Tab` | Cycle sub-view (Chart → Hyperparams → Logs) |
| `j` / `k` | Navigate metrics |
| `s` | Toggle run status |
| `t` | Manage tags |
| `n` | Edit notes |
| `K` | Stop running container |
| `m` | Export as Markdown |
| `e` | Export as CSV |
| `x` | Export as LaTeX |
| `Esc` | Back to Dashboard |

### Compare View
| Key | Action |
|-----|--------|
| `Tab` | Cycle metric |
| `m` / `e` / `x` | Export (Markdown / CSV / LaTeX) |
| `Esc` | Back to Menu |

### GPU Monitor
| Key | Action |
|-----|--------|
| `Esc` / `g` | Back to Menu |

### Settings
| Key | Action |
|-----|--------|
| `Esc` | Back to Menu |

---

## Configuration

A `config.toml` is looked up from the current directory, or you can specify one with `--config`. The default database is stored at `~/.local/share/experiment-tracker/tracker.db` on Linux.

```toml
[general]
watch_dirs = ["./experiments"]   # directories to scan for log files
refresh_rate_ms = 250            # UI refresh interval

[ui]
theme = "dark"
show_sparklines = true
show_gpu_bar = true
max_chart_points = 500

[parser]
default_format = "auto"          # "jsonl", "csv", or "auto"
epoch_field = "epoch"
step_field = "step"
loss_field = "loss"
accuracy_field = "accuracy"

[gpu]
poll_interval_secs = 2
temp_warning = 80                # °C
temp_critical = 90
vram_warning = 80                # %
vram_critical = 95

[docker]
default_image = "thesis-training:latest"
gpu = true
container_workdir = "/workspace/output"
```

---

## Export

From Run Detail or Compare view, exports are written to `<watch_dir>/exports/` with a timestamped filename.

| Format | Key | Output |
|--------|-----|--------|
| Markdown | `m` | `run-name_20240118_143022.md` |
| CSV | `e` | `run-name_20240118_143022.csv` |
| LaTeX | `x` | `run-name_20240118_143022.tex` |

---

## Docker Integration

From the menu select **Run Experiment** (or press `R` in the Dashboard). A dialog lets you configure:

- Docker image
- Training command
- Output directory
- GPU passthrough (requires `nvidia-container-toolkit`)

The app mounts the output directory into the container and watches it for new metric files. Container status (running / completed / failed) is tracked in the database and updated automatically.

---

## Project Structure

```
src/
├── main.rs              # Entry point, CLI parsing, event loop
├── app/
│   ├── state.rs         # App state, View enum, InputMode
│   ├── actions.rs       # Action enum
│   └── handler.rs       # Key handling and action execution
├── ui/
│   ├── mod.rs           # Top-level render dispatch
│   ├── splash.rs        # Splash screen
│   ├── menu.rs          # Main menu
│   ├── dashboard.rs     # Run list
│   ├── run_detail.rs    # Single run detail (chart, hyperparams, logs)
│   ├── compare.rs       # Multi-run comparison
│   ├── gpu_screen.rs    # Full GPU monitor screen
│   ├── gpu_bar.rs       # Top GPU status bar
│   ├── settings.rs      # Settings/config overview
│   ├── status_bar.rs    # Bottom status bar
│   ├── popups.rs        # Help and other overlays
│   ├── run_dialog.rs    # Docker run dialog popup
│   ├── ascii_art.rs     # Shared ASCII art constants
│   └── components/      # Reusable widgets (table, tabs, sparkline, input, popup)
├── db/
│   ├── schema.rs        # SQLite schema and migrations
│   ├── runs.rs          # Run CRUD
│   ├── metrics.rs       # Metric insert/query
│   └── tags.rs          # Tag management
├── models/
│   ├── run.rs           # Run model
│   ├── metric.rs        # Metric model
│   ├── tag.rs           # Tag model
│   └── hyperparams.rs   # HyperParam model
├── gpu/
│   └── monitor.rs       # nvidia-smi polling, GpuStats, GpuHistory
├── docker/
│   └── manager.rs       # Docker container lifecycle management
├── watcher/
│   ├── directory.rs     # File system watcher (notify crate)
│   └── parser.rs        # JSONL and CSV log parser
├── export/
│   ├── markdown.rs      # Markdown export
│   ├── csv.rs           # CSV export
│   └── latex.rs         # LaTeX export
├── config/
│   └── settings.rs      # Config structs and TOML loading
├── platform.rs          # OS-specific helpers (paths, nvidia-smi location, unicode support)
└── utils/               # Color and time utilities
```

---

## License

MIT
