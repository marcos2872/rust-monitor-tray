use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub user_percent: f32,
    pub system_percent: f32,
    pub idle_percent: f32,
    pub steal_percent: f32,
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
    pub total_read_bytes_per_sec: u64,
    pub total_write_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: f64,
    pub available_space: f64,
    pub used_space: f64,
    pub usage_percent: f32,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
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
    pub is_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureSensor {
    pub label: String,
    pub chip: String,
    pub temperature_celsius: f32,
    pub max_celsius: Option<f32>,
    pub critical_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanSensor {
    pub label: String,
    pub rpm: u64,
    pub duty_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoltageSensor {
    pub label: String,
    pub volts: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentSensor {
    pub label: String,
    pub amps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSensor {
    pub label: String,
    pub watts: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorMetrics {
    pub temperatures: Vec<TemperatureSensor>,
    pub average_temperature_celsius: Option<f32>,
    pub hottest_temperature_celsius: Option<f32>,
    pub hottest_label: String,
    pub fans: Vec<FanSensor>,
    pub voltages: Vec<VoltageSensor>,
    pub currents: Vec<CurrentSensor>,
    pub powers: Vec<PowerSensor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub architecture: String,
    pub process_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub sensors: SensorMetrics,
    pub system_info: SystemInfo,
    pub uptime: u64,
    pub load_average: (f64, f64, f64),
}
