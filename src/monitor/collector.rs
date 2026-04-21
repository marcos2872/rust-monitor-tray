use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use sysinfo::{Components, DiskRefreshKind, Disks, Networks, ProcessRefreshKind, ProcessesToUpdate, System};

use super::hwmon::{collect_hwmon_metrics_from_path, HWMON_BASE_PATH};
use super::{
    CpuMetrics, DiskInfo, DiskMetrics, GpuInfo, MemoryMetrics, NetworkInterface,
    NetworkMetrics, ProcessInfo, SensorMetrics, SystemInfo, SystemMetrics, TemperatureSensor,
};

const BYTES_TO_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const SECTOR_BYTES: f64 = 512.0;
const TOP_PROCESSES: usize = 15;
/// Mede latência a cada N ciclos (~10s com sampleInterval de 1500ms).
const LATENCY_INTERVAL_CYCLES: u32 = 7;
/// Atualiza GPU com menor frequência para evitar scan de DRM e `nvidia-smi` em todo ciclo.
const GPU_INTERVAL_CYCLES: u32 = 3;
/// Atualiza sensores com menor frequência, sem perder responsividade percebida.
const SENSOR_INTERVAL_CYCLES: u32 = 2;
/// Atualiza processos com menor frequência, reduzindo custo de `/proc/<pid>`.
const PROCESS_INTERVAL_CYCLES: u32 = 2;
/// Frequência muda pouco; não precisa ser atualizada em todo tick.
const CPU_FREQUENCY_INTERVAL_CYCLES: u32 = 10;

pub(crate) fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

// ---------------------------------------------------------------------------
// /proc/stat
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct CpuStatRaw {
    pub(crate) user: u64, pub(crate) nice: u64, pub(crate) system: u64,
    pub(crate) idle: u64, pub(crate) iowait: u64, pub(crate) irq: u64,
    pub(crate) softirq: u64, pub(crate) steal: u64,
}

impl CpuStatRaw {
    fn total_idle(&self)   -> u64 { self.idle + self.iowait }
    fn total_system(&self) -> u64 { self.system + self.irq + self.softirq }
    fn total_user(&self)   -> u64 { self.user + self.nice }
}

fn read_cpu_stat_raw() -> Option<CpuStatRaw> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    for line in content.lines() {
        if !line.starts_with("cpu ") { continue; }
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() < 5 { return None; }
        return Some(CpuStatRaw {
            user: p.get(1).and_then(|v| v.parse().ok()).unwrap_or(0),
            nice: p.get(2).and_then(|v| v.parse().ok()).unwrap_or(0),
            system: p.get(3).and_then(|v| v.parse().ok()).unwrap_or(0),
            idle: p.get(4).and_then(|v| v.parse().ok()).unwrap_or(0),
            iowait: p.get(5).and_then(|v| v.parse().ok()).unwrap_or(0),
            irq: p.get(6).and_then(|v| v.parse().ok()).unwrap_or(0),
            softirq: p.get(7).and_then(|v| v.parse().ok()).unwrap_or(0),
            steal: p.get(8).and_then(|v| v.parse().ok()).unwrap_or(0),
        });
    }
    None
}

fn compute_cpu_percents(prev: &CpuStatRaw, curr: &CpuStatRaw) -> (f32, f32, f32, f32) {
    let d_user   = curr.total_user().saturating_sub(prev.total_user());
    let d_system = curr.total_system().saturating_sub(prev.total_system());
    let d_idle   = curr.total_idle().saturating_sub(prev.total_idle());
    let d_steal  = curr.steal.saturating_sub(prev.steal);
    let d_total  = d_user + d_system + d_idle + d_steal;
    if d_total == 0 { return (0.0, 0.0, 100.0, 0.0); }
    let s = 100.0 / d_total as f32;
    ((d_user as f32 * s).min(100.0), (d_system as f32 * s).min(100.0),
     (d_idle  as f32 * s).min(100.0), (d_steal  as f32 * s).min(100.0))
}

// ---------------------------------------------------------------------------
// /proc/diskstats
// ---------------------------------------------------------------------------

fn read_diskstats() -> HashMap<String, (u64, u64)> {
    let mut r = HashMap::new();
    let Ok(c) = std::fs::read_to_string("/proc/diskstats") else { return r; };
    for line in c.lines() {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() < 10 { continue; }
        r.insert(p[2].to_string(), (p[5].parse().unwrap_or(0), p[9].parse().unwrap_or(0)));
    }
    r
}

fn compute_disk_io_rates(
    before: &HashMap<String, (u64, u64)>,
    after:  &HashMap<String, (u64, u64)>,
    elapsed_secs: f64,
) -> (HashMap<String, u64>, HashMap<String, u64>) {
    let mut read_rates  = HashMap::new();
    let mut write_rates = HashMap::new();
    for (name, (sr_a, sw_a)) in after {
        if let Some((sr_b, sw_b)) = before.get(name) {
            let dr = sr_a.saturating_sub(*sr_b);
            let dw = sw_a.saturating_sub(*sw_b);
            read_rates.insert(name.clone(),  (dr as f64 * SECTOR_BYTES / elapsed_secs).round() as u64);
            write_rates.insert(name.clone(), (dw as f64 * SECTOR_BYTES / elapsed_secs).round() as u64);
        }
    }
    (read_rates, write_rates)
}

fn device_basename(path: &std::ffi::OsStr) -> String {
    Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("").to_string()
}

fn build_disk_info(disk: &sysinfo::Disk, read_rate: u64, write_rate: u64) -> DiskInfo {
    let total_space     = bytes_to_gb(disk.total_space());
    let available_space = bytes_to_gb(disk.available_space());
    let used_space      = total_space - available_space;
    let usage_percent   = if total_space > 0.0 { (used_space as f32 / total_space as f32) * 100.0 } else { 0.0 };
    DiskInfo {
        name: disk.name().to_str().unwrap_or("Unknown").to_string(),
        mount_point: disk.mount_point().to_string_lossy().to_string(),
        total_space, available_space, used_space, usage_percent,
        read_bytes_per_sec: read_rate, write_bytes_per_sec: write_rate,
    }
}

// ---------------------------------------------------------------------------
// /sys/class/net
// ---------------------------------------------------------------------------

fn read_interface_operstate(name: &str) -> bool {
    std::fs::read_to_string(format!("/sys/class/net/{name}/operstate"))
        .map(|s| matches!(s.trim(), "up" | "unknown"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Latência — gateway + ping com timeout
// ---------------------------------------------------------------------------

/// Lê o IP do gateway padrão de /proc/net/route (hex little-endian).
fn read_default_gateway() -> Option<String> {
    let content = std::fs::read_to_string("/proc/net/route").ok()?;
    for line in content.lines().skip(1) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 3 || f[1] != "00000000" { continue; }
        let gw = u32::from_str_radix(f[2], 16).ok()?;
        return Some(format!("{}.{}.{}.{}", gw & 0xFF, (gw >> 8) & 0xFF, (gw >> 16) & 0xFF, (gw >> 24) & 0xFF));
    }
    None
}

/// Executa `ping -c1 -W1 <host>` com timeout total de 1500 ms.
async fn ping_host(host: &str) -> Option<f32> {
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(1500),
        tokio::process::Command::new("ping").args(["-c1", "-W1", host]).output(),
    ).await;
    let output = result.ok()?.ok()?;
    if !output.status.success() { return None; }
    let text = String::from_utf8_lossy(&output.stdout);
    for part in text.split_whitespace() {
        if let Some(ms) = part.strip_prefix("time=") {
            return ms.trim_end_matches("ms").trim().parse().ok();
        }
    }
    None
}

async fn measure_gateway_latency() -> (Option<String>, Option<f32>) {
    if let Some(gw) = read_default_gateway() {
        let latency = ping_host(&gw).await;
        (Some(gw), latency)
    } else {
        (None, None)
    }
}

fn process_refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .without_tasks()
}

fn should_refresh_every(counter: &mut u32, interval_cycles: u32) -> bool {
    if interval_cycles <= 1 {
        return true;
    }
    if *counter + 1 >= interval_cycles {
        *counter = 0;
        true
    } else {
        *counter += 1;
        false
    }
}


// ---------------------------------------------------------------------------
// SystemMonitor
// ---------------------------------------------------------------------------

pub struct SystemMonitor {
    pub(crate) system: System,
    pub(crate) disks: Disks,
    pub(crate) networks: Networks,
    pub(crate) components: Components,
    pub(crate) cpu_user_percent: f32,
    pub(crate) cpu_system_percent: f32,
    pub(crate) cpu_idle_percent: f32,
    pub(crate) cpu_steal_percent: f32,
    pub(crate) disk_read_rates: HashMap<String, u64>,
    pub(crate) disk_write_rates: HashMap<String, u64>,
    pub(crate) cached_gpus: Vec<GpuInfo>,
    pub(crate) cached_sensors: Option<SensorMetrics>,
    pub(crate) cached_top_processes: Option<Vec<ProcessInfo>>,
    pub(crate) cached_gateway_ip: Option<String>,
    pub(crate) cached_gateway_latency_ms: Option<f32>,
    pub(crate) latency_cycle: u32,
    pub(crate) gpu_cycle: u32,
    pub(crate) sensor_cycle: u32,
    pub(crate) process_cycle: u32,
    pub(crate) cpu_frequency_cycle: u32,
}

impl Default for SystemMonitor {
    fn default() -> Self { Self::new() }
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            system,
            disks:      Disks::new_with_refreshed_list(),
            networks:   Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            cpu_user_percent: 0.0, cpu_system_percent: 0.0,
            cpu_idle_percent: 0.0, cpu_steal_percent:  0.0,
            disk_read_rates: HashMap::new(), disk_write_rates: HashMap::new(),
            cached_gpus: vec![],
            cached_sensors: None,
            cached_top_processes: None,
            cached_gateway_ip: None, cached_gateway_latency_ms: None,
            latency_cycle: 0,
            gpu_cycle: 0,
            sensor_cycle: 0,
            process_cycle: 0,
            cpu_frequency_cycle: 0,
        }
    }

    /// Atualiza todas as métricas.
    /// O ciclo é dividido entre métricas rápidas (CPU, memória, disco, rede)
    /// e métricas mais lentas (GPU, sensores, processos), reduzindo trabalho recorrente.
    pub async fn update_metrics(&mut self) {
        let refresh_latency = should_refresh_every(&mut self.latency_cycle, LATENCY_INTERVAL_CYCLES);
        let refresh_gpus = self.cached_gpus.is_empty()
            || should_refresh_every(&mut self.gpu_cycle, GPU_INTERVAL_CYCLES);
        let refresh_sensors = self.cached_sensors.is_none()
            || should_refresh_every(&mut self.sensor_cycle, SENSOR_INTERVAL_CYCLES);
        let refresh_processes = self.cached_top_processes.is_none()
            || should_refresh_every(&mut self.process_cycle, PROCESS_INTERVAL_CYCLES);
        let refresh_cpu_frequency = should_refresh_every(
            &mut self.cpu_frequency_cycle,
            CPU_FREQUENCY_INTERVAL_CYCLES,
        );

        let ping_task = if refresh_latency {
            Some(tokio::spawn(measure_gateway_latency()))
        } else {
            None
        };

        let cpu_stat_before = read_cpu_stat_raw();
        let disk_io_before = read_diskstats();
        let process_refresh = process_refresh_kind();

        // Primeira coleta: inicializa o delta de CPU e, quando necessário, de processos.
        self.system.refresh_cpu_usage();
        if refresh_processes {
            self.system.refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh);
        }

        let window_start = Instant::now();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let elapsed_secs = window_start.elapsed().as_secs_f64().max(0.001);

        // Segunda coleta: consolida valores finais do ciclo.
        self.system.refresh_cpu_usage();
        if refresh_cpu_frequency {
            self.system.refresh_cpu_frequency();
        }
        self.system.refresh_memory();
        if refresh_processes {
            self.system.refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh);
            self.cached_top_processes = Some(self.collect_top_processes());
        }

        self.disks.refresh_specifics(false, DiskRefreshKind::nothing().with_storage());
        self.networks.refresh(false);
        if refresh_sensors {
            self.components.refresh(false);
            self.cached_sensors = Some(self.collect_sensor_metrics());
        }

        if let (Some(b), Some(a)) = (cpu_stat_before, read_cpu_stat_raw()) {
            let (user, system, idle, steal) = compute_cpu_percents(&b, &a);
            self.cpu_user_percent = user;
            self.cpu_system_percent = system;
            self.cpu_idle_percent = idle;
            self.cpu_steal_percent = steal;
        }

        let (read_rates, write_rates) =
            compute_disk_io_rates(&disk_io_before, &read_diskstats(), elapsed_secs);
        self.disk_read_rates = read_rates;
        self.disk_write_rates = write_rates;

        if refresh_gpus {
            self.cached_gpus = super::gpu::collect_gpu_metrics().await;
        }

        if let Some(task) = ping_task {
            let (gw_ip, gw_lat) = task.await.unwrap_or((None, None));
            self.cached_gateway_ip = gw_ip;
            self.cached_gateway_latency_ms = gw_lat;
        }
    }

    pub fn get_cpu_metrics(&self) -> CpuMetrics {
        let cpus = self.system.cpus();
        let core_count = cpus.len();
        let per_core_usage: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let total_usage = if core_count > 0 { per_core_usage.iter().sum::<f32>() / core_count as f32 } else { 0.0 };
        CpuMetrics {
            usage_percent: total_usage, user_percent: self.cpu_user_percent,
            system_percent: self.cpu_system_percent, idle_percent: self.cpu_idle_percent,
            steal_percent: self.cpu_steal_percent, core_count, per_core_usage,
            frequency: cpus.first().map(|c| c.frequency()).unwrap_or(0),
            name:      cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
        }
    }

    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        let total_memory     = bytes_to_gb(self.system.total_memory());
        let used_memory      = bytes_to_gb(self.system.used_memory());
        let available_memory = bytes_to_gb(self.system.available_memory());
        let usage_percent = if total_memory > 0.0 { (used_memory as f32 / total_memory as f32) * 100.0 } else { 0.0 };
        MemoryMetrics { total_memory, used_memory, available_memory, usage_percent,
            total_swap: bytes_to_gb(self.system.total_swap()),
            used_swap:  bytes_to_gb(self.system.used_swap()) }
    }

    pub fn get_disk_metrics(&self) -> DiskMetrics {
        let disks: Vec<DiskInfo> = self.disks.iter().map(|disk| {
            let dev = device_basename(disk.name());
            build_disk_info(disk,
                *self.disk_read_rates.get(&dev).unwrap_or(&0),
                *self.disk_write_rates.get(&dev).unwrap_or(&0))
        }).collect();
        DiskMetrics {
            total_space:               disks.iter().map(|d| d.total_space).sum(),
            used_space:                disks.iter().map(|d| d.used_space).sum(),
            available_space:           disks.iter().map(|d| d.available_space).sum(),
            total_read_bytes_per_sec:  disks.iter().map(|d| d.read_bytes_per_sec).sum(),
            total_write_bytes_per_sec: disks.iter().map(|d| d.write_bytes_per_sec).sum(),
            disks,
        }
    }

    pub fn get_network_metrics(&self) -> NetworkMetrics {
        let mut interfaces = HashMap::new();
        let mut total_bytes_received    = 0u64;
        let mut total_bytes_transmitted = 0u64;
        for (name, data) in &self.networks {
            let iface = NetworkInterface {
                bytes_received:      data.total_received(),
                bytes_transmitted:   data.total_transmitted(),
                packets_received:    data.total_packets_received(),
                packets_transmitted: data.total_packets_transmitted(),
                errors_received:     data.total_errors_on_received(),
                errors_transmitted:  data.total_errors_on_transmitted(),
                is_up:               read_interface_operstate(name),
            };
            total_bytes_received    += iface.bytes_received;
            total_bytes_transmitted += iface.bytes_transmitted;
            interfaces.insert(name.clone(), iface);
        }
        NetworkMetrics {
            interfaces, total_bytes_received, total_bytes_transmitted,
            gateway_ip: self.cached_gateway_ip.clone(),
            gateway_latency_ms: self.cached_gateway_latency_ms,
        }
    }

    fn collect_sensor_metrics(&self) -> SensorMetrics {
        let hwmon = collect_hwmon_metrics_from_path(Path::new(HWMON_BASE_PATH));
        let (hwmon_temps, fans, voltages, currents, powers) =
            (hwmon.temperatures, hwmon.fans, hwmon.voltages, hwmon.currents, hwmon.powers);

        let temperatures: Vec<TemperatureSensor> = if !hwmon_temps.is_empty() {
            hwmon_temps
        } else {
            self.components
                .iter()
                .filter_map(|c| {
                    let t = c.temperature()?;
                    if !t.is_finite() {
                        return None;
                    }
                    let label = c.label().trim();
                    Some(TemperatureSensor {
                        label: if label.is_empty() {
                            "Sensor".to_string()
                        } else {
                            label.to_string()
                        },
                        chip: "Sistema".to_string(),
                        temperature_celsius: t,
                        max_celsius: c.max().filter(|v| v.is_finite()),
                        critical_celsius: c.critical().filter(|v| v.is_finite()),
                    })
                })
                .collect()
        };

        let average_temperature_celsius = if temperatures.is_empty() {
            None
        } else {
            Some(
                temperatures
                    .iter()
                    .map(|s| s.temperature_celsius)
                    .sum::<f32>()
                    / temperatures.len() as f32,
            )
        };

        fn max_by_temp<'a>(
            iter: impl Iterator<Item = &'a TemperatureSensor>,
        ) -> Option<&'a TemperatureSensor> {
            iter.max_by(|l, r| {
                l.temperature_celsius
                    .partial_cmp(&r.temperature_celsius)
                    .unwrap_or(Ordering::Equal)
            })
        }

        fn chip_matches(sensor: &TemperatureSensor, chips: &[&str]) -> bool {
            chips.iter().any(|chip| sensor.chip.eq_ignore_ascii_case(chip))
        }

        let hottest = max_by_temp(temperatures.iter());
        let cpu_chips = ["coretemp", "k10temp", "zenpower"];
        let gpu_chips = ["amdgpu", "radeon", "nouveau"];
        let hottest_cpu =
            max_by_temp(temperatures.iter().filter(|s| chip_matches(s, &cpu_chips)));
        let hottest_gpu_chip =
            max_by_temp(temperatures.iter().filter(|s| chip_matches(s, &gpu_chips)));

        let hottest_temperature_celsius = hottest.map(|s| s.temperature_celsius);
        let hottest_label = hottest.map(|s| s.label.clone()).unwrap_or_default();
        let hottest_cpu_celsius = hottest_cpu.map(|s| s.temperature_celsius);
        let hottest_cpu_label = hottest_cpu.map(|s| s.label.clone()).unwrap_or_default();
        let hottest_gpu_celsius = hottest_gpu_chip.map(|s| s.temperature_celsius);
        let hottest_gpu_label = hottest_gpu_chip.map(|s| s.label.clone()).unwrap_or_default();

        SensorMetrics {
            temperatures,
            average_temperature_celsius,
            hottest_temperature_celsius,
            hottest_label,
            hottest_cpu_celsius,
            hottest_cpu_label,
            hottest_gpu_celsius,
            hottest_gpu_label,
            fans,
            voltages,
            currents,
            powers,
        }
    }

    pub fn get_sensor_metrics(&self) -> SensorMetrics {
        self.cached_sensors
            .clone()
            .unwrap_or_else(|| self.collect_sensor_metrics())
    }

    fn collect_top_processes(&self) -> Vec<ProcessInfo> {
        let core_count = self.system.cpus().len().max(1) as f32;
        let mut procs: Vec<ProcessInfo> = self
            .system
            .processes()
            .values()
            .map(|p| ProcessInfo {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_percent: p.cpu_usage() / core_count,
                memory_mb: p.memory() as f64 / (1024.0 * 1024.0),
            })
            .filter(|p| p.cpu_percent > 0.0 || p.memory_mb > 1.0)
            .collect();
        procs.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(Ordering::Equal)
        });
        procs.truncate(TOP_PROCESSES);
        procs
    }

    /// Retorna os TOP_PROCESSES processos com maior uso de CPU,
    /// normalizado pelo número de cores (0-100% do total do sistema).
    pub fn get_top_processes(&self) -> Vec<ProcessInfo> {
        self.cached_top_processes
            .clone()
            .unwrap_or_else(|| self.collect_top_processes())
    }

    pub fn get_all_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu:     self.get_cpu_metrics(),
            memory:  self.get_memory_metrics(),
            disk:    self.get_disk_metrics(),
            network: self.get_network_metrics(),
            sensors: self.get_sensor_metrics(),
            gpus:    self.cached_gpus.clone(),
            top_processes: self.get_top_processes(),
            system_info: SystemInfo {
                hostname:       System::host_name().unwrap_or_else(|| "unknown".to_string()),
                os_name:        System::name().unwrap_or_else(|| "Linux".to_string()),
                os_version:     System::os_version().unwrap_or_default(),
                kernel_version: System::kernel_version().unwrap_or_default(),
                architecture:   System::cpu_arch(),
                process_count:  self.system.processes().len(),
            },
            uptime: System::uptime(),
            load_average: { let la = System::load_average(); (la.one, la.five, la.fifteen) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_device_basename_extrai_nome_do_caminho() {
        assert_eq!(device_basename(OsStr::new("/dev/sda")), "sda");
        assert_eq!(device_basename(OsStr::new("/dev/nvme0n1")), "nvme0n1");
        assert_eq!(device_basename(OsStr::new("")), "");
    }

    #[test]
    fn test_compute_cpu_percents_distribui_corretamente() {
        let prev = CpuStatRaw { user: 0, nice: 0, system: 0, idle: 0, iowait: 0, irq: 0, softirq: 0, steal: 0 };
        let curr = CpuStatRaw { user: 40, nice: 10, system: 20, idle: 30, iowait: 0, irq: 0, softirq: 0, steal: 0 };
        let (user, system, idle, steal) = compute_cpu_percents(&prev, &curr);
        assert!((user - 50.0).abs() < 0.1);
        assert!((system - 20.0).abs() < 0.1);
        assert!((idle - 30.0).abs() < 0.1);
        assert!((steal - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_cpu_percents_retorna_idle_total_sem_delta() {
        let snap = CpuStatRaw { user: 100, nice: 0, system: 50, idle: 200, iowait: 0, irq: 0, softirq: 0, steal: 0 };
        let (_, _, idle, steal) = compute_cpu_percents(&snap, &snap);
        assert_eq!(idle, 100.0);
        assert_eq!(steal, 0.0);
    }

    #[test]
    fn test_compute_cpu_percents_contabiliza_steal() {
        let prev = CpuStatRaw { user: 0, nice: 0, system: 0, idle: 0, iowait: 0, irq: 0, softirq: 0, steal: 0 };
        let curr = CpuStatRaw { user: 25, nice: 0, system: 25, idle: 25, iowait: 0, irq: 0, softirq: 0, steal: 25 };
        let (user, system, idle, steal) = compute_cpu_percents(&prev, &curr);
        assert!((user - 25.0).abs() < 0.1);
        assert!((steal - 25.0).abs() < 0.1);
        let _ = (system, idle);
    }

    #[test]
    fn test_compute_disk_io_rates_converte_setores_para_bytes_por_seg() {
        let mut before = HashMap::new();
        before.insert("sda".to_string(), (1000u64, 500u64));
        let mut after = HashMap::new();
        after.insert("sda".to_string(), (1200u64, 700u64));
        let (read_rates, write_rates) = compute_disk_io_rates(&before, &after, 1.0);
        assert_eq!(*read_rates.get("sda").unwrap(), 102_400);
        assert_eq!(*write_rates.get("sda").unwrap(), 102_400);
    }

    #[test]
    fn test_compute_disk_io_rates_ignora_dispositivos_ausentes_no_before() {
        let before = HashMap::new();
        let mut after = HashMap::new();
        after.insert("nvme0n1".to_string(), (500u64, 200u64));
        let (read_rates, _) = compute_disk_io_rates(&before, &after, 1.0);
        assert!(read_rates.is_empty());
    }
}
