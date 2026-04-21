# Modelos de Dados — Monitor Tray

Todos os modelos são definidos em `src/monitor/models.rs` e derivam `Serialize` / `Deserialize` (serde). O payload completo é serializado como JSON pelo método DBus `GetMetricsJson`.

---

## SystemMetrics

Raiz do payload JSON.

| Campo | Tipo | Descrição |
|---|---|---|
| `cpu` | `CpuMetrics` | Métricas de CPU |
| `memory` | `MemoryMetrics` | Métricas de memória RAM e swap |
| `disk` | `DiskMetrics` | Métricas de armazenamento |
| `network` | `NetworkMetrics` | Métricas de rede |
| `sensors` | `SensorMetrics` | Sensores de hardware |
| `gpus` | `Vec<GpuInfo>` | Lista de GPUs detectadas |
| `system_info` | `SystemInfo` | Informações do sistema operacional |
| `uptime` | `u64` | Segundos desde o boot |
| `load_average` | `(f64, f64, f64)` | Load avg 1 min, 5 min, 15 min |

---

## CpuMetrics

| Campo | Tipo | Unidade | Descrição |
|---|---|---|---|
| `usage_percent` | `f32` | % | Uso total (média de todos os cores) |
| `user_percent` | `f32` | % | Uso em espaço de usuário (delta /proc/stat) |
| `system_percent` | `f32` | % | Uso do kernel (delta /proc/stat) |
| `idle_percent` | `f32` | % | Tempo ocioso (idle + iowait) |
| `steal_percent` | `f32` | % | Tempo roubado por hipervisor (VMs) |
| `core_count` | `usize` | — | Número de cores lógicos |
| `per_core_usage` | `Vec<f32>` | % | Uso individual por core |
| `frequency` | `u64` | MHz | Frequência atual do primeiro core |
| `name` | `String` | — | Modelo do processador |

---

## MemoryMetrics

| Campo | Tipo | Unidade | Descrição |
|---|---|---|---|
| `total_memory` | `f64` | GB | RAM total |
| `used_memory` | `f64` | GB | RAM em uso |
| `available_memory` | `f64` | GB | RAM disponível |
| `usage_percent` | `f32` | % | `used / total × 100` |
| `total_swap` | `f64` | GB | Swap total |
| `used_swap` | `f64` | GB | Swap em uso |

---

## DiskMetrics

| Campo | Tipo | Unidade | Descrição |
|---|---|---|---|
| `disks` | `Vec<DiskInfo>` | — | Lista de partições |
| `total_space` | `f64` | GB | Soma do espaço total de todas as partições |
| `used_space` | `f64` | GB | Soma do espaço usado |
| `available_space` | `f64` | GB | Soma do espaço disponível |
| `total_read_bytes_per_sec` | `u64` | B/s | Taxa de leitura agregada |
| `total_write_bytes_per_sec` | `u64` | B/s | Taxa de escrita agregada |

### DiskInfo

| Campo | Tipo | Unidade | Descrição |
|---|---|---|---|
| `name` | `String` | — | Nome do dispositivo (ex.: `/dev/nvme0n1p3`) |
| `mount_point` | `String` | — | Ponto de montagem (ex.: `/`, `/home`) |
| `total_space` | `f64` | GB | Capacidade total |
| `available_space` | `f64` | GB | Espaço disponível |
| `used_space` | `f64` | GB | `total - available` |
| `usage_percent` | `f32` | % | `used / total × 100` |
| `read_bytes_per_sec` | `u64` | B/s | Taxa de leitura atual |
| `write_bytes_per_sec` | `u64` | B/s | Taxa de escrita atual |

---

## NetworkMetrics

| Campo | Tipo | Descrição |
|---|---|---|
| `interfaces` | `HashMap<String, NetworkInterface>` | Mapa `nome → dados` |
| `total_bytes_received` | `u64` | Total recebido (bytes acumulados desde o boot) |
| `total_bytes_transmitted` | `u64` | Total enviado (bytes acumulados desde o boot) |

### NetworkInterface

| Campo | Tipo | Descrição |
|---|---|---|
| `bytes_received` | `u64` | Bytes recebidos acumulados |
| `bytes_transmitted` | `u64` | Bytes enviados acumulados |
| `packets_received` | `u64` | Pacotes recebidos |
| `packets_transmitted` | `u64` | Pacotes enviados |
| `errors_received` | `u64` | Erros de recepção |
| `errors_transmitted` | `u64` | Erros de transmissão |
| `is_up` | `bool` | `true` se `operstate` é `"up"` ou `"unknown"` |

---

## SensorMetrics

| Campo | Tipo | Descrição |
|---|---|---|
| `temperatures` | `Vec<TemperatureSensor>` | Sensores térmicos (hwmon ou sysinfo) |
| `average_temperature_celsius` | `Option<f32>` | Média de todas as temperaturas |
| `hottest_temperature_celsius` | `Option<f32>` | Temperatura mais alta encontrada |
| `hottest_label` | `String` | Nome do sensor mais quente |
| `fans` | `Vec<FanSensor>` | Ventiladores expostos em hwmon |
| `voltages` | `Vec<VoltageSensor>` | Tensões expostas em hwmon |
| `currents` | `Vec<CurrentSensor>` | Correntes expostas em hwmon |
| `powers` | `Vec<PowerSensor>` | Potências expostas em hwmon |

### TemperatureSensor

| Campo | Tipo | Descrição |
|---|---|---|
| `label` | `String` | Nome do sensor (ex.: `"Core 0"`, `"edge"`) |
| `chip` | `String` | Nome do chip hwmon (ex.: `"coretemp"`, `"amdgpu"`) |
| `temperature_celsius` | `f32` | Temperatura atual em °C |
| `max_celsius` | `Option<f32>` | Limite máximo (quando exposto) |
| `critical_celsius` | `Option<f32>` | Limite crítico (quando exposto) |

---

## GpuInfo

| Campo | Tipo | Unidade | Disponibilidade |
|---|---|---|---|
| `name` | `String` | — | Sempre |
| `vendor` | `GpuVendor` | — | Sempre (`amd`/`nvidia`/`intel`/`unknown`) |
| `usage_percent` | `Option<f32>` | % | AMD ✅ NVIDIA ✅ Intel ❌ |
| `vram_used_gb` | `Option<f64>` | GB | AMD ✅ NVIDIA ✅ Intel ❌ (UMA) |
| `vram_total_gb` | `Option<f64>` | GB | AMD ✅ NVIDIA ✅ Intel ❌ (UMA) |
| `vram_usage_percent` | `Option<f32>` | % | AMD ✅ NVIDIA ✅ Intel ❌ |
| `shader_clock_mhz` | `Option<u64>` | MHz | AMD ✅ NVIDIA ✅ Intel ⚠️ (kernel ≥ 5.16) |
| `memory_clock_mhz` | `Option<u64>` | MHz | AMD ✅ NVIDIA ✅ Intel ❌ |
| `temperature_celsius` | `Option<f32>` | °C | AMD ✅ NVIDIA ✅ Intel ⚠️ |
| `power_watts` | `Option<f32>` | W | AMD ✅ NVIDIA ✅ Intel ❌ |
| `fan_rpm` | `Option<u64>` | RPM | AMD ✅ NVIDIA ❌ Intel ❌ |

---

## SystemInfo

| Campo | Tipo | Exemplo |
|---|---|---|
| `hostname` | `String` | `"fedora"` |
| `os_name` | `String` | `"Fedora Linux"` |
| `os_version` | `String` | `"43"` |
| `kernel_version` | `String` | `"6.19.12-200.fc43.x86_64"` |
| `architecture` | `String` | `"x86_64"` |
| `process_count` | `usize` | `1566` |
