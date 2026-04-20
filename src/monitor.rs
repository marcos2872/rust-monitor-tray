use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{Disks, Networks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub core_count: usize,
    pub per_core_usage: Vec<f32>,
    pub frequency: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_memory: f64,
    pub used_memory: f64,
    pub available_memory: f64,
    pub usage_percent: f32,
    pub total_swap: f64,
    pub used_swap: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub disks: Vec<DiskInfo>,
    pub total_space: f64,
    pub used_space: f64,
    pub available_space: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: f64,
    pub available_space: f64,
    pub used_space: f64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interfaces: HashMap<String, NetworkInterface>,
    pub total_bytes_received: u64,
    pub total_bytes_transmitted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_received: u64,
    pub errors_transmitted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub uptime: u64,
    pub load_average: (f64, f64, f64),
}

// Constante para conversão de bytes para GB
const BYTES_TO_GB: f64 = 1024.0 * 1024.0 * 1024.0; // 1.073.741.824

// Função para converter bytes para GB
fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

pub struct SystemMonitor {
    system: System,
    disks: Disks,
    networks: Networks,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        Self {
            system,
            disks,
            networks,
        }
    }

    pub async fn update_metrics(&mut self) {
        self.system.refresh_all();
        // Disks and Networks need to be refreshed with 'true' parameter
        // to fully update all data
        // Wait minimum interval for accurate CPU measurements
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        self.system.refresh_all();
    }

    pub fn get_cpu_metrics(&self) -> CpuMetrics {
        let cpus = self.system.cpus();
        let core_count = cpus.len();
        let per_core_usage: Vec<f32> = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
        let total_usage = if core_count > 0 {
            per_core_usage.iter().sum::<f32>() / core_count as f32
        } else {
            0.0
        };
        let frequency = cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0);
        let name = cpus
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_default();

        CpuMetrics {
            usage_percent: total_usage,
            core_count,
            per_core_usage,
            frequency,
            name,
        }
    }

    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        let total_memory = bytes_to_gb(self.system.total_memory());
        let used_memory = bytes_to_gb(self.system.used_memory());
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
            used_swap: bytes_to_gb(self.system.used_swap()),
        }
    }

    pub fn get_disk_metrics(&self) -> DiskMetrics {
        let disks: Vec<DiskInfo> = self
            .disks
            .iter()
            .map(|disk| {
                let total_space = bytes_to_gb(disk.total_space());
                let available_space = bytes_to_gb(disk.available_space());
                let used_space = total_space - available_space;
                let usage_percent = if total_space > 0.0 {
                    (used_space as f32 / total_space as f32) * 100.0
                } else {
                    0.0
                };

                DiskInfo {
                    name: disk.name().to_str().unwrap_or("Unknown").to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space,
                    available_space,
                    used_space,
                    usage_percent,
                }
            })
            .collect();

        let total_space = disks.iter().map(|d| d.total_space).sum();
        let used_space = disks.iter().map(|d| d.used_space).sum();
        let available_space = disks.iter().map(|d| d.available_space).sum();

        DiskMetrics {
            disks,
            total_space,
            used_space,
            available_space,
        }
    }

    pub fn get_network_metrics(&self) -> NetworkMetrics {
        let mut interfaces = HashMap::new();
        let mut total_bytes_received = 0;
        let mut total_bytes_transmitted = 0;

        for (interface_name, data) in &self.networks {
            let interface = NetworkInterface {
                bytes_received: data.total_received(),
                bytes_transmitted: data.total_transmitted(),
                packets_received: data.total_packets_received(),
                packets_transmitted: data.total_packets_transmitted(),
                errors_received: data.total_errors_on_received(),
                errors_transmitted: data.total_errors_on_transmitted(),
            };

            total_bytes_received += interface.bytes_received;
            total_bytes_transmitted += interface.bytes_transmitted;

            interfaces.insert(interface_name.clone(), interface);
        }

        NetworkMetrics {
            interfaces,
            total_bytes_received,
            total_bytes_transmitted,
        }
    }

    pub fn get_all_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu: self.get_cpu_metrics(),
            memory: self.get_memory_metrics(),
            disk: self.get_disk_metrics(),
            network: self.get_network_metrics(),
            uptime: System::uptime(),
            load_average: {
                let load_avg = System::load_average();
                (load_avg.one, load_avg.five, load_avg.fifteen)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_gb_converts_gibibytes() {
        let bytes = 2 * 1024 * 1024 * 1024;

        assert_eq!(bytes_to_gb(bytes), 2.0);
    }

    #[test]
    fn test_get_cpu_metrics_returns_zero_usage_when_system_has_no_cpu_snapshot() {
        let monitor = SystemMonitor {
            system: System::new(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        };

        let cpu = monitor.get_cpu_metrics();

        assert_eq!(cpu.core_count, 0);
        assert_eq!(cpu.per_core_usage, Vec::<f32>::new());
        assert_eq!(cpu.usage_percent, 0.0);
        assert_eq!(cpu.frequency, 0);
        assert_eq!(cpu.name, "");
    }

    #[test]
    fn test_get_memory_metrics_returns_zero_usage_when_total_memory_is_zero() {
        let monitor = SystemMonitor {
            system: System::new(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        };

        let memory = monitor.get_memory_metrics();

        assert_eq!(memory.total_memory, 0.0);
        assert_eq!(memory.used_memory, 0.0);
        assert_eq!(memory.available_memory, 0.0);
        assert_eq!(memory.usage_percent, 0.0);
    }

    #[test]
    fn test_get_cpu_metrics_returns_consistent_shape_on_live_system() {
        let monitor = SystemMonitor::new();

        let cpu = monitor.get_cpu_metrics();

        assert_eq!(cpu.per_core_usage.len(), cpu.core_count);
        assert!(cpu.usage_percent.is_finite());
        assert!(cpu.usage_percent >= 0.0);
    }

    #[test]
    fn test_get_disk_metrics_aggregates_child_disks() {
        let monitor = SystemMonitor::new();

        let disk = monitor.get_disk_metrics();
        let expected_total: f64 = disk.disks.iter().map(|item| item.total_space).sum();
        let expected_used: f64 = disk.disks.iter().map(|item| item.used_space).sum();
        let expected_available: f64 = disk.disks.iter().map(|item| item.available_space).sum();

        assert!((disk.total_space - expected_total).abs() < f64::EPSILON);
        assert!((disk.used_space - expected_used).abs() < f64::EPSILON);
        assert!((disk.available_space - expected_available).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_network_metrics_totals_match_interface_sums() {
        let monitor = SystemMonitor::new();

        let network = monitor.get_network_metrics();
        let expected_received: u64 = network
            .interfaces
            .values()
            .map(|interface| interface.bytes_received)
            .sum();
        let expected_transmitted: u64 = network
            .interfaces
            .values()
            .map(|interface| interface.bytes_transmitted)
            .sum();

        assert_eq!(network.total_bytes_received, expected_received);
        assert_eq!(network.total_bytes_transmitted, expected_transmitted);
    }

    #[test]
    fn test_get_all_metrics_returns_non_negative_snapshot() {
        let monitor = SystemMonitor::new();

        let metrics = monitor.get_all_metrics();

        assert!(metrics.uptime <= System::uptime());
        assert!(metrics.cpu.usage_percent.is_finite());
        assert!(metrics.memory.usage_percent.is_finite());
        assert!(metrics.load_average.0.is_finite());
        assert!(metrics.load_average.1.is_finite());
        assert!(metrics.load_average.2.is_finite());
    }
}
