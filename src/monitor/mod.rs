mod collector;
pub(crate) mod gpu;
mod hwmon;
mod models;

pub use collector::SystemMonitor;
pub use models::{
    CpuMetrics, CurrentSensor, DiskInfo, DiskMetrics, FanSensor, FastMetrics, GpuInfo, GpuVendor,
    MemoryMetrics, NetworkInterface, NetworkMetrics, NetworkSpeedTestPhase, NetworkSpeedTestState,
    NetworkSpeedTestStatus, PowerSensor, ProcessInfo, SensorMetrics, SlowMetrics, SystemInfo,
    SystemMetrics, TemperatureSensor, VoltageSensor,
};

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use sysinfo::{Components, Disks, Networks, System};

    use super::collector::{bytes_to_gb, SystemMonitor};
    use super::hwmon::{collect_hwmon_metrics_from_path, parse_sensor_index};

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
        assert_eq!(
            parse_sensor_index("fan1_input", "fan", "_input"),
            Some("1".to_string())
        );
        assert_eq!(
            parse_sensor_index("power12_input", "power", "_input"),
            Some("12".to_string())
        );
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
        fs::write(hwmon0.join("temp1_input"), "45000\n").expect("should write temp");
        fs::write(hwmon0.join("temp1_label"), "CPU Core\n").expect("should write temp label");

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

        assert_eq!(metrics.temperatures.len(), 1);
        assert_eq!(metrics.temperatures[0].chip, "nct6798");
        assert_eq!(metrics.temperatures[0].label, "CPU Core");
        assert!((metrics.temperatures[0].temperature_celsius - 45.0).abs() < 0.01);

        fs::remove_dir_all(base).expect("should clean fixture dir");
    }

    #[test]
    fn test_get_cpu_metrics_returns_zero_usage_when_system_has_no_cpu_snapshot() {
        let monitor = SystemMonitor {
            system: System::new(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            cpu_user_percent: 0.0,
            cpu_system_percent: 0.0,
            cpu_idle_percent: 0.0,
            cpu_steal_percent: 0.0,
            disk_read_rates: std::collections::HashMap::new(),
            disk_write_rates: std::collections::HashMap::new(),
            cached_gpus: vec![],
            cached_sensors: None,
            cached_top_processes: None,
            cached_gateway_ip: None,
            cached_gateway_latency_ms: None,
            latency_cycle: 0,
            gpu_cycle: 0,
            sensor_cycle: 0,
            process_cycle: 0,
            cpu_frequency_cycle: 0,
            last_gpu_refresh: None,
            last_sensor_refresh: None,
            last_process_refresh: None,
        };

        let cpu = monitor.get_cpu_metrics();

        assert_eq!(cpu.core_count, 0);
        assert_eq!(cpu.per_core_usage, Vec::<f32>::new());
        assert_eq!(cpu.usage_percent, 0.0);
        assert_eq!(cpu.user_percent, 0.0);
        assert_eq!(cpu.system_percent, 0.0);
        assert_eq!(cpu.idle_percent, 0.0);
        assert_eq!(cpu.steal_percent, 0.0);
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
            cpu_user_percent: 0.0,
            cpu_system_percent: 0.0,
            cpu_idle_percent: 0.0,
            cpu_steal_percent: 0.0,
            disk_read_rates: std::collections::HashMap::new(),
            disk_write_rates: std::collections::HashMap::new(),
            cached_gpus: vec![],
            cached_sensors: None,
            cached_top_processes: None,
            cached_gateway_ip: None,
            cached_gateway_latency_ms: None,
            latency_cycle: 0,
            gpu_cycle: 0,
            sensor_cycle: 0,
            process_cycle: 0,
            cpu_frequency_cycle: 0,
            last_gpu_refresh: None,
            last_sensor_refresh: None,
            last_process_refresh: None,
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
        assert!(sensors.fans.iter().all(|sensor| sensor
            .duty_percent
            .map(|value| value.is_finite() && (0.0..=100.0).contains(&value))
            .unwrap_or(true)));
        assert!(sensors
            .voltages
            .iter()
            .all(|sensor| sensor.volts.is_finite()));
        assert!(sensors
            .currents
            .iter()
            .all(|sensor| sensor.amps.is_finite()));
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
        assert!(!metrics.system_info.hostname.is_empty());
        assert!(!metrics.system_info.os_name.is_empty());
        assert!(!metrics.system_info.kernel_version.is_empty());
        assert!(!metrics.system_info.architecture.is_empty());
        assert!(metrics.system_info.process_count > 0);
        assert!(metrics
            .sensors
            .temperatures
            .iter()
            .all(|sensor| sensor.temperature_celsius.is_finite()));
    }
}
