use std::time::Duration;
use sysinfo::System;

/// Per-core CPU utilization percentage
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CpuCoreData {
    pub index: usize,
    pub usage_percent: f32,
    pub frequency_mhz: u64,
}

/// Memory breakdown (btop style: used / buffers / cached / free)
#[derive(Debug, Clone)]
pub struct MemBreakdown {
    pub total_mib: u64,
    pub used_mib: u64,
    pub buffers_mib: u64,
    pub cached_mib: u64,
    pub free_mib: u64,
}

/// CPU and memory data (swap fields for later use)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CpuData {
    pub cores: Vec<CpuCoreData>,
    pub total_usage: f32,
    pub cpu_temp_celsius: u32,
    pub mem: MemBreakdown,
    pub swap_total_mib: u64,
    pub swap_used_mib: u64,
}

impl Default for CpuData {
    fn default() -> Self {
        Self {
            cores: Vec::new(),
            total_usage: 0.0,
            cpu_temp_celsius: 0,
            mem: MemBreakdown::default(),
            swap_total_mib: 0,
            swap_used_mib: 0,
        }
    }
}

impl Default for MemBreakdown {
    fn default() -> Self {
        Self {
            total_mib: 0,
            used_mib: 0,
            buffers_mib: 0,
            cached_mib: 0,
            free_mib: 0,
        }
    }
}

/// Read CPU temperature from /sys/class/hwmon.
/// Scans all hwmon devices and matches by label:
///   Tctl > CPUTIN > Composite > SYSTIN > first temp*_input
/// Returns 0 if no sensor is found.
fn read_cpu_temp() -> u32 {
    let hwmon = "/sys/class/hwmon";
    let entries: Vec<_> = std::fs::read_dir(hwmon)
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    // Priority-ordered label prefixes to match
    let priorities = ["Tctl", "CPUTIN", "Composite", "SYSTIN"];

    for label in &priorities {
        for entry in &entries {
            let dir_path = entry.path();
            let temp_entries: Vec<_> = std::fs::read_dir(&dir_path)
                .into_iter()
                .flatten()
                .flatten()
                .collect();
            for temp_entry in &temp_entries {
                let name = temp_entry.file_name().to_string_lossy().to_string();
                if !name.starts_with("temp") || !name.ends_with("_input") {
                    continue;
                }
                // Check if this hwmon has a matching label
                // temp1_input → temp1_label (replace _input with _label)
                let label_name = name.replace("_input", "_label");
                let label_file = dir_path.join(&label_name);
                if let Ok(l) = std::fs::read_to_string(&label_file) {
                    if l.trim() == *label {
                        let path = temp_entry.path();
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(millidegrees) = content.trim().parse::<u64>() {
                                if millidegrees > 0 {
                                    return (millidegrees / 1000) as u32;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: first non-zero temp*_input
    for entry in &entries {
        let dir_path = entry.path();
        let temp_entries: Vec<_> = std::fs::read_dir(&dir_path)
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        for temp_entry in &temp_entries {
            let name = temp_entry.file_name().to_string_lossy().to_string();
            if name.starts_with("temp") && name.ends_with("_input") {
                if let Ok(content) = std::fs::read_to_string(&temp_entry.path()) {
                    if let Ok(millidegrees) = content.trim().parse::<u64>() {
                        if millidegrees > 0 {
                            return (millidegrees / 1000) as u32;
                        }
                    }
                }
            }
        }
    }
    0
}

/// Parse /proc/meminfo for memory breakdown (btop style)
fn read_meminfo() -> MemBreakdown {
    let mut total = 0u64;
    let mut free = 0u64;
    let mut buffers = 0u64;
    let mut cached = 0u64;

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let key = parts[0].trim_end_matches(':');
            let val_kb: u64 = parts[1].parse().unwrap_or(0);
            let val_mib = val_kb / 1024;

            match key {
                "MemTotal" => total = val_mib,
                "MemFree" => free = val_mib,
                "Buffers" => buffers = val_mib,
                "Cached" => cached = val_mib,
                _ => {}
            }
        }
    }

    let used = total.saturating_sub(free + buffers + cached);
    MemBreakdown {
        total_mib: total,
        used_mib: used,
        buffers_mib: buffers,
        cached_mib: cached,
        free_mib: free,
    }
}

/// Persistent CPU collector — keeps a sysinfo::System instance so
/// consecutive refresh_cpu_all() calls produce real utilization numbers.
pub struct CpuCollector {
    sys: System,
}

impl CpuCollector {
    /// Initialize and do the first (warmup) read so the next call has real numbers.
    pub fn new() -> Self {
        let mut collector = Self {
            sys: System::new_all(),
        };
        // Warmup: first read always gives 0%, so read once now.
        collector.sys.refresh_cpu_all();
        collector.sys.refresh_cpu_frequency();
        collector
    }

    /// Collect CPU + memory data (call after at least 1s since last call).
    pub fn collect(&mut self) -> Result<CpuData, String> {
        self.sys.refresh_cpu_all();
        self.sys.refresh_cpu_frequency();
        self.sys.refresh_memory();

        let cores: Vec<CpuCoreData> = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| CpuCoreData {
                index: i,
                usage_percent: cpu.cpu_usage(),
                frequency_mhz: cpu.frequency(),
            })
            .collect();

        let total_usage = self.sys.global_cpu_usage();
        let cpu_temp_celsius = read_cpu_temp();
        let mem = read_meminfo();
        let swap_total_mib = self.sys.total_swap() / 1024 / 1024;
        let swap_used_mib = (self.sys.total_swap() - self.sys.free_swap()) / 1024 / 1024;

        Ok(CpuData {
            cores,
            total_usage,
            cpu_temp_celsius,
            mem,
            swap_total_mib,
            swap_used_mib,
        })
    }
}

/// Format uptime as "Dd Hh Mm"
pub fn format_uptime() -> String {
    let uptime_secs = System::uptime();
    let dur = Duration::from_secs(uptime_secs);
    let secs = dur.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Format current time as "HH:MM:SS"
pub fn format_time() -> String {
    use chrono::Local;
    Local::now().format("%H:%M:%S").to_string()
}
