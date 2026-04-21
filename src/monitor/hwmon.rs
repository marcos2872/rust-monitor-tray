use std::fs;
use std::path::Path;

use super::{CurrentSensor, FanSensor, PowerSensor, TemperatureSensor, VoltageSensor};

pub(crate) const HWMON_BASE_PATH: &str = "/sys/class/hwmon";
const MILLI_SCALE: f32 = 1000.0;
const MICRO_SCALE: f32 = 1_000_000.0;
const PWM_MAX_VALUE: f32 = 255.0;

#[derive(Default)]
pub(crate) struct HwmonMetrics {
    pub(crate) temperatures: Vec<TemperatureSensor>,
    pub(crate) fans: Vec<FanSensor>,
    pub(crate) voltages: Vec<VoltageSensor>,
    pub(crate) currents: Vec<CurrentSensor>,
    pub(crate) powers: Vec<PowerSensor>,
}

/// Lê um arquivo texto, remove espaços nas extremidades e retorna `None` se vazio.
fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
}

/// Lê e converte o conteúdo de um arquivo para `u64`.
fn read_u64(path: &Path) -> Option<u64> {
    read_trimmed(path)?.parse().ok()
}

/// Lê um valor numérico de um arquivo e divide por `scale` (ex.: milli → base).
fn read_scaled_f32(path: &Path, scale: f32) -> Option<f32> {
    let value: f32 = read_trimmed(path)?.parse().ok()?;
    let scaled = value / scale;
    scaled.is_finite().then_some(scaled)
}

/// Extrai o índice numérico de um nome de arquivo de sensor.
/// Exemplo: `parse_sensor_index("fan2_input", "fan", "_input")` → `Some("2")`.
pub(crate) fn parse_sensor_index(file_name: &str, prefix: &str, suffix: &str) -> Option<String> {
    if !file_name.starts_with(prefix) || !file_name.ends_with(suffix) {
        return None;
    }

    let index = &file_name[prefix.len()..file_name.len() - suffix.len()];
    if index.is_empty() || !index.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    Some(index.to_string())
}

/// Converte identificadores técnicos para exibição: underscores e hifens → espaços.
fn prettify_identifier(value: &str) -> String {
    value
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Retorna o nome legível do chip hwmon a partir do arquivo `name` do diretório.
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

/// Monta o rótulo completo de um sensor no formato `"chip: rótulo"` ou
/// `"chip: TipoFallback índice"` quando não há arquivo `*_label`.
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

/// Coleta todas as métricas de sensores disponíveis em `base_path`.
/// Lê temperatura, ventiladores, tensão, corrente e potência de cada chip hwmon.
pub(crate) fn collect_hwmon_metrics_from_path(base_path: &Path) -> HwmonMetrics {
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

            if let Some(index) = parse_sensor_index(&file_name, "temp", "_input") {
                if let Some(celsius) = read_scaled_f32(&file_path, MILLI_SCALE) {
                    if celsius.is_finite() {
                        let chip  = hwmon_chip_name(&path);
                        let label = read_trimmed(&path.join(format!("temp{index}_label")))
                            .map(|l| prettify_identifier(&l))
                            .unwrap_or_else(|| format!("Temp {index}"));
                        let max_celsius = read_scaled_f32(
                            &path.join(format!("temp{index}_max")), MILLI_SCALE,
                        );
                        let critical_celsius = read_scaled_f32(
                            &path.join(format!("temp{index}_crit")), MILLI_SCALE,
                        );
                        metrics.temperatures.push(TemperatureSensor {
                            label,
                            chip,
                            temperature_celsius: celsius,
                            max_celsius,
                            critical_celsius,
                        });
                    }
                }
                continue;
            }

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

    metrics.temperatures.sort_by(|a, b| {
        a.chip.cmp(&b.chip)
            .then_with(|| a.label.cmp(&b.label))
    });
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
