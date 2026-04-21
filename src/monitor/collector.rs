use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

use sysinfo::{Components, Disks, Networks, System};

use super::hwmon::{collect_hwmon_metrics_from_path, HWMON_BASE_PATH};
use super::{
    CpuMetrics, DiskInfo, DiskMetrics, MemoryMetrics, NetworkInterface, NetworkMetrics,
    SensorMetrics, SystemMetrics, TemperatureSensor,
};

const BYTES_TO_GB: f64 = 1024.0 * 1024.0 * 1024.0;
/// 1 setor = 512 bytes; janela de medição = 200 ms → ×5 para bytes/seg.
const SECTOR_BYTES: u64 = 512;
const IO_INTERVALS_PER_SEC: u64 = 5;

pub(crate) fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

// ---------------------------------------------------------------------------
// Leitura de /proc/stat para breakdown user/system/idle da CPU
// ---------------------------------------------------------------------------

/// Snapshot cru da linha "cpu" de /proc/stat.
#[derive(Clone)]
pub(crate) struct CpuStatRaw {
    pub(crate) user: u64,
    pub(crate) nice: u64,
    pub(crate) system: u64,
    pub(crate) idle: u64,
    pub(crate) iowait: u64,
    pub(crate) irq: u64,
    pub(crate) softirq: u64,
    pub(crate) steal: u64,
}

impl CpuStatRaw {
    fn total_idle(&self) -> u64 {
        self.idle + self.iowait
    }

    fn total_system(&self) -> u64 {
        self.system + self.irq + self.softirq
    }

    fn total_user(&self) -> u64 {
        self.user + self.nice
    }
}

fn read_cpu_stat_raw() -> Option<CpuStatRaw> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    for line in content.lines() {
        if !line.starts_with("cpu ") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return None;
        }
        return Some(CpuStatRaw {
            user:    parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0),
            nice:    parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(0),
            system:  parts.get(3).and_then(|v| v.parse().ok()).unwrap_or(0),
            idle:    parts.get(4).and_then(|v| v.parse().ok()).unwrap_or(0),
            iowait:  parts.get(5).and_then(|v| v.parse().ok()).unwrap_or(0),
            irq:     parts.get(6).and_then(|v| v.parse().ok()).unwrap_or(0),
            softirq: parts.get(7).and_then(|v| v.parse().ok()).unwrap_or(0),
            steal:   parts.get(8).and_then(|v| v.parse().ok()).unwrap_or(0),
        });
    }
    None
}

/// Calcula (user%, system%, idle%) a partir de dois snapshots de /proc/stat.
fn compute_cpu_percents(prev: &CpuStatRaw, curr: &CpuStatRaw) -> (f32, f32, f32) {
    let d_user   = curr.total_user().saturating_sub(prev.total_user());
    let d_system = curr.total_system().saturating_sub(prev.total_system());
    let d_idle   = curr.total_idle().saturating_sub(prev.total_idle());
    let d_steal  = curr.steal.saturating_sub(prev.steal);
    let d_total  = d_user + d_system + d_idle + d_steal;

    if d_total == 0 {
        return (0.0, 0.0, 100.0);
    }

    let scale = 100.0 / d_total as f32;
    (
        (d_user   as f32 * scale).min(100.0),
        (d_system as f32 * scale).min(100.0),
        (d_idle   as f32 * scale).min(100.0),
    )
}

// ---------------------------------------------------------------------------
// Leitura de /proc/diskstats para taxas de I/O de disco
// ---------------------------------------------------------------------------

/// Retorna mapa: nome do dispositivo → (setores lidos, setores escritos) cumulativos.
fn read_diskstats() -> HashMap<String, (u64, u64)> {
    let mut result = HashMap::new();
    let content = match std::fs::read_to_string("/proc/diskstats") {
        Ok(c) => c,
        Err(_) => return result,
    };
    for line in content.lines() {
        // Formato: major minor nome col1..col11+
        // col5 = setores lidos, col9 = setores escritos (índice 0-based)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        let name            = parts[2].to_string();
        let sectors_read:    u64 = parts[5].parse().unwrap_or(0);
        let sectors_written: u64 = parts[9].parse().unwrap_or(0);
        result.insert(name, (sectors_read, sectors_written));
    }
    result
}

/// Extrai apenas o nome base do dispositivo (ex.: "/dev/sda" → "sda").
fn device_basename(path: &std::ffi::OsStr) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

// ---------------------------------------------------------------------------
// Estado da interface de rede via /sys/class/net
// ---------------------------------------------------------------------------

/// Retorna true se a interface estiver "up" segundo /sys/class/net/{name}/operstate.
fn read_interface_operstate(name: &str) -> bool {
    std::fs::read_to_string(format!("/sys/class/net/{name}/operstate"))
        .map(|s| s.trim() == "up")
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// SystemMonitor
// ---------------------------------------------------------------------------

pub struct SystemMonitor {
    pub(crate) system: System,
    pub(crate) disks: Disks,
    pub(crate) networks: Networks,
    pub(crate) components: Components,
    /// Percentual de CPU consumido por processos do usuário (delta /proc/stat).
    pub(crate) cpu_user_percent: f32,
    /// Percentual de CPU consumido pelo kernel (delta /proc/stat).
    pub(crate) cpu_system_percent: f32,
    /// Percentual de CPU em idle (delta /proc/stat).
    pub(crate) cpu_idle_percent: f32,
    /// Taxa de leitura por dispositivo em bytes/seg (janela 200 ms).
    pub(crate) disk_read_rates: HashMap<String, u64>,
    /// Taxa de escrita por dispositivo em bytes/seg (janela 200 ms).
    pub(crate) disk_write_rates: HashMap<String, u64>,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let disks      = Disks::new_with_refreshed_list();
        let networks   = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        Self {
            system,
            disks,
            networks,
            components,
            cpu_user_percent:   0.0,
            cpu_system_percent: 0.0,
            cpu_idle_percent:   0.0,
            disk_read_rates:    HashMap::new(),
            disk_write_rates:   HashMap::new(),
        }
    }

    /// Atualiza todas as métricas do sistema. Realiza duas amostragens com
    /// 200 ms de intervalo para que os percentuais de CPU sejam precisos.
    pub async fn update_metrics(&mut self) {
        // Snapshots antes do intervalo de medição
        let cpu_stat_before  = read_cpu_stat_raw();
        let disk_io_before   = read_diskstats();

        self.system.refresh_all();
        self.disks.refresh(false);
        self.networks.refresh(false);
        self.components.refresh(false);

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        self.system.refresh_all();
        self.disks.refresh(false);
        self.networks.refresh(false);
        self.components.refresh(false);

        // Snapshots após o intervalo de medição
        let cpu_stat_after = read_cpu_stat_raw();
        let disk_io_after  = read_diskstats();

        // Atualiza breakdown de CPU (user/system/idle)
        if let (Some(before), Some(after)) = (cpu_stat_before, cpu_stat_after) {
            let (user, system, idle) = compute_cpu_percents(&before, &after);
            self.cpu_user_percent   = user;
            self.cpu_system_percent = system;
            self.cpu_idle_percent   = idle;
        }

        // Atualiza taxas de I/O de disco (bytes/seg sobre janela de 200 ms)
        self.disk_read_rates.clear();
        self.disk_write_rates.clear();
        for (name, (sr_after, sw_after)) in &disk_io_after {
            if let Some((sr_before, sw_before)) = disk_io_before.get(name) {
                let delta_read  = sr_after.saturating_sub(*sr_before);
                let delta_write = sw_after.saturating_sub(*sw_before);
                // bytes/seg = delta_setores × 512 / 0,2 s = delta × 512 × 5
                self.disk_read_rates.insert(
                    name.clone(),
                    delta_read  * SECTOR_BYTES * IO_INTERVALS_PER_SEC,
                );
                self.disk_write_rates.insert(
                    name.clone(),
                    delta_write * SECTOR_BYTES * IO_INTERVALS_PER_SEC,
                );
            }
        }
    }

    pub fn get_cpu_metrics(&self) -> CpuMetrics {
        let cpus       = self.system.cpus();
        let core_count = cpus.len();
        let per_core_usage: Vec<f32> = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
        let total_usage = if core_count > 0 {
            per_core_usage.iter().sum::<f32>() / core_count as f32
        } else {
            0.0
        };
        let frequency = cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0);
        let name = cpus.first().map(|cpu| cpu.brand().to_string()).unwrap_or_default();

        CpuMetrics {
            usage_percent:   total_usage,
            user_percent:    self.cpu_user_percent,
            system_percent:  self.cpu_system_percent,
            idle_percent:    self.cpu_idle_percent,
            core_count,
            per_core_usage,
            frequency,
            name,
        }
    }

    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        let total_memory     = bytes_to_gb(self.system.total_memory());
        let used_memory      = bytes_to_gb(self.system.used_memory());
        let available_memory = bytes_to_gb(self.system.available_memory());
        let usage_percent = if total_memory > 0.0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        MemoryMetrics {
            total_memory,
            used_memory,
            available_memory,
            usage_percent,
            total_swap: bytes_to_gb(self.system.total_swap()),
            used_swap:  bytes_to_gb(self.system.used_swap()),
        }
    }

    pub fn get_disk_metrics(&self) -> DiskMetrics {
        let disks: Vec<DiskInfo> = self
            .disks
            .iter()
            .map(|disk| {
                let total_space     = bytes_to_gb(disk.total_space());
                let available_space = bytes_to_gb(disk.available_space());
                let used_space      = total_space - available_space;
                let usage_percent   = if total_space > 0.0 {
                    (used_space as f32 / total_space as f32) * 100.0
                } else {
                    0.0
                };

                let device           = device_basename(disk.name());
                let read_bytes_per_sec  = *self.disk_read_rates.get(&device).unwrap_or(&0);
                let write_bytes_per_sec = *self.disk_write_rates.get(&device).unwrap_or(&0);

                DiskInfo {
                    name: disk.name().to_str().unwrap_or("Unknown").to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space,
                    available_space,
                    used_space,
                    usage_percent,
                    read_bytes_per_sec,
                    write_bytes_per_sec,
                }
            })
            .collect();

        let total_space             = disks.iter().map(|d| d.total_space).sum();
        let used_space              = disks.iter().map(|d| d.used_space).sum();
        let available_space         = disks.iter().map(|d| d.available_space).sum();
        let total_read_bytes_per_sec  = disks.iter().map(|d| d.read_bytes_per_sec).sum();
        let total_write_bytes_per_sec = disks.iter().map(|d| d.write_bytes_per_sec).sum();

        DiskMetrics {
            disks,
            total_space,
            used_space,
            available_space,
            total_read_bytes_per_sec,
            total_write_bytes_per_sec,
        }
    }

    pub fn get_network_metrics(&self) -> NetworkMetrics {
        let mut interfaces          = HashMap::new();
        let mut total_bytes_received    = 0;
        let mut total_bytes_transmitted = 0;

        for (interface_name, data) in &self.networks {
            let is_up = read_interface_operstate(interface_name);
            let interface = NetworkInterface {
                bytes_received:      data.total_received(),
                bytes_transmitted:   data.total_transmitted(),
                packets_received:    data.total_packets_received(),
                packets_transmitted: data.total_packets_transmitted(),
                errors_received:     data.total_errors_on_received(),
                errors_transmitted:  data.total_errors_on_transmitted(),
                is_up,
            };

            total_bytes_received    += interface.bytes_received;
            total_bytes_transmitted += interface.bytes_transmitted;
            interfaces.insert(interface_name.clone(), interface);
        }

        NetworkMetrics {
            interfaces,
            total_bytes_received,
            total_bytes_transmitted,
        }
    }

    pub fn get_sensor_metrics(&self) -> SensorMetrics {
        let temperatures: Vec<TemperatureSensor> = self
            .components
            .iter()
            .filter_map(|component| {
                let temperature = component.temperature()?;
                if !temperature.is_finite() {
                    return None;
                }
                let label = component.label().trim();
                Some(TemperatureSensor {
                    label: if label.is_empty() {
                        "Sensor".to_string()
                    } else {
                        label.to_string()
                    },
                    temperature_celsius: temperature,
                    max_celsius:      component.max().filter(|v| v.is_finite()),
                    critical_celsius: component.critical().filter(|v| v.is_finite()),
                })
            })
            .collect();

        let average_temperature_celsius = if temperatures.is_empty() {
            None
        } else {
            Some(
                temperatures.iter().map(|s| s.temperature_celsius).sum::<f32>()
                    / temperatures.len() as f32,
            )
        };

        let hottest_sensor = temperatures.iter().max_by(|l, r| {
            l.temperature_celsius
                .partial_cmp(&r.temperature_celsius)
                .unwrap_or(Ordering::Equal)
        });
        let hottest_temperature_celsius = hottest_sensor.map(|s| s.temperature_celsius);
        let hottest_label = hottest_sensor.map(|s| s.label.clone()).unwrap_or_default();

        let hwmon_metrics = collect_hwmon_metrics_from_path(Path::new(HWMON_BASE_PATH));

        SensorMetrics {
            temperatures,
            average_temperature_celsius,
            hottest_temperature_celsius,
            hottest_label,
            fans:      hwmon_metrics.fans,
            voltages:  hwmon_metrics.voltages,
            currents:  hwmon_metrics.currents,
            powers:    hwmon_metrics.powers,
        }
    }

    pub fn get_all_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu:     self.get_cpu_metrics(),
            memory:  self.get_memory_metrics(),
            disk:    self.get_disk_metrics(),
            network: self.get_network_metrics(),
            sensors: self.get_sensor_metrics(),
            uptime:  System::uptime(),
            load_average: {
                let la = System::load_average();
                (la.one, la.five, la.fifteen)
            },
        }
    }
}
