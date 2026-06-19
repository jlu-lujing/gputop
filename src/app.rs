use crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;

use crate::cpu::{CpuCollector, CpuData};
use crate::event::handle_key;
use crate::gpu::GpuData;
use crate::gpu::GpuProcess;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    GpuMem,
    SmPercent,
    CpuPercent,
}

impl Default for SortColumn {
    fn default() -> Self {
        Self::SmPercent
    }
}

/// Maximum history points for sparklines (90 seconds)
const SPARKLINE_LEN: usize = 90;

pub struct App {
    pub running: bool,
    pub gpu_data: Vec<GpuData>,
    pub cpu_data: CpuData,
    pub processes: Vec<GpuProcess>,
    pub sort_column: SortColumn,
    pub selected_process: Option<usize>,
    pub refresh_interval: Duration,
    pub last_refresh: std::time::Instant,
    pub error: Option<String>,
    /// History of GPU utilization per GPU (for sparklines)
    pub gpu_util_history: Vec<Vec<u64>>,
    /// Persistent CPU collector (keeps sysinfo::System alive for real CPU%)
    cpu_collector: CpuCollector,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            gpu_data: Vec::new(),
            cpu_data: CpuData::default(),
            processes: Vec::new(),
            sort_column: SortColumn::default(),
            selected_process: None,
            refresh_interval: Duration::from_secs(1),
            last_refresh: std::time::Instant::now(),
            error: None,
            gpu_util_history: Vec::new(),
            cpu_collector: CpuCollector::new(),
        }
    }

    pub fn refresh_data(&mut self) {
        let prev_gpu_count = self.gpu_data.len();

        // Refresh GPU data
        match crate::gpu::collect_gpu_data() {
            Ok(data) => {
                // Track history for sparklines
                if data.len() != prev_gpu_count {
                    self.gpu_util_history = data
                        .iter()
                        .map(|g| vec![g.gpu_utilization as u64])
                        .collect();
                } else {
                    for (i, gpu) in data.iter().enumerate() {
                        if i < self.gpu_util_history.len() {
                            let hist = &mut self.gpu_util_history[i];
                            hist.push(gpu.gpu_utilization as u64);
                            if hist.len() > SPARKLINE_LEN {
                                hist.remove(0);
                            }
                        } else {
                            self.gpu_util_history.push(vec![gpu.gpu_utilization as u64]);
                        }
                    }
                }
                self.gpu_data = data;
            }
            Err(e) => {
                self.error = Some(format!("GPU: {}", e));
            }
        }

        // Collect GPU processes from all GPUs
        self.processes = self
            .gpu_data
            .iter()
            .flat_map(|gpu| gpu.processes.clone())
            .collect();

        // Sort processes
        self.sort_processes();

        // Refresh CPU data using persistent collector
        match self.cpu_collector.collect() {
            Ok(data) => {
                self.cpu_data = data;
            }
            Err(e) => {
                self.error = Some(format!("CPU: {}", e));
            }
        }

        self.last_refresh = std::time::Instant::now();
    }

    fn sort_processes(&mut self) {
        match self.sort_column {
            SortColumn::Pid => self.processes.sort_by(|a, b| a.pid.cmp(&b.pid)),
            SortColumn::GpuMem => self
                .processes
                .sort_by(|a, b| b.gpu_mem_mib.cmp(&a.gpu_mem_mib)),
            SortColumn::SmPercent => self
                .processes
                .sort_by(|a, b| b.sm_percent.cmp(&a.sm_percent)),
            SortColumn::CpuPercent => self.processes.sort_by(|a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
        }
    }

    pub fn handle_events(&mut self) {
        if !event::poll(Duration::from_millis(100)).unwrap_or(true) {
            return;
        }

        let event = match event::read() {
            Ok(e) => e,
            Err(_) => return,
        };
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }
            handle_key(self, key.code);
        }
    }
}
