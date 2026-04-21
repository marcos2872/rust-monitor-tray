use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use sysinfo::{Components, Disks, Networks, System};

use super::hwmon::{collect_hwmon_metrics_from_path, HWMON_BASE_PATH};
use super::{
    CpuMetrics, DiskInfo, DiskMetrics, MemoryMetrics, NetworkInterface, NetworkMetrics,
    SensorMetrics, SystemInfo, SystemMetrics, TemperatureSensor,
};

const BYTES_TO_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const SECTOR_BYTES: f64 = 512.0;

pub(crate) fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

// ---------------------------------------------------------------------------
// Leitura de /proc/stat para breakdown user/system/idle/steal da CPU
// ---------------------------------------------------------------------------

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
    fn total_idle(&self) -> u64 { self.idle + self.iowait }
    fn total_system(&self) -> u64 { self.system + self.irq + self.softirq }
    fn total_user(&self) -> u64 { self.user + self.nice }
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

/// Retorna (user%, system%, idle%, steal%) a partir de dois snapshots de /proc/stat.
fn compute_cpu_percents(prev: &CpuStatRaw, curr: &CpuStatRaw) -> (f32, f32, f32, f32) {
    let d_user   = curr.total_user().saturating_sub(prev.total_user());
    let d_system = curr.total_system().saturating_sub(prev.total_system());
    let d_idle   = curr.total_idle().saturating_sub(prev.total_idle());
    let d_steal  = curr.steal.saturating_sub(prev.steal);
    let d_total  = d_user + d_system + d_idle + d_steal;

    if d_total == 0 {
        return (0.0, 0.0, 100.0, 0.0);
    }

    let scale = 100.0 / d_total as f32;
    (
        (d_user   as f32 * scale).min(100.0),
        (d_system as f32 * scale).min(100.0),
        (d_idle   as f32 * scale).min(100.0),
        (d_steal  as f32 * scale).min(100.0),
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

/// Calcula taxas de leitura e escrita em bytes/seg a partir de dois snapshots de
/// `/proc/diskstats` e do tempo real decorrido.
fn compute_disk_io_rates(
    before: &HashMap<String, (u64, u64)>,
    after:  &HashMap<String, (u64, u64)>,
    elapsed_secs: f64,
) -> (HashMap<String, u64>, HashMap<String, u64>) {
    let mut read_rates  = HashMap::new();
    let mut write_rates = HashMap::new();
    for (name, (sr_after, sw_after)) in after {
        if let Some((sr_before, sw_before)) = before.get(name) {
            let d_read  = sr_after.saturating_sub(*sr_before);
            let d_write = sw_after.saturating_sub(*sw_before);
            read_rates.insert(
                name.clone(),
                (d_read  as f64 * SECTOR_BYTES / elapsed_secs).round() as u64,
            );
            write_rates.insert(
                name.clone(),
                (d_write as f64 * SECTOR_BYTES / elapsed_secs).round() as u64,
            );
        }
    }
    (read_rates, write_rates)
}

/// Constrói um `DiskInfo` a partir de uma entrada de disco do sysinfo e suas taxas de I/O.
fn build_disk_info(disk: &sysinfo::Disk, read_rate: u64, write_rate: u64) -> DiskInfo {
    let total_space     = bytes_to_gb(disk.total_space());
    let available_space = bytes_to_gb(disk.available_space());
    let used_space      = total_space - available_space;
    let usage_percent   = if total_space > 0.0 {
        (used_space as f32 / total_space as f32) * 100.0
    } else {
        0.0
    };
    DiskInfo {
        name:               disk.name().to_str().unwrap_or("Unknown").to_string(),
        mount_point:        disk.mount_point().to_string_lossy().to_string(),
        total_space,
        available_space,
        used_space,
        usage_percent,
        read_bytes_per_sec:  read_rate,
        write_bytes_per_sec: write_rate,
    }
}

// ---------------------------------------------------------------------------
// Estado da interface de rede via /sys/class/net
// ---------------------------------------------------------------------------

/// Retorna true quando a interface está ativa.
/// "up"      → link físico confirmado.
/// "unknown" → interface sem conceito de link (loopback, tunnels) — tratada como UP.
fn read_interface_operstate(name: &str) -> bool {
    let state = std::fs::read_to_string(format!("/sys/class/net/{name}/operstate"))
        .unwrap_or_default();
    matches!(state.trim(), "up" | "unknown")
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

        Self {
            system,
            disks:      Disks::new_with_refreshed_list(),
            networks:   Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            cpu_user_percent:   0.0,
            cpu_system_percent: 0.0,
            cpu_idle_percent:   0.0,
            cpu_steal_percent:  0.0,
            disk_read_rates:    HashMap::new(),
            disk_write_rates:   HashMap::new(),
        }
    }

    /// Atualiza métricas com duas amostragens separadas por ~200 ms.
    /// Usa `Instant` para calcular taxas de I/O com o tempo real decorrido.
    pub async fn update_metrics(&mut self) {
        let cpu_stat_before = read_cpu_stat_raw();
        let disk_io_before  = read_diskstats();

        self.system.refresh_all();
        self.disks.refresh(false);
        self.networks.refresh(false);
        self.components.refresh(false);

        let window_start = Instant::now();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let elapsed_secs = window_start.elapsed().as_secs_f64().max(0.001);

        self.system.refresh_all();
        self.disks.refresh(false);
        self.networks.refresh(false);
        self.components.refresh(false);

        let cpu_stat_after = read_cpu_stat_raw();
        let disk_io_after  = read_diskstats();

        // Breakdown de CPU (user / system / idle / steal)
        if let (Some(before), Some(after)) = (cpu_stat_before, cpu_stat_after) {
            let (user, system, idle, steal) = compute_cpu_percents(&before, &after);
            self.cpu_user_percent   = user;
            self.cpu_system_percent = system;
            self.cpu_idle_percent   = idle;
            self.cpu_steal_percent  = steal;
        }

        // Taxas de I/O usando o tempo real decorrido (não assume 200 ms fixo)
        let (read_rates, write_rates) =
            compute_disk_io_rates(&disk_io_before, &disk_io_after, elapsed_secs);
        self.disk_read_rates  = read_rates;
        self.disk_write_rates = write_rates;
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

        CpuMetrics {
            usage_percent:   total_usage,
            user_percent:    self.cpu_user_percent,
            system_percent:  self.cpu_system_percent,
            idle_percent:    self.cpu_idle_percent,
            steal_percent:   self.cpu_steal_percent,
            core_count,
            per_core_usage,
            frequency: cpus.first().map(|c| c.frequency()).unwrap_or(0),
            name:      cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
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
                let device     = device_basename(disk.name());
                let read_rate  = *self.disk_read_rates.get(&device).unwrap_or(&0);
                let write_rate = *self.disk_write_rates.get(&device).unwrap_or(&0);
                build_disk_info(disk, read_rate, write_rate)
            })
            .collect();

        DiskMetrics {
            total_space:              disks.iter().map(|d| d.total_space).sum(),
            used_space:               disks.iter().map(|d| d.used_space).sum(),
            available_space:          disks.iter().map(|d| d.available_space).sum(),
            total_read_bytes_per_sec:  disks.iter().map(|d| d.read_bytes_per_sec).sum(),
            total_write_bytes_per_sec: disks.iter().map(|d| d.write_bytes_per_sec).sum(),
            disks,
        }
    }

    pub fn get_network_metrics(&self) -> NetworkMetrics {
        let mut interfaces          = HashMap::new();
        let mut total_bytes_received    = 0u64;
        let mut total_bytes_transmitted = 0u64;

        for (interface_name, data) in &self.networks {
            let interface = NetworkInterface {
                bytes_received:      data.total_received(),
                bytes_transmitted:   data.total_transmitted(),
                packets_received:    data.total_packets_received(),
                packets_transmitted: data.total_packets_transmitted(),
                errors_received:     data.total_errors_on_received(),
                errors_transmitted:  data.total_errors_on_transmitted(),
                is_up:               read_interface_operstate(interface_name),
            };
            total_bytes_received    += interface.bytes_received;
            total_bytes_transmitted += interface.bytes_transmitted;
            interfaces.insert(interface_name.clone(), interface);
        }

        NetworkMetrics { interfaces, total_bytes_received, total_bytes_transmitted }
    }

    pub fn get_sensor_metrics(&self) -> SensorMetrics {
        // Desestrutura hwmon_metrics para mover temperatures sem clone
        let hwmon = collect_hwmon_metrics_from_path(Path::new(HWMON_BASE_PATH));
        let (hwmon_temps, fans, voltages, currents, powers) = (
            hwmon.temperatures,
            hwmon.fans,
            hwmon.voltages,
            hwmon.currents,
            hwmon.powers,
        );

        // hwmon é a fonte primária; sysinfo::Components é fallback para VMs/containers
        let temperatures: Vec<TemperatureSensor> = if !hwmon_temps.is_empty() {
            hwmon_temps
        } else {
            self.components
                .iter()
                .filter_map(|c| {
                    let t = c.temperature()?;
                    if !t.is_finite() { return None; }
                    let label = c.label().trim();
                    Some(TemperatureSensor {
                        label: if label.is_empty() { "Sensor".to_string() } else { label.to_string() },
                        chip:  "Sistema".to_string(),
                        temperature_celsius: t,
                        max_celsius:      c.max().filter(|v| v.is_finite()),
                        critical_celsius: c.critical().filter(|v| v.is_finite()),
                    })
                })
                .collect()
        };

        let average_temperature_celsius = if temperatures.is_empty() {
            None
        } else {
            Some(temperatures.iter().map(|s| s.temperature_celsius).sum::<f32>()
                / temperatures.len() as f32)
        };

        let hottest = temperatures.iter().max_by(|l, r| {
            l.temperature_celsius.partial_cmp(&r.temperature_celsius)
                .unwrap_or(Ordering::Equal)
        });

        SensorMetrics {
            hottest_temperature_celsius: hottest.map(|s| s.temperature_celsius),
            hottest_label:              hottest.map(|s| s.label.clone()).unwrap_or_default(),
            average_temperature_celsius,
            temperatures,
            fans,
            voltages,
            currents,
            powers,
        }
    }

    pub fn get_all_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu:     self.get_cpu_metrics(),
            memory:  self.get_memory_metrics(),
            disk:    self.get_disk_metrics(),
            network: self.get_network_metrics(),
            sensors: self.get_sensor_metrics(),
            system_info: SystemInfo {
                hostname:       System::host_name().unwrap_or_else(|| "unknown".to_string()),
                os_name:        System::name().unwrap_or_else(|| "Linux".to_string()),
                os_version:     System::os_version().unwrap_or_default(),
                kernel_version: System::kernel_version().unwrap_or_default(),
                architecture:   System::cpu_arch(),
                process_count:  self.system.processes().len(),
            },
            uptime: System::uptime(),
            load_average: {
                let la = System::load_average();
                (la.one, la.five, la.fifteen)
            },
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
        assert_eq!(device_basename(OsStr::new("sda")), "sda");
        assert_eq!(device_basename(OsStr::new("")), "");
    }

    #[test]
    fn test_compute_cpu_percents_distribui_corretamente() {
        let prev = CpuStatRaw { user: 0, nice: 0, system: 0, idle: 0,
            iowait: 0, irq: 0, softirq: 0, steal: 0 };
        // user+nice=50, system=20, idle=30 de 100 ticks totais
        let curr = CpuStatRaw { user: 40, nice: 10, system: 20, idle: 30,
            iowait: 0, irq: 0, softirq: 0, steal: 0 };
        let (user, system, idle, steal) = compute_cpu_percents(&prev, &curr);
        assert!((user   - 50.0).abs() < 0.1, "user% esperado 50, obteve {user}");
        assert!((system - 20.0).abs() < 0.1, "system% esperado 20, obteve {system}");
        assert!((idle   - 30.0).abs() < 0.1, "idle% esperado 30, obteve {idle}");
        assert!((steal  -  0.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_cpu_percents_retorna_idle_total_sem_delta() {
        let snap = CpuStatRaw { user: 100, nice: 0, system: 50, idle: 200,
            iowait: 0, irq: 0, softirq: 0, steal: 0 };
        let (_, _, idle, steal) = compute_cpu_percents(&snap, &snap);
        assert_eq!(idle,  100.0);
        assert_eq!(steal,   0.0);
    }

    #[test]
    fn test_compute_cpu_percents_contabiliza_steal() {
        let prev = CpuStatRaw { user: 0, nice: 0, system: 0, idle: 0,
            iowait: 0, irq: 0, softirq: 0, steal: 0 };
        let curr = CpuStatRaw { user: 25, nice: 0, system: 25, idle: 25,
            iowait: 0, irq: 0, softirq: 0, steal: 25 };
        let (user, system, idle, steal) = compute_cpu_percents(&prev, &curr);
        assert!((user   - 25.0).abs() < 0.1);
        assert!((system - 25.0).abs() < 0.1);
        assert!((idle   - 25.0).abs() < 0.1);
        assert!((steal  - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_disk_io_rates_converte_setores_para_bytes_por_seg() {
        let mut before = HashMap::new();
        before.insert("sda".to_string(), (1000u64, 500u64));
        let mut after = HashMap::new();
        after.insert("sda".to_string(), (1200u64, 700u64));
        // delta_read=200, delta_write=200; 200 * 512 / 1.0 = 102400
        let (read_rates, write_rates) = compute_disk_io_rates(&before, &after, 1.0);
        assert_eq!(*read_rates.get("sda").unwrap(),  102_400);
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
