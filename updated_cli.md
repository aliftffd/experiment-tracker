# Instruction: Add Splash Screen, Main Menu, and Settings to Experiment Tracker

## Project Context

This is a Rust TUI application — an ML experiment tracker built with `ratatui` + `crossterm`. The project is structured in `src/` with these modules: `app/` (state, actions, handler), `ui/` (dashboard, run_detail, compare, gpu_bar, gpu_screen, popups, run_dialog, status_bar, components/), `db/`, `docker/`, `gpu/`, `export/`, `watcher/`, `config/`, `models.rs`, `platform.rs`, `utils/`.

The app currently launches directly into the Dashboard view. This task adds 3 new features: a splash screen, a main menu, and a settings screen.

---

## Feature 1: Splash Screen

### What It Is

A 2-second branded landing screen shown on app startup. It masks initialization latency (GPU detection, Docker check, DB open, file scan) while giving the app visual identity.

### ASCII Art

Use this exact ASCII art (compact version that fits 60-column terminals):

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

Also provide an ASCII fallback for terminals that don't support Unicode (check `platform::supports_unicode()`):

```
    +-----------------------------------------------+
    |                                               |
    |   EXPTRACK                                    |
    |   ML Experiment Tracker v0.1.0                |
    |                                               |
    +-----------------------------------------------+
```

### Behavior

- Display the splash screen immediately on startup.
- While the splash is visible, run initialization in the background (it's already done before render, so this is just visual delay).
- Show system detection results below the ASCII art:
  - GPU: name if detected, or "not detected"
  - Docker: "Ready" if running, "not found" if unavailable
  - Database: number of existing runs
- Auto-dismiss after 2 seconds OR on any keypress (whichever comes first).
- Add a `--no-splash` CLI flag (add to the `Cli` struct in `main.rs`) that skips directly to the menu.
- Show "Press any key to continue..." at the bottom.

### Implementation

1. Add `View::Splash` to the `View` enum in `src/app/state.rs`.
2. Create `src/ui/splash.rs` with a `render(app: &App, frame: &mut Frame)` function.
3. Center the ASCII art vertically and horizontally in the terminal.
4. Use `Color::Cyan` for the box borders, `Color::White` with `Modifier::BOLD` for the title text, `Color::DarkGray` for the status info.
5. In `main.rs`, set `app.current_view = View::Splash` initially.
6. In the event loop: if `current_view == View::Splash`, check if 2 seconds have elapsed since startup OR any key was pressed → transition to `View::Menu`.
7. Add a `splash_start: Instant` field to `App` state to track the 2-second timer.

---

## Feature 2: Main Menu

### What It Is

A navigation hub shown after the splash screen. It presents all top-level features as a selectable list with live context information on the right side.

### Layout

```
    ╔═══════════════════════════════════════════════╗
    ║                                               ║
    ║   ▄▄▄ ▄   ▄ ▄▄▄  ▄▄▄▄ ▄▄▄  ▄▄▄  ▄▄▄ ▄  ▄  ║
    ║   █    ▀▄▀  █▄▄█   █   █▄▄▀ █▄▄█ █   █▄▄▀   ║
    ║   ▀▀▀ ▀ ▀  █      ▀   ▀  ▀ ▀  ▀ ▀▀▀ ▀  ▀   ║
    ║                                               ║
    ╠═══════════════════════════════════════════════╣
    ║                                               ║
    ║   ▶ [1]  Dashboard         (5 experiments)    ║
    ║     [2]  Run Experiment    (Docker)            ║
    ║     [3]  GPU Monitor       (RTX 3050 — 42°C)  ║
    ║     [4]  Compare Runs                         ║
    ║     [5]  Settings                             ║
    ║     [q]  Quit                                 ║
    ║                                               ║
    ╠═══════════════════════════════════════════════╣
    ║  GPU: ✓ RTX 3050  Docker: ✓ Running           ║
    ║  DB: 5 runs  Watch: ./experiments             ║
    ╚═══════════════════════════════════════════════╝
```

### Menu Items

Each menu item has: label, keyboard shortcut (number key), and dynamic context text.

| # | Label | Context (dynamic) | Action |
|---|-------|-------------------|--------|
| 1 | Dashboard | `(N experiments)` — count from `app.runs.len()` | Navigate to `View::Dashboard` |
| 2 | Run Experiment | `(Docker)` if Docker available, `(unavailable)` if not | Open `RunDialog` if Docker available, show status message if not |
| 3 | GPU Monitor | GPU name + temp if detected, `(not detected)` if no GPU | Navigate to `View::GpuMonitor` |
| 4 | Compare Runs | `(N selected)` if runs are marked, empty otherwise | Navigate to `View::Compare` if runs selected, show status message if none |
| 5 | Settings | (no dynamic text) | Navigate to `View::Settings` |
| q | Quit | (no dynamic text) | Quit the app |

### Behavior

- `j/k` or `↑/↓` moves the `▶` cursor between menu items.
- `Enter` or the number key (1-5) selects the item.
- `q` quits.
- The cursor wraps around (going up from item 1 goes to Quit, going down from Quit goes to item 1).
- The menu is the new "home" — `Esc` from Dashboard, GPU Monitor, Compare, or Settings returns to the Menu.
- The bottom status bar shows live GPU summary and Docker status.
- GPU polling should still happen while on the menu (so the temperature shown is live).

### Implementation

1. Add `View::Menu` to the `View` enum.
2. Add `menu_selected: usize` field to `App` state (0-5 for the 6 menu items, default 0).
3. Create `src/ui/menu.rs` with `render(app: &mut App, frame: &mut Frame)`.
4. The ASCII art from the splash should appear at the top of the menu too (reuse the art as a shared constant or function in a `src/ui/ascii_art.rs` helper module).
5. Add menu key handling in `handler.rs`: when `View::Menu` is active, handle `j/k/Up/Down` for navigation, `Enter/1-5` for selection, `q` for quit.
6. Update `Back` action: when on Dashboard/GpuMonitor/Compare/Settings, `Esc` goes to `View::Menu` instead of `View::Dashboard`.
7. The Menu view should NOT show the GPU bar at the top (it already has system info in the bottom panel of the menu box).

### Menu Actions on Selection

```rust
match menu_selected {
    0 => navigate_to(View::Dashboard),
    1 => {
        if docker_available {
            open_run_dialog()
        } else {
            set_status("Docker not available")
        }
    },
    2 => navigate_to(View::GpuMonitor),
    3 => {
        if compare_run_ids.is_empty() {
            set_status("No runs selected. Go to Dashboard and press Space to mark runs.")
        } else {
            load_compare_data();
            navigate_to(View::Compare)
        }
    },
    4 => navigate_to(View::Settings),
    5 => should_quit = true,
}
```

---

## Feature 3: Settings Screen

### What It Is

A read-only configuration overview screen. Shows current app settings, system capabilities, and database stats.

### Layout

```
┌─────────────── Settings ──────────────────────────┐
│                                                   │
│  Watch Directories                                │
│  ────────────────                                 │
│  1. ./experiments                                 │
│                                                   │
│  Database                                         │
│  ────────                                         │
│  Path: ~/.local/share/experiment-tracker/         │
│  Runs: 5  │  Size: 340 KB                         │
│                                                   │
│  Docker                                           │
│  ──────                                           │
│  Status: ✓ Running (v24.0.7)                      │
│  GPU Support: ✓ nvidia-container-toolkit          │
│  Default Image: thesis-training:latest            │
│                                                   │
│  GPU                                              │
│  ───                                              │
│  Device: NVIDIA GeForce RTX 3050 Mobile           │
│  Driver: 545.29                                   │
│  Poll Interval: 2s                                │
│  Temp Warning: 80°C  │  Critical: 90°C            │
│                                                   │
│  Config File                                      │
│  ───────────                                      │
│  Location: ~/.config/experiment-tracker/config.toml│
│                                                   │
└───────────────────────────────────────────────────┘
 NORMAL  Settings  Esc:back  ?:help  q:quit
```

### Behavior

- Read-only. No editing. Just displays current configuration.
- `Esc` returns to Menu.
- The GPU bar appears at the top (like other views).
- Status bar at the bottom with normal keyhints.
- Data comes from `app.config`, `app.docker_info`, `app.gpu_stats`, and `app.db` (run count).
- For database file size, use `std::fs::metadata()` on the DB path.
- If Docker is not installed, show "Status: ✗ Not found" in red.
- If GPU is not detected, show "Device: Not detected" in gray.

### Implementation

1. Add `View::Settings` to the `View` enum.
2. Create `src/ui/settings.rs` with `render(app: &mut App, frame: &mut Frame)`.
3. Layout: GPU bar (1 row) → Scrollable content → Status bar (1 row).
4. Content is a `Paragraph` with styled `Line`s. Section headers in `Cyan` + `Bold`, labels in `DarkGray`, values in `White`.
5. Add settings key handling in handler: `Esc` → `Back` (to menu).
6. Add a helper `App::db_file_size()` method that returns the DB file size as a formatted string (e.g., "340 KB", "1.2 MB").

---

## Navigation Flow (Updated)

```
App Start
  │
  ▼
Splash (2s or keypress)
  │
  ▼
Menu ◄──────────────────────────────────┐
  │                                     │
  ├── [1] Dashboard ──── Run Detail     │
  │        │  (Esc)                     │
  │        └────────────────────────────┤
  ├── [2] Run Dialog (popup on Menu)    │
  │        │  (Esc cancels)             │
  │        └────────────────────────────┤
  ├── [3] GPU Monitor                   │
  │        │  (Esc)                     │
  │        └────────────────────────────┤
  ├── [4] Compare                       │
  │        │  (Esc)                     │
  │        └────────────────────────────┤
  ├── [5] Settings                      │
  │        │  (Esc)                     │
  │        └────────────────────────────┘
  └── [q] Quit
```

**Key rule:** `Esc` from any top-level view (Dashboard, GPU, Compare, Settings) returns to Menu. `Esc` from a sub-view (RunDetail) returns to its parent (Dashboard). The Menu is the root.

---

## Files to Create

| File | Purpose |
|------|---------|
| `src/ui/splash.rs` | Splash screen rendering with ASCII art |
| `src/ui/menu.rs` | Main menu rendering with live context |
| `src/ui/settings.rs` | Settings/config overview screen |
| `src/ui/ascii_art.rs` | Shared ASCII art constants (Unicode + ASCII fallback) |

## Files to Modify

| File | Changes |
|------|---------|
| `src/app/state.rs` | Add `View::Splash`, `View::Menu`, `View::Settings` to `View` enum. Add `menu_selected: usize` and `splash_start: Instant` fields to `App`. Add `db_file_size()` method. Initialize `current_view` to `View::Splash`. |
| `src/app/handler.rs` | Add key handling for `View::Splash` (any key → Menu), `View::Menu` (j/k/Enter/1-5/q), `View::Settings` (Esc → back). Update `Back` action so top-level views go to Menu instead of Dashboard. |
| `src/app/actions.rs` | Add `MenuSelect(usize)` action. |
| `src/ui/mod.rs` | Add `pub mod splash; pub mod menu; pub mod settings; pub mod ascii_art;`. Update `render()` match to route `View::Splash`, `View::Menu`, `View::Settings`. |
| `src/main.rs` | Add `--no-splash` CLI flag. Set initial view to `View::Splash` (or `View::Menu` if `--no-splash`). In event loop, handle splash auto-dismiss timer (check if 2s elapsed and transition to Menu). |

## Do NOT Modify

- Export files (`src/export/`)
- Database files (`src/db/`)
- Parser/watcher files (`src/watcher/`)
- Docker manager (`src/docker/`)
- GPU monitor (`src/gpu/`)
- Existing UI components that work correctly

---

## Style Guidelines

- Use the same color scheme as the rest of the app:
  - `Color::Cyan` for highlighted/active items and section headers
  - `Color::DarkGray` for labels and inactive items
  - `Color::White` for values and content
  - `Color::Green` for positive status (✓ detected, running)
  - `Color::Red` for negative status (✗ not found, error)
  - `Color::Yellow` for warnings
  - `Color::Magenta` for the GPU badge (matches gpu_bar)
- The `▶` cursor on menu items uses `Color::Cyan` + `Modifier::BOLD`.
- Box-drawing characters (`╔═╗║╚╝╠╣`) use `Color::Cyan`.
- The ASCII art text uses `Color::Cyan`.
- The version string uses `Color::DarkGray`.
- Center content both vertically and horizontally in the terminal for splash and menu.

## Testing

After implementation:
1. `cargo build` — must compile with zero errors.
2. `cargo run -- --seed` — should show splash → menu → navigate all options.
3. `cargo run -- --no-splash --seed` — should skip splash, go directly to menu.
4. Verify `Esc` from Dashboard returns to Menu (not quit).
5. Verify all 5 menu items work (Dashboard, Run Experiment, GPU, Compare, Settings).
6. Verify splash auto-dismisses after 2 seconds without input.
7. Verify splash dismisses immediately on any keypress.
8. Verify the GPU temperature updates live on the menu screen.
9. Verify the menu shows correct run count and Docker/GPU status.
10. Verify Settings screen displays all config sections.

## Important Notes

- The `platform::supports_unicode()` function already exists in `src/platform.rs`. Use it to choose between Unicode and ASCII art.
- The app version should come from `env!("CARGO_PKG_VERSION")` so it stays in sync with `Cargo.toml`.
- Keep the splash ASCII art as `const &str` arrays in `ascii_art.rs` so they can be reused in both splash and menu screens.
- The menu item for "Run Experiment" should open the RunDialog popup while staying on the Menu view (the popup overlay system already handles this).
- When the run dialog is open from the menu, `InputMode::RunDialog` takes over key handling as it already does.
- GPU polling (`app.poll_gpu_if_needed()`) must continue on Splash, Menu, and Settings screens — it runs in the main event loop regardless of view.

