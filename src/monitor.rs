use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{CpuExt, DiskExt, NetworkExt, System, SystemExt};

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
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self { system }
    }

    pub async fn update_metrics(&mut self) {
        self.system.refresh_all();
        // Wait minimum interval for accurate CPU measurements
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        self.system.refresh_all();
    }

    pub fn get_cpu_metrics(&self) -> CpuMetrics {
        let cpus = self.system.cpus();
        let total_usage = cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        let per_core_usage = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
        let frequency = cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0);
        let name = self.system.global_cpu_info().brand().to_string();

        CpuMetrics {
            usage_percent: total_usage,
            core_count: cpus.len(),
            per_core_usage,
            frequency,
            name,
        }
    }

    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        let total_memory = bytes_to_gb(self.system.total_memory());
        let used_memory = bytes_to_gb(self.system.used_memory());
        let available_memory = bytes_to_gb(self.system.available_memory());
        let usage_percent = (used_memory as f32 / total_memory as f32) * 100.0;

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
            .system
            .disks()
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
                    name: disk.name().to_string_lossy().to_string(),
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

        for (interface_name, data) in self.system.networks() {
            let interface = NetworkInterface {
                bytes_received: data.received(),
                bytes_transmitted: data.transmitted(),
                packets_received: data.packets_received(),
                packets_transmitted: data.packets_transmitted(),
                errors_received: data.errors_on_received(),
                errors_transmitted: data.errors_on_transmitted(),
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
            uptime: self.system.uptime(),
            load_average: {
                let load_avg = self.system.load_average();
                (load_avg.one, load_avg.five, load_avg.fifteen)
            },
        }
    }
}
