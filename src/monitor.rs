use serde::{Deserialize, Serialize};
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
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    // pub interfaces: HashMap<String, NetworkInterface>,
    pub total_bytes_received: u64,
    pub total_bytes_transmitted: u64,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct NetworkInterface {
//     pub bytes_received: u64,
//     pub bytes_transmitted: u64,
//     pub packets_received: u64,
//     pub packets_transmitted: u64,
//     pub errors_received: u64,
//     pub errors_transmitted: u64,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub name: Option<String>,
    pub host_name: Option<String>,
    pub os_version: Option<String>,
}

#[allow(dead_code)]
pub struct SystemMonitor {
    system: System,
}

// Constante para conversão de bytes para GB
const BYTES_TO_GB: f64 = 1024.0 * 1024.0 * 1024.0; // 1.073.741.824

// Função para converter bytes para GB
fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

#[allow(dead_code)]
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
        let total_usage = self.system.global_cpu_usage();
        let per_core_usage = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
        let frequency = cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0);
        let name = cpus
            .first()
            .map(|cpu| cpu.name())
            .unwrap_or("CPU não especificado")
            .to_string();

        CpuMetrics {
            usage_percent: total_usage,
            core_count: cpus.len(),
            per_core_usage,
            frequency,
            name,
        }
    }

    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let available_memory = self.system.available_memory();
        let usage_percent = (used_memory as f32 / total_memory as f32) * 100.0;

        MemoryMetrics {
            total_memory: bytes_to_gb(total_memory),
            used_memory: bytes_to_gb(used_memory),
            available_memory: bytes_to_gb(available_memory),
            usage_percent,
            total_swap: bytes_to_gb(self.system.total_swap()),
            used_swap: bytes_to_gb(self.system.used_swap()),
        }
    }

    pub fn get_disk_metrics(&self) -> DiskMetrics {
        let disks = Disks::new_with_refreshed_list();

        let mut disks_info = Vec::<DiskInfo>::new();

        for disk in &disks {
            let total_space = disk.total_space();
            let name = disk.name().to_string_lossy().to_string();
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0 {
                (used_space as f32 / total_space as f32) * 100.0
            } else {
                0.0
            };

            if bytes_to_gb(total_space) > 64.0 {
                disks_info.push(DiskInfo {
                    total_space,
                    available_space,
                    usage_percent,
                    used_space,
                    name,
                    mount_point,
                });
            }
        }

        let total_space = disks_info.iter().map(|d| d.total_space).sum();
        let used_space = disks_info.iter().map(|d| d.used_space).sum();
        let available_space = disks_info.iter().map(|d| d.available_space).sum();

        DiskMetrics {
            disks: disks_info,
            total_space,
            used_space,
            available_space,
        }
    }

    pub fn get_network_metrics(&self) -> NetworkMetrics {
        // let mut interfaces = HashMap::new();
        let networks = Networks::new_with_refreshed_list();
        let mut total_bytes_received = 0;
        let mut total_bytes_transmitted = 0;

        for (_interface_name, data) in &networks {
            // let interface = NetworkInterface {
            //     bytes_received: data.received(),
            //     bytes_transmitted: data.transmitted(),
            //     packets_received: data.packets_received(),
            //     packets_transmitted: data.packets_transmitted(),
            //     errors_received: data.errors_on_received(),
            //     errors_transmitted: data.errors_on_transmitted(),
            // };

            total_bytes_received += data.total_received();
            total_bytes_transmitted += data.total_transmitted();

            // interfaces.insert(interface_name.clone(), interface);
        }

        NetworkMetrics {
            // interfaces,
            total_bytes_received,
            total_bytes_transmitted,
        }
    }

    pub fn get_all_metrics(&self) -> SystemMetrics {
        let name = System::name();
        let host_name = System::host_name();
        let os_version = System::os_version();
        SystemMetrics {
            cpu: self.get_cpu_metrics(),
            memory: self.get_memory_metrics(),
            disk: self.get_disk_metrics(),
            network: self.get_network_metrics(),
            name,
            host_name,
            os_version,
        }
    }
}
