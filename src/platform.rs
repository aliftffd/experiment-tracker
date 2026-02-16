use std::path::PathBuf;

/// Detecting an OS
#[derive(Debug, Clone, PartialEq)]
pub enum OsType {
    Linux,
    Windows,
    MacOs,
    Unknown,
}

/// Detected current OS
pub fn detect_os() -> OsType {
    if cfg!(target_os = "linux") {
        OsType::Linux
    } else if cfg!(target_os = "windows") {
        OsType::Windows
    } else if cfg!(target_os = "macos") {
        OsType::MacOs
    } else {
        OsType::Unknown
    }
}

/// Get the platform-specific data directory
/// Linux:   ~/.local/share/experiment-tracker/
/// Windows: C:\Users\<user>\AppData\Local\experiment-tracker\
/// macOS:   ~/Library/Application Support/experiment-tracker/

pub fn data_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = home_dir();
                home.join("AppData").join("Local")
            })
    } else if cfg!(target_os = "macos") {
        home_dir().join("Library").join("Application Support")
    } else {
        // linux : respect XDG
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir().join(".local").join("share"))
    };

    base.join("experiment-tracker")
}

/// Get the platform-specific config directory
/// Linux:   ~/.config/experiment-tracker/
/// Windows: C:\Users\<user>\AppData\Local\experiment-tracker\
/// macOS:   ~/Library/Application Support/experiment-tracker/

pub fn config_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = home_dir();
                home.join("AppData").join("Local")
            })
    } else if cfg!(target_os = "macos") {
        home_dir().join("Library").join("Application Support")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir().join(".config"))
    };

    base.join("experiment-tracker")
}

/// get the default database path
pub fn default_db_path() -> PathBuf {
    data_dir().join("tracker.db")
}

/// get the default config file path
pub fn default_config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// get home directory cross platform
pub fn home_dir() -> PathBuf {
    /// Try home first (LINUX/macOS), then USERPROFILE (WINDOWS)
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                PathBuf::from("C:\\Users\\Default")
            } else {
                PathBuf::from("/tmp")
            }
        })
}

/// expand ~ in paths, cross-platform
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") || path.starts_with("~\\") {
        home_dir().join(&path[2..])
    } else if path == "~" {
        home_dir()
    } else {
        PathBuf::from(path)
    }
}

/// find nvidia-smi binary path both OS (LINUX and Windows)
pub fn find_nvidia_smi() -> Option<PathBuf> {
    // find the first PATH (works on both platforms if configured)
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            return Some(PathBuf::from("nvidia-smi"));
        }
    }

    // windows search common location
    if cfg!(target_os = "windows") {
        let candidates = [
            r"C:\Windows\System32\nvidia-smi.exe",
            r"C:\Program Files\NVIDIA Corporation\NVSMI\nvidia-smi.exe",
        ];

        for path in &candidates {
            let p = PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }
    }

    None
}

/// find docker binary
pub fn find_docker() -> Option<PathBuf> {
    if let Ok(output) = std::process::Command::new("docker")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            return Some(PathBuf::from("docker"));
        }
    }

    // windows: Docker desktop location
    if cfg!(target_os = "windows") {
        let p = PathBuf::from(r"C:\Program Files\Docker\Docker\resources\bin\docker.exe");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Convert a host path to a docker compatiuble volume mount path
pub fn to_docker_path(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy().to_string();

    if cfg!(target_os = "windows") {
        // COnvert backlashes to forward shlases
        let path_str = path_str.replace('\\', "/tmp/");

        // Convert C:/ to /c/
        if path_str.len() >= 2 && path_str.as_bytes()[1] == b':' {
            let drive = path_str.as_bytes()[0].to_ascii_lowercase() as char;
            return format!("/{}{}", drive, &path_str[2..]);
        }

        path_str
    } else {
        path_str
    }
}

/// check if the terminal support unicode
pub fn supports_unicode() -> bool {
    if cfg!(target_os = "windows") {
        // Windows Terminal and pwsh support Unicode
        // cmd.exe might not
        std::env::var("WT_SESSION").is_ok()  // Windows Terminal sets this
            || std::env::var("TERM_PROGRAM").is_ok()
    } else {
        // Most Linux/macOS terminals support Unicode
        true
    }
}
