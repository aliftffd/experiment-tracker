/// Large block-letter ASCII art for splash screen
pub const SPLASH_ART: &[&str] = &[
    "███████╗██╗  ██╗██████╗ ████████╗██████╗  █████╗  ██████╗██╗  ██╗",
    "██╔════╝╚██╗██╔╝██╔══██╗╚══██╔══╝██╔══██╗██╔══██╗██╔════╝██║ ██╔╝",
    "█████╗   ╚███╔╝ ██████╔╝   ██║   ██████╔╝███████║██║     █████╔╝ ",
    "██╔══╝   ██╔██╗ ██╔═══╝    ██║   ██╔══██╗██╔══██║██║     ██╔═██╗ ",
    "███████╗██╔╝ ██╗██║        ██║   ██║  ██║██║  ██║╚██████╗██║  ╚██╗",
    "╚══════╝╚═╝  ╚═╝╚═╝        ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝",
];

/// Width of the large splash art (for centering)
pub const SPLASH_ART_WIDTH: u16 = 67;

/// ASCII fallback for terminals without Unicode
pub const SPLASH_ART_ASCII: &[&str] = &[
    "  _____ _  _ ___ _____ ___    _   ___ _  __",
    " | __\\ \\/ /| _ |_   _| _ \\  /_\\ / __| |/ /",
    " | _| >  < |  _/ | | |   / / _ \\ (__| ' < ",
    " |___/_/\\_\\|_|   |_| |_|_\\/_/ \\_\\___|_|\\_\\",
];

pub const SPLASH_ART_ASCII_WIDTH: u16 = 46;

/// Compact art for menu header
pub const MENU_ART: &[&str] = &[
    "╔═══════════════════════════════════════════════╗",
    "║                                               ║",
    "║   ▄▄▄ ▄   ▄ ▄▄▄  ▄▄▄▄ ▄▄▄  ▄▄▄  ▄▄▄ ▄  ▄  ║",
    "║   █    ▀▄▀  █▄▄█   █   █▄▄▀ █▄▄█ █   █▄▄▀   ║",
    "║   ▀▀▀ ▀ ▀  █      ▀   ▀  ▀ ▀  ▀ ▀▀▀ ▀  ▀   ║",
    "║                                               ║",
];

pub const MENU_ART_ASCII: &[&str] = &[
    "+-----------------------------------------------+",
    "|                                               |",
    "|   EXPTRACK                                    |",
    "|                                               |",
];

pub const MENU_ART_WIDTH: u16 = 49;

pub const MENU_SEPARATOR_UNICODE: &str = "╠═══════════════════════════════════════════════╣";
pub const MENU_SEPARATOR_ASCII: &str   = "+-----------------------------------------------+";
pub const MENU_BOTTOM_UNICODE: &str    = "╚═══════════════════════════════════════════════╝";
pub const MENU_BOTTOM_ASCII: &str      = "+-----------------------------------------------+";

/// Get version string
pub fn version_string() -> String {
    format!("ML Experiment Tracker v{}", env!("CARGO_PKG_VERSION"))
}
