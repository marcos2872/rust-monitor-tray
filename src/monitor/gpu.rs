use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::{GpuInfo, GpuVendor};

static NVIDIA_PROBE_FAILED: AtomicBool = AtomicBool::new(false);
static INTEL_USAGE_CACHE: OnceLock<Mutex<HashMap<String, IntelUsageSample>>> = OnceLock::new();

const BYTES_TO_GB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Clone, Copy)]
struct IntelUsageSample {
    collected_at: Instant,
    rc6_residency_ms: u64,
}

// ---------------------------------------------------------------------------
// Leitura de arquivos sysfs
// ---------------------------------------------------------------------------

fn read_trimmed(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_u64(path: &Path) -> Option<u64> {
    read_trimmed(path)?.parse().ok()
}

/// Lê miligraus Celsius e converte para Celsius.
fn read_millidegrees(path: &Path) -> Option<f32> {
    let raw: f32 = read_trimmed(path)?.parse().ok()?;
    (raw / 1000.0).is_finite().then_some(raw / 1000.0)
}

/// Lê microwatts e converte para Watts.
fn read_microwatts(path: &Path) -> Option<f32> {
    let raw: f32 = read_trimmed(path)?.parse().ok()?;
    (raw / 1_000_000.0).is_finite().then_some(raw / 1_000_000.0)
}

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_TO_GB
}

fn read_first_u64(paths: &[PathBuf]) -> Option<u64> {
    paths.iter().find_map(|path| read_u64(path))
}

fn intel_usage_cache() -> &'static Mutex<HashMap<String, IntelUsageSample>> {
    INTEL_USAGE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// ---------------------------------------------------------------------------
// Detecção de vendor e nome
// ---------------------------------------------------------------------------

/// Identifica o fabricante da GPU a partir do driver exposto em uevent.
fn detect_vendor(card_path: &Path) -> GpuVendor {
    let content = std::fs::read_to_string(card_path.join("device/uevent")).unwrap_or_default();
    for line in content.lines() {
        if let Some(driver) = line.strip_prefix("DRIVER=") {
            return match driver.trim() {
                "amdgpu" | "radeon" => GpuVendor::Amd,
                "nvidia" => GpuVendor::Nvidia,
                "i915" | "xe" => GpuVendor::Intel,
                _ => GpuVendor::Unknown,
            };
        }
    }
    GpuVendor::Unknown
}

/// Monta nome descritivo da GPU usando o driver e o identificador do card.
fn detect_gpu_name(card_path: &Path, vendor: &GpuVendor) -> String {
    let prefix = match vendor {
        GpuVendor::Amd => "AMD GPU",
        GpuVendor::Nvidia => "NVIDIA GPU",
        GpuVendor::Intel => "Intel GPU",
        GpuVendor::Unknown => "GPU",
    };
    let card_id = card_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("card");
    format!("{prefix} ({card_id})")
}

/// Localiza o diretório hwmon dentro do dispositivo DRM
/// (ex.: `/sys/class/drm/card1/device/hwmon/hwmon2`).
fn find_device_hwmon(dev_path: &Path) -> Option<PathBuf> {
    let hwmon_dir = dev_path.join("hwmon");
    std::fs::read_dir(&hwmon_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.is_dir())
}

// ---------------------------------------------------------------------------
// AMD (amdgpu / radeon) — métricas via sysfs
// ---------------------------------------------------------------------------

/// Lê o clock ativo de um arquivo `pp_dpm_*` (linha marcada com `*`).
/// Formato de cada linha: `"2: 952Mhz *"`
fn read_active_clock_mhz(path: &Path) -> Option<u64> {
    let content = read_trimmed(path)?;
    for line in content.lines() {
        if !line.contains('*') {
            continue;
        }
        for token in line.split_whitespace() {
            if let Some(mhz_str) = token.strip_suffix("Mhz") {
                return mhz_str.parse().ok();
            }
        }
    }
    None
}

fn collect_amd_gpu(card_path: &Path) -> GpuInfo {
    let dev = card_path.join("device");

    let usage_percent =
        read_u64(&dev.join("gpu_busy_percent")).map(|v| (v as f32).clamp(0.0, 100.0));

    let vram_used = read_u64(&dev.join("mem_info_vram_used"));
    let vram_total = read_u64(&dev.join("mem_info_vram_total"));
    let vram_used_gb = vram_used.map(bytes_to_gb);
    let vram_total_gb = vram_total.map(bytes_to_gb);
    let vram_usage_percent = vram_used.zip(vram_total).and_then(|(u, t)| {
        if t > 0 {
            Some((u as f32 / t as f32) * 100.0)
        } else {
            None
        }
    });

    let shader_clock_mhz = read_active_clock_mhz(&dev.join("pp_dpm_sclk"));
    let memory_clock_mhz = read_active_clock_mhz(&dev.join("pp_dpm_mclk"));

    let (temperature_celsius, power_watts, fan_rpm, fan_duty_percent) =
        if let Some(hwmon) = find_device_hwmon(&dev) {
            let rpm = read_u64(&hwmon.join("fan1_input"));
            let duty = rpm
                .and(read_u64(&hwmon.join("pwm1")))
                .map(|v| (v as f32 / 255.0 * 100.0).clamp(0.0, 100.0));
            (
                read_millidegrees(&hwmon.join("temp1_input")),
                read_microwatts(&hwmon.join("power1_input")),
                rpm,
                duty,
            )
        } else {
            (None, None, None, None)
        };

    GpuInfo {
        name: detect_gpu_name(card_path, &GpuVendor::Amd),
        vendor: GpuVendor::Amd,
        usage_percent,
        vram_used_gb,
        vram_total_gb,
        vram_usage_percent,
        shader_clock_mhz,
        memory_clock_mhz,
        temperature_celsius,
        power_watts,
        fan_rpm,
        fan_duty_percent,
    }
}

// ---------------------------------------------------------------------------
// Intel (i915 / xe) — métricas via sysfs + estimativa por RC6
// ---------------------------------------------------------------------------

/// Estima uso da iGPU Intel a partir do delta do contador RC6.
///
/// O kernel incrementa `rc6_residency_ms` enquanto a GPU está ociosa em RC6.
/// Assim, o tempo fora de RC6 é uma boa aproximação do tempo ocupado.
fn estimate_intel_usage_from_rc6(
    previous_rc6_ms: u64,
    current_rc6_ms: u64,
    elapsed: Duration,
) -> Option<f32> {
    if current_rc6_ms < previous_rc6_ms {
        return None;
    }

    let elapsed_ms = elapsed.as_secs_f32() * 1000.0;
    if elapsed_ms < 1.0 {
        return None;
    }

    let idle_ms = (current_rc6_ms - previous_rc6_ms) as f32;
    let idle_ratio = (idle_ms / elapsed_ms).clamp(0.0, 1.0);
    Some(((1.0 - idle_ratio) * 100.0).clamp(0.0, 100.0))
}

fn collect_intel_usage_percent(card_path: &Path) -> Option<f32> {
    let rc6_enabled = read_first_u64(&[
        card_path.join("power/rc6_enable"),
        card_path.join("gt/gt0/power/rc6_enable"),
    ])?;
    if rc6_enabled == 0 {
        return None;
    }

    let rc6_residency_ms = read_first_u64(&[
        card_path.join("power/rc6_residency_ms"),
        card_path.join("gt/gt0/power/rc6_residency_ms"),
    ])?;

    let key = card_path.to_string_lossy().to_string();
    let now = Instant::now();
    let mut cache = intel_usage_cache().lock().ok()?;
    let previous = cache.insert(
        key,
        IntelUsageSample {
            collected_at: now,
            rc6_residency_ms,
        },
    )?;

    estimate_intel_usage_from_rc6(
        previous.rc6_residency_ms,
        rc6_residency_ms,
        now.duration_since(previous.collected_at),
    )
}

fn collect_intel_gpu(card_path: &Path) -> GpuInfo {
    let shader_clock_mhz = read_first_u64(&[
        card_path.join("gt/gt0/rps_cur_freq_mhz"),
        card_path.join("gt/gt0/rps_act_freq_mhz"),
        card_path.join("gt_cur_freq_mhz"),
        card_path.join("gt_act_freq_mhz"),
    ]);

    let temperature_celsius = find_device_hwmon(&card_path.join("device"))
        .and_then(|hwmon| read_millidegrees(&hwmon.join("temp1_input")));

    GpuInfo {
        name: detect_gpu_name(card_path, &GpuVendor::Intel),
        vendor: GpuVendor::Intel,
        usage_percent: collect_intel_usage_percent(card_path),
        vram_used_gb: None, // UMA — memória compartilhada com a RAM
        vram_total_gb: None,
        vram_usage_percent: None,
        shader_clock_mhz,
        memory_clock_mhz: None,
        temperature_celsius,
        power_watts: None,
        fan_rpm: None,
        fan_duty_percent: None,
    }
}

// ---------------------------------------------------------------------------
// NVIDIA — métricas via nvidia-smi (subprocess assíncrono)
// ---------------------------------------------------------------------------

/// Analisa uma linha CSV do nvidia-smi com as colunas:
/// name, utilization.gpu, memory.used, memory.total,
/// clocks.gr, clocks.mem, temperature.gpu, power.draw
fn parse_nvidia_csv_line(line: &str) -> Option<GpuInfo> {
    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.len() < 8 {
        return None;
    }

    let name = parts[0].to_string();
    let usage_percent = parts[1].parse::<f32>().ok().map(|v| v.clamp(0.0, 100.0));
    let vram_used_mib = parts[2].parse::<f64>().ok();
    let vram_total_mib = parts[3].parse::<f64>().ok();
    let shader_clock_mhz = parts[4].parse::<u64>().ok();
    let memory_clock_mhz = parts[5].parse::<u64>().ok();
    let temperature_celsius = parts[6].parse::<f32>().ok();
    let power_watts = parts[7].parse::<f32>().ok();

    let vram_used_gb = vram_used_mib.map(|v| v / 1024.0);
    let vram_total_gb = vram_total_mib.map(|v| v / 1024.0);
    let vram_usage_percent = vram_used_mib.zip(vram_total_mib).and_then(|(u, t)| {
        if t > 0.0 {
            Some((u / t * 100.0) as f32)
        } else {
            None
        }
    });

    Some(GpuInfo {
        name,
        vendor: GpuVendor::Nvidia,
        usage_percent,
        vram_used_gb,
        vram_total_gb,
        vram_usage_percent,
        shader_clock_mhz,
        memory_clock_mhz,
        temperature_celsius,
        power_watts,
        fan_rpm: None,
        fan_duty_percent: None,
    })
}

async fn collect_nvidia_gpus() -> Vec<GpuInfo> {
    let result = tokio::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,memory.used,memory.total,\
             clocks.gr,clocks.mem,temperature.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .await;

    match result {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(parse_nvidia_csv_line)
                .collect()
        }
        _ => vec![],
    }
}

fn gpu_sort_key(gpu: &GpuInfo) -> (u8, u8, String) {
    let dedicated_rank = if gpu.vram_total_gb.is_some() { 0 } else { 1 };
    let vendor_rank = match gpu.vendor {
        GpuVendor::Nvidia => 0,
        GpuVendor::Amd => 1,
        GpuVendor::Intel => 2,
        GpuVendor::Unknown => 3,
    };

    (dedicated_rank, vendor_rank, gpu.name.to_ascii_lowercase())
}

fn sort_gpus_by_preference(gpus: &mut [GpuInfo]) {
    gpus.sort_by_key(gpu_sort_key);
}

// ---------------------------------------------------------------------------
// Ponto de entrada público
// ---------------------------------------------------------------------------

/// Detecta e coleta métricas de todas as GPUs disponíveis no sistema.
/// Suporta AMD e Intel via sysfs; NVIDIA via subprocess `nvidia-smi`.
pub async fn collect_gpu_metrics() -> Vec<GpuInfo> {
    let mut gpus: Vec<GpuInfo> = vec![];
    let mut found_nvidia_drm = false;

    // Varre /sys/class/drm/cardN para AMD e Intel
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        let mut cards: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let s = name.to_string_lossy();
                s.starts_with("card") && s.len() > 4 && s[4..].parse::<u32>().is_ok()
            })
            .collect();
        cards.sort_by_key(|e| e.file_name());

        for entry in cards {
            let card_path = entry.path();
            match detect_vendor(&card_path) {
                GpuVendor::Amd => gpus.push(collect_amd_gpu(&card_path)),
                GpuVendor::Intel => gpus.push(collect_intel_gpu(&card_path)),
                GpuVendor::Nvidia => found_nvidia_drm = true,
                GpuVendor::Unknown => {}
            }
        }
    }

    // NVIDIA: evita subprocesso recorrente em máquinas sem driver/GPU NVIDIA.
    if found_nvidia_drm || !NVIDIA_PROBE_FAILED.load(AtomicOrdering::Relaxed) {
        let nvidia = collect_nvidia_gpus().await;
        if found_nvidia_drm || !nvidia.is_empty() {
            gpus.extend(nvidia);
        } else {
            NVIDIA_PROBE_FAILED.store(true, AtomicOrdering::Relaxed);
        }
    }

    sort_gpus_by_preference(&mut gpus);
    gpus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu(name: &str, vendor: GpuVendor, vram_total_gb: Option<f64>) -> GpuInfo {
        GpuInfo {
            name: name.to_string(),
            vendor,
            usage_percent: None,
            vram_used_gb: None,
            vram_total_gb,
            vram_usage_percent: None,
            shader_clock_mhz: None,
            memory_clock_mhz: None,
            temperature_celsius: None,
            power_watts: None,
            fan_rpm: None,
            fan_duty_percent: None,
        }
    }

    #[test]
    fn test_estimate_intel_usage_from_rc6_returns_busy_percentage() {
        let usage = estimate_intel_usage_from_rc6(100, 550, Duration::from_millis(1000));

        assert!(usage
            .map(|value| (value - 55.0).abs() < 0.01)
            .unwrap_or(false));
    }

    #[test]
    fn test_estimate_intel_usage_from_rc6_clamps_idle_above_elapsed() {
        let usage = estimate_intel_usage_from_rc6(100, 1400, Duration::from_millis(1000));

        assert_eq!(usage, Some(0.0));
    }

    #[test]
    fn test_parse_nvidia_csv_line_reads_usage_and_vram() {
        let gpu = parse_nvidia_csv_line("GeForce RTX 4060, 37, 2048, 8188, 2100, 8001, 65, 78.5")
            .expect("should parse nvidia-smi csv line");

        assert_eq!(gpu.name, "GeForce RTX 4060");
        assert_eq!(gpu.vendor, GpuVendor::Nvidia);
        assert_eq!(gpu.usage_percent, Some(37.0));
        assert_eq!(gpu.shader_clock_mhz, Some(2100));
        assert_eq!(gpu.memory_clock_mhz, Some(8001));
        assert_eq!(gpu.temperature_celsius, Some(65.0));
        assert_eq!(gpu.power_watts, Some(78.5));
        assert!(gpu
            .vram_used_gb
            .map(|value| (value - 2.0).abs() < 0.01)
            .unwrap_or(false));
        assert!(gpu
            .vram_total_gb
            .map(|value| (value - 7.996).abs() < 0.01)
            .unwrap_or(false));
        assert!(gpu
            .vram_usage_percent
            .map(|value| (value - 25.012215).abs() < 0.01)
            .unwrap_or(false));
    }

    #[test]
    fn test_sort_gpus_by_preference_puts_dedicated_gpu_first() {
        let mut gpus = vec![
            gpu("Intel Graphics", GpuVendor::Intel, None),
            gpu("GeForce RTX 4060", GpuVendor::Nvidia, Some(8.0)),
            gpu("AMD Radeon", GpuVendor::Amd, Some(12.0)),
        ];

        sort_gpus_by_preference(&mut gpus);

        assert_eq!(gpus[0].vendor, GpuVendor::Nvidia);
        assert_eq!(gpus[1].vendor, GpuVendor::Amd);
        assert_eq!(gpus[2].vendor, GpuVendor::Intel);
    }
}
