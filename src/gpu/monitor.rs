use anyhow::Result;
use std::process::Command;
use std::time::Instant;

use crate::platform;

/// GPU statistics from a sinle poll
#[derive(Debug, Clone)]
pub struct GpuStats {
    pub utilization_percent: u32,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub temperature_celsius: u32,
    pub power_draw_watts: f32,
    pub power_limit_watts: f32,
    pub clock_speed_mhz: u32,
    pub fan_speed_percent: Option<u32>, // None on laptops
    pub gpu_name: String,
    pub driver_version: String,
    pub timestamp: Instant,
}

impl GpuStats{
    /// Vram usage as "%"
    pub fn vram_percent(&self) -> f32 {
        if self.vram_total_mb == 0 {
            return 0.0;
        }
        (self.vram_used_mb as f32 / self.vram_total_mb as f32) * 100.0
    }

    /// poser usage as percentage of limit
    pub fn power_percent(&self) -> f32 {
        if self.power_limit_watts <= 0.0 {
            return 0.0;
        }
        (self.power_draw_watts / self.power_limit_watts) * 100.0
    }
    /// is CRAM cratically full ?
    pub fn vram_critical(&self, threshold: f32) -> bool {
        self.vram_percent() > threshold
    }

    /// is GPU overheating
    pub fn temp_critical(&self, threshold: u32) -> bool {
        self.temperature_celsius > threshold
    }
}

/// A process using the GPU
#[derive(Debug, Clone)]
pub struct GpuProcess {
    pub pid: u32,
    pub name: String,
    pub vram_used_mb: u64,
}

/// ring buffer for GPU history
pub struct GpuHistory {
    entries: Vec<GpuStats>,
    capacity: usize,
    write_index: usize,
}

impl GpuHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
            write_index: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn push(&mut self, stats: GpuStats) {
        if self.entries.len() < self.capacity {
            self.entries.push(stats);
        } else {
            self.entries[self.write_index] = stats;
        }
        self.write_index = (self.write_index + 1) % self.capacity;
    }

    /// get entries in chronological order
    pub fn ordered(&self) -> Vec<&GpuStats> {
        if self.entries.len() < self.capacity {
            // buffer not full yet -- entries are already in order
            self.entries.iter().collect()
        } else {
            // buffer is full - read from write_index (olders) to write_index-1 (newest)
            let mut result = Vec::with_capacity(self.capacity);
            for i in 0..self.capacity {
                let ix = (self.write_index + i) % self.capacity;
                result.push(&self.entries[ix]);
            }
            result
        }
    }

    // get utitilization for charting
    pub fn utilization_series(&self) -> Vec<f64> {
        self.ordered()
            .iter()
            .map(|s| s.utilization_percent as f64)
            .collect()
    }

    // get VRAM usage
    pub fn vram_series(&self) -> Vec<f64> {
        self.ordered()
            .iter()
            .map(|s| s.vram_percent() as f64)
            .collect()
    }

    // get temperature
    pub fn temp_series(&self) -> Vec<f64> {
        self.ordered()
            .iter()
            .map(|s| s.temperature_celsius as f64)
            .collect()
    }
}

/// The GPU monitor - pools nvidia-smi and parses results
pub struct GpuMonitor {
    nvidia_smi_path : String,
}

impl GpuMonitor {
    ///.try to create a GPU monitor. returns non if no GPU is found
    pub fn new() -> Option<Self> {
        let path = platform::find_nvidia_smi()?;
        let path_str = path.to_string_lossy().to_string();

        // verify it actually works
        let output = Command::new(&path_str)
            .args(["--query-gpu=name", "--format=csv,noheader"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(Self{
                nvidia_smi_path: path_str,
            })
        } else{
            None
        }
    }

    // pool current GPU statisitcs
    pub fn poll_stats(&self) -> Result<GpuStats> {
        let output = Command::new(&self.nvidia_smi_path)
            .args([
                "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,power.limit,clocks.current.sm,fan.speed,name,driver_version",
                "--format=csv,noheader,nounits",
            ])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();

        if line.is_empty() {
            anyhow::bail!("nvidia-smi returned empty output")
        }

        let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();

        if fields.len() < 10 {
            anyhow::bail!(
                "nvidia-smi returned {} fields, expected 10: {}",
                fields.len(),
                line
                );
        }

        Ok(GpuStats {
            utilization_percent: parse_u32(fields[0]),
            vram_used_mb: parse_u64(fields[1]),
            vram_total_mb: parse_u64(fields[2]),
            temperature_celsius: parse_u32(fields[3]),
            power_draw_watts: parse_f32(fields[4]),
            power_limit_watts: parse_f32(fields[5]),
            clock_speed_mhz: parse_u32(fields[6]),
            fan_speed_percent: parse_optional_u32(fields[7]),
            gpu_name: fields[8].to_string(),
            driver_version: fields[9].to_string(),
            timestamp: Instant::now(),
        })
    }

    /// poll GPU processed (whats using VRAM)
    pub fn poll_processes(&self) -> Result<Vec<GpuProcess>> {
        let output = Command::new(&self.nvidia_smi_path)
            .args([
                "--query-compute-apps=pid,process_name,used_gpu_memory",
                "--format=csv,noheader,nounits",
            ])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.contains("No Running") {
                continue;
            }

            let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();
            if fields.len() >= 3 {
                processes.push(GpuProcess {
                    pid: parse_u32(fields[0]),
                    name: fields[1].to_string(),
                    vram_used_mb: parse_u64(fields[2]),
                });
            }
        }

        Ok(processes)
    }

}

// ─── Parsing helpers ──────────────────────────────

fn parse_u32(s: &str) -> u32 {
    s.trim().parse::<f64>().unwrap_or(0.0) as u32
}

fn parse_u64(s: &str) -> u64 {
    s.trim().parse::<f64>().unwrap_or(0.0) as u64
}

fn parse_f32(s: &str) -> f32 {
    s.trim().parse::<f32>().unwrap_or(0.0)
}

fn parse_optional_u32(s: &str) -> Option<u32> {
    let s = s.trim();
    if s == "[N/A]" || s == "N/A" || s.is_empty() {
        None
    } else {
        Some(parse_u32(s))
    }
}
