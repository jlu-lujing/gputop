use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::Nvml;

/// GPU process information
#[derive(Debug, Clone)]
pub struct GpuProcess {
    pub pid: u32,
    pub name: String,
    pub gpu_mem_mib: u64,
    pub sm_percent: u32,
    pub cpu_percent: f32,
}

/// GPU data collected from NVML
#[derive(Debug, Clone)]
pub struct GpuData {
    pub index: u32,
    pub name: String,
    pub gpu_utilization: u32,
    #[allow(dead_code)]
    pub mem_utilization: u32,
    pub mem_used_mib: u64,
    pub mem_total_mib: u64,
    pub temperature: u32,
    pub power_draw_watts: u32,
    pub power_limit_watts: u32,
    pub fan_speed: Option<u32>,
    pub graphics_clock: u32,
    pub memory_clock: u32,
    pub processes: Vec<GpuProcess>,
}

fn gpu_mem_to_mib(mem: &nvml_wrapper::enums::device::UsedGpuMemory) -> u64 {
    match mem {
        nvml_wrapper::enums::device::UsedGpuMemory::Used(b) => *b / 1024 / 1024,
        nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => 0,
    }
}

pub fn collect_gpu_data() -> Result<Vec<GpuData>, String> {
    let nvml = Nvml::init().map_err(|e| format!("NVML init failed: {}", e))?;

    let device_count = nvml.device_count().map_err(|e| e.to_string())?;

    let mut gpus = Vec::new();

    for i in 0..device_count {
        let gpu = nvml.device_by_index(i).map_err(|e| e.to_string())?;

        let name = gpu.name().unwrap_or_else(|_| "Unknown".to_string());

        let (gpu_utilization, mem_utilization) = gpu
            .utilization_rates()
            .map(|r| (r.gpu, r.memory))
            .unwrap_or((0, 0));

        let (mem_used_mib, mem_total_mib) = gpu
            .memory_info()
            .map(|m| (m.used / 1024 / 1024, m.total / 1024 / 1024))
            .unwrap_or((0, 0));

        let temperature = gpu.temperature(TemperatureSensor::Gpu).unwrap_or(0);

        let power_draw_watts = gpu.power_usage().map(|w| w as u32 / 1000).unwrap_or(0);
        let power_limit_watts = gpu
            .enforced_power_limit()
            .map(|w| w as u32 / 1000)
            .unwrap_or(0);

        let fan_speed = gpu.fan_speed(0).ok();

        // Clock speeds (MHz)
        let graphics_clock = gpu.clock_info(Clock::Graphics).unwrap_or(0);
        let memory_clock = gpu.clock_info(Clock::Memory).unwrap_or(0);

        // Collect GPU processes (compute + graphics)
        let mut processes = Vec::new();

        // Compute processes
        if let Ok(compute_procs) = gpu.running_compute_processes() {
            for p in &compute_procs {
                let name = nvml
                    .sys_process_name(p.pid, 64)
                    .unwrap_or_else(|_| "<unknown>".to_string());
                processes.push(GpuProcess {
                    pid: p.pid,
                    name,
                    gpu_mem_mib: gpu_mem_to_mib(&p.used_gpu_memory),
                    sm_percent: 0,
                    cpu_percent: 0.0,
                });
            }
        }

        // Graphics processes
        if let Ok(gfx_procs) = gpu.running_graphics_processes() {
            for p in &gfx_procs {
                processes.push(GpuProcess {
                    pid: p.pid,
                    name: "<unknown>".to_string(),
                    gpu_mem_mib: gpu_mem_to_mib(&p.used_gpu_memory),
                    sm_percent: 0,
                    cpu_percent: 0.0,
                });
            }
        }

        // Try to get per-process SM utilization
        if let Ok(util_stats) = gpu.process_utilization_stats(None) {
            for s in &util_stats {
                if let Some(proc) = processes.iter_mut().find(|p| p.pid == s.pid) {
                    proc.sm_percent = s.sm_util;
                }
            }
        }

        gpus.push(GpuData {
            index: i,
            name,
            gpu_utilization,
            mem_utilization,
            mem_used_mib,
            mem_total_mib,
            temperature,
            power_draw_watts,
            power_limit_watts,
            fan_speed,
            graphics_clock,
            memory_clock,
            processes,
        });
    }

    Ok(gpus)
}
