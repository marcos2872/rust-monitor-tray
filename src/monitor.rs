use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use sysinfo::{Components, Disks, Networks, System};

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
pub struct TemperatureSensor {
    pub label: String,
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
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub sensors: SensorMetrics,
    pub uptime: u64,
    pub load_average: (f64, f64, f64),
}

const BYTES_TO_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const HWMON_BASE_PATH: &str = "/sys/class/hwmon";
const MILLI_SCALE: f32 = 1000.0;
const MICRO_SCALE: f32 = 1_000_000.0;
const PWM_MAX_VALUE: f32 = 255.0;

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
}

fn read_u64(path: &Path) -> Option<u64> {
    read_trimmed(path)?.parse().ok()
}

fn read_scaled_f32(path: &Path, scale: f32) -> Option<f32> {
    let value: f32 = read_trimmed(path)?.parse().ok()?;
    let scaled = value / scale;
    scaled.is_finite().then_some(scaled)
}

fn parse_sensor_index(file_name: &str, prefix: &str, suffix: &str) -> Option<String> {
    if !file_name.starts_with(prefix) || !file_name.ends_with(suffix) {
        return None;
    }

    let index = &file_name[prefix.len()..file_name.len() - suffix.len()];
    if index.is_empty() || !index.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    Some(index.to_string())
}

fn prettify_identifier(value: &str) -> String {
    value
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn hwmon_chip_name(directory: &Path) -> String {
    if let Some(name) = read_trimmed(&directory.join("name")) {
        return prettify_identifier(&name);
    }

    directory
        .file_name()
        .and_then(|value| value.to_str())
        .map(prettify_identifier)
        .unwrap_or_else(|| "hwmon".to_string())
}

fn hwmon_sensor_label(directory: &Path, prefix: &str, index: &str, fallback_name: &str) -> String {
    let chip_name = hwmon_chip_name(directory);
    let label_path = directory.join(format!("{prefix}{index}_label"));

    if let Some(label) = read_trimmed(&label_path) {
        let pretty_label = prettify_identifier(&label);
        if pretty_label.is_empty() {
            return format!("{chip_name}: {fallback_name} {index}");
        }
        return format!("{chip_name}: {pretty_label}");
    }

    format!("{chip_name}: {fallback_name} {index}")
}

#[derive(Default)]
struct HwmonMetrics {
    fans: Vec<FanSensor>,
    voltages: Vec<VoltageSensor>,
    currents: Vec<CurrentSensor>,
    powers: Vec<PowerSensor>,
}

fn collect_hwmon_metrics_from_path(base_path: &Path) -> HwmonMetrics {
    let mut metrics = HwmonMetrics::default();

    let directories = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(_) => return metrics,
    };

    for directory in directories.filter_map(Result::ok) {
        let path = directory.path();
        if !path.is_dir() {
            continue;
        }

        let files = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for file in files.filter_map(Result::ok) {
            let file_path = file.path();
            if !file_path.is_file() {
                continue;
            }

            let Some(file_name) = file.file_name().to_str().map(str::to_string) else {
                continue;
            };

            if let Some(index) = parse_sensor_index(&file_name, "fan", "_input") {
                if let Some(rpm) = read_u64(&file_path) {
                    let duty_percent = read_scaled_f32(&path.join(format!("pwm{index}")), 1.0)
                        .map(|value| (value / PWM_MAX_VALUE * 100.0).clamp(0.0, 100.0));

                    metrics.fans.push(FanSensor {
                        label: hwmon_sensor_label(&path, "fan", &index, "Fan"),
                        rpm,
                        duty_percent,
                    });
                }
                continue;
            }

            if let Some(index) = parse_sensor_index(&file_name, "in", "_input") {
                if let Some(volts) = read_scaled_f32(&file_path, MILLI_SCALE) {
                    metrics.voltages.push(VoltageSensor {
                        label: hwmon_sensor_label(&path, "in", &index, "Voltage"),
                        volts,
                    });
                }
                continue;
            }

            if let Some(index) = parse_sensor_index(&file_name, "curr", "_input") {
                if let Some(amps) = read_scaled_f32(&file_path, MILLI_SCALE) {
                    metrics.currents.push(CurrentSensor {
                        label: hwmon_sensor_label(&path, "curr", &index, "Current"),
                        amps,
                    });
                }
                continue;
            }

            if let Some(index) = parse_sensor_index(&file_name, "power", "_input") {
                if let Some(watts) = read_scaled_f32(&file_path, MICRO_SCALE) {
                    metrics.powers.push(PowerSensor {
                        label: hwmon_sensor_label(&path, "power", &index, "Power"),
                        watts,
                    });
                }
            }
        }
    }

    metrics.fans.sort_by(|left, right| left.label.cmp(&right.label));
    metrics
        .voltages
        .sort_by(|left, right| left.label.cmp(&right.label));
    metrics
        .currents
        .sort_by(|left, right| left.label.cmp(&right.label));
    metrics
        .powers
        .sort_by(|left, right| left.label.cmp(&right.label));

    metrics
}

pub struct SystemMonitor {
    system: System,
    disks: Disks,
    networks: Networks,
    components: Components,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_fixture_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("monitor-tray-tests-{unique}"))
    }

    #[test]
    fn test_bytes_to_gb_converts_gibibytes() {
        let bytes = 2 * 1024 * 1024 * 1024;
        assert_eq!(bytes_to_gb(bytes), 2.0);
    }

    #[test]
    fn test_parse_sensor_index_extracts_numeric_suffix() {
        assert_eq!(parse_sensor_index("fan1_input", "fan", "_input"), Some("1".to_string()));
        assert_eq!(parse_sensor_index("power12_input", "power", "_input"), Some("12".to_string()));
        assert_eq!(parse_sensor_index("fanx_input", "fan", "_input"), None);
        assert_eq!(parse_sensor_index("fan1_label", "fan", "_input"), None);
    }

    #[test]
    fn test_collect_hwmon_metrics_reads_fans_voltage_current_and_power() {
        let base = temp_fixture_dir();
        let hwmon0 = base.join("hwmon0");
        fs::create_dir_all(&hwmon0).expect("should create fixture dir");

        fs::write(hwmon0.join("name"), "nct6798\n").expect("should write chip name");
        fs::write(hwmon0.join("fan1_input"), "1450\n").expect("should write fan rpm");
        fs::write(hwmon0.join("fan1_label"), "CPU Fan\n").expect("should write fan label");
        fs::write(hwmon0.join("pwm1"), "128\n").expect("should write pwm");
        fs::write(hwmon0.join("in0_input"), "1200\n").expect("should write voltage");
        fs::write(hwmon0.join("curr1_input"), "2500\n").expect("should write current");
        fs::write(hwmon0.join("power1_input"), "65500000\n").expect("should write power");

        let metrics = collect_hwmon_metrics_from_path(&base);

        assert_eq!(metrics.fans.len(), 1);
        assert_eq!(metrics.fans[0].label, "nct6798: CPU Fan");
        assert_eq!(metrics.fans[0].rpm, 1450);
        assert!(metrics.fans[0]
            .duty_percent
            .map(|value| (value - 50.19608).abs() < 0.01)
            .unwrap_or(false));

        assert_eq!(metrics.voltages.len(), 1);
        assert_eq!(metrics.voltages[0].label, "nct6798: Voltage 0");
        assert!((metrics.voltages[0].volts - 1.2).abs() < f32::EPSILON);

        assert_eq!(metrics.currents.len(), 1);
        assert_eq!(metrics.currents[0].label, "nct6798: Current 1");
        assert!((metrics.currents[0].amps - 2.5).abs() < f32::EPSILON);

        assert_eq!(metrics.powers.len(), 1);
        assert_eq!(metrics.powers[0].label, "nct6798: Power 1");
        assert!((metrics.powers[0].watts - 65.5).abs() < f32::EPSILON);

        fs::remove_dir_all(base).expect("should clean fixture dir");
    }

    #[test]
    fn test_get_cpu_metrics_returns_zero_usage_when_system_has_no_cpu_snapshot() {
        let monitor = SystemMonitor {
            system: System::new(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
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
            components: Components::new_with_refreshed_list(),
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
    fn test_get_sensor_metrics_returns_finite_temperatures_when_available() {
        let monitor = SystemMonitor::new();

        let sensors = monitor.get_sensor_metrics();

        assert!(sensors
            .temperatures
            .iter()
            .all(|sensor| sensor.temperature_celsius.is_finite()));
        assert!(sensors
            .average_temperature_celsius
            .map(|value| value.is_finite())
            .unwrap_or(true));
        assert!(sensors
            .hottest_temperature_celsius
            .map(|value| value.is_finite())
            .unwrap_or(true));
        assert!(sensors.fans.iter().all(|sensor| sensor.duty_percent
            .map(|value| value.is_finite() && (0.0..=100.0).contains(&value))
            .unwrap_or(true)));
        assert!(sensors.voltages.iter().all(|sensor| sensor.volts.is_finite()));
        assert!(sensors.currents.iter().all(|sensor| sensor.amps.is_finite()));
        assert!(sensors.powers.iter().all(|sensor| sensor.watts.is_finite()));
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
        assert!(metrics
            .sensors
            .temperatures
            .iter()
            .all(|sensor| sensor.temperature_celsius.is_finite()));
    }
}
