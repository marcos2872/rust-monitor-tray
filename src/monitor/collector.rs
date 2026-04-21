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

pub(crate) fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

pub struct SystemMonitor {
    pub(crate) system: System,
    pub(crate) disks: Disks,
    pub(crate) networks: Networks,
    pub(crate) components: Components,
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

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        Self {
            system,
            disks,
            networks,
            components,
        }
    }

    pub async fn update_metrics(&mut self) {
        self.system.refresh_all();
        self.disks.refresh(false);
        self.networks.refresh(false);
        self.components.refresh(false);
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        self.system.refresh_all();
        self.disks.refresh(false);
        self.networks.refresh(false);
        self.components.refresh(false);
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

        let total_space = disks.iter().map(|disk| disk.total_space).sum();
        let used_space = disks.iter().map(|disk| disk.used_space).sum();
        let available_space = disks.iter().map(|disk| disk.available_space).sum();

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
                    max_celsius: component.max().filter(|value| value.is_finite()),
                    critical_celsius: component.critical().filter(|value| value.is_finite()),
                })
            })
            .collect();

        let average_temperature_celsius = if temperatures.is_empty() {
            None
        } else {
            Some(
                temperatures
                    .iter()
                    .map(|sensor| sensor.temperature_celsius)
                    .sum::<f32>()
                    / temperatures.len() as f32,
            )
        };

        let hottest_sensor = temperatures.iter().max_by(|left, right| {
            left.temperature_celsius
                .partial_cmp(&right.temperature_celsius)
                .unwrap_or(Ordering::Equal)
        });
        let hottest_temperature_celsius = hottest_sensor.map(|sensor| sensor.temperature_celsius);
        let hottest_label = hottest_sensor
            .map(|sensor| sensor.label.clone())
            .unwrap_or_default();
        let hwmon_metrics = collect_hwmon_metrics_from_path(Path::new(HWMON_BASE_PATH));

        SensorMetrics {
            temperatures,
            average_temperature_celsius,
            hottest_temperature_celsius,
            hottest_label,
            fans: hwmon_metrics.fans,
            voltages: hwmon_metrics.voltages,
            currents: hwmon_metrics.currents,
            powers: hwmon_metrics.powers,
        }
    }

    pub fn get_all_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            cpu: self.get_cpu_metrics(),
            memory: self.get_memory_metrics(),
            disk: self.get_disk_metrics(),
            network: self.get_network_metrics(),
            sensors: self.get_sensor_metrics(),
            uptime: System::uptime(),
            load_average: {
                let load_avg = System::load_average();
                (load_avg.one, load_avg.five, load_avg.fifteen)
            },
        }
    }
}
