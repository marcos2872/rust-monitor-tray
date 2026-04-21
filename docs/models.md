# Modelos de dados — Monitor Tray

Todos os modelos são definidos em `src/monitor/models.rs` e derivam `Serialize` / `Deserialize` com `serde`.

O backend hoje expõe três payloads JSON principais via DBus:

- `GetMetricsJson` → snapshot completo legado (`SystemMetrics`)
- `FastMetricsJson` → snapshot quente (`FastMetrics`)
- `SlowMetricsJson` → snapshot lento (`SlowMetrics`)

---

## FastMetrics

Payload quente usado no polling frequente do frontend.

| Campo | Tipo | Descrição |
|---|---|---|
| `cpu` | `CpuMetrics` | Métricas de CPU |
| `memory` | `MemoryMetrics` | Métricas de memória RAM e swap |
| `disk` | `DiskMetrics` | Métricas de armazenamento |
| `network` | `NetworkMetrics` | Métricas de rede |
| `uptime` | `u64` | Segundos desde o boot |
| `load_average` | `(f64, f64, f64)` | Load average de 1, 5 e 15 minutos |

---

## SlowMetrics

Payload lento usado para métricas mais caras ou menos voláteis.

| Campo | Tipo | Descrição |
|---|---|---|
| `sensors` | `SensorMetrics` | Sensores de hardware |
| `gpus` | `Vec<GpuInfo>` | Lista de GPUs detectadas |
| `top_processes` | `Vec<ProcessInfo>` | Top 15 processos por uso de CPU |
| `system_info` | `SystemInfo` | Informações do sistema operacional |

---

## NetworkSpeedTestStatus

Payload do fluxo manual de speed test exposto por `GetNetworkSpeedTestStatusJson`.

| Campo | Tipo | Descrição |
|---|---|---|
| `state` | `NetworkSpeedTestState` | `idle`, `running`, `success`, `error` ou `cancelled` |
| `phase` | `NetworkSpeedTestPhase` | fase interna: `idle`, `preparing`, `running`, `parsing`, `done`, `cancelled` |
| `tool` | `Option<String>` | CLI utilizada (`speedtest` ou `speedtest-cli`) |
| `ping_ms` | `Option<f32>` | Ping do teste em milissegundos |
| `download_mbps` | `Option<f32>` | Download em Mbps |
| `upload_mbps` | `Option<f32>` | Upload em Mbps |
| `server_name` | `Option<String>` | Nome do servidor do teste |
| `server_location` | `Option<String>` | Localização/host do servidor |
| `started_at_unix_ms` | `Option<u64>` | Início do teste em epoch ms |
| `finished_at_unix_ms` | `Option<u64>` | Fim do teste em epoch ms |
| `error` | `Option<String>` | Mensagem de erro amigável |

---

## SystemMetrics

Raiz do payload JSON completo legado.

| Campo | Tipo | Descrição |
|---|---|---|
| `cpu` | `CpuMetrics` | Métricas de CPU |
| `memory` | `MemoryMetrics` | Métricas de memória RAM e swap |
| `disk` | `DiskMetrics` | Métricas de armazenamento |
| `network` | `NetworkMetrics` | Métricas de rede |
| `sensors` | `SensorMetrics` | Sensores de hardware |
| `gpus` | `Vec<GpuInfo>` | Lista de GPUs detectadas |
| `top_processes` | `Vec<ProcessInfo>` | Top 15 processos por uso de CPU |
| `system_info` | `SystemInfo` | Informações do sistema operacional |
| `uptime` | `u64` | Segundos desde o boot |
| `load_average` | `(f64, f64, f64)` | Load average de 1, 5 e 15 minutos |

---

## CpuMetrics

| Campo | Tipo | Unidade | Descrição |
|---|---|---|---|
| `usage_percent` | `f32` | % | Uso total médio do sistema |
| `user_percent` | `f32` | % | Tempo em espaço de usuário |
| `system_percent` | `f32` | % | Tempo em kernel |
| `idle_percent` | `f32` | % | Tempo ocioso |
| `steal_percent` | `f32` | % | Tempo roubado por hipervisor |
| `core_count` | `usize` | — | Número de núcleos lógicos |
| `per_core_usage` | `Vec<f32>` | % | Uso individual por core |
| `frequency` | `u64` | MHz | Frequência do primeiro core |
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
| `disks` | `Vec<DiskInfo>` | — | Lista de partições/montagens |
| `total_space` | `f64` | GB | Soma do espaço total |
| `used_space` | `f64` | GB | Soma do espaço usado |
| `available_space` | `f64` | GB | Soma do espaço disponível |
| `total_read_bytes_per_sec` | `u64` | B/s | Taxa agregada de leitura |
| `total_write_bytes_per_sec` | `u64` | B/s | Taxa agregada de escrita |

### DiskInfo

| Campo | Tipo | Unidade | Descrição |
|---|---|---|---|
| `name` | `String` | — | Nome do dispositivo |
| `mount_point` | `String` | — | Ponto de montagem |
| `total_space` | `f64` | GB | Capacidade total |
| `available_space` | `f64` | GB | Espaço disponível |
| `used_space` | `f64` | GB | `total - available` |
| `usage_percent` | `f32` | % | `used / total × 100` |
| `read_bytes_per_sec` | `u64` | B/s | Taxa atual de leitura |
| `write_bytes_per_sec` | `u64` | B/s | Taxa atual de escrita |

---

## NetworkMetrics

| Campo | Tipo | Descrição |
|---|---|---|
| `interfaces` | `HashMap<String, NetworkInterface>` | Mapa `nome → dados da interface` |
| `total_bytes_received` | `u64` | Bytes recebidos acumulados desde o boot |
| `total_bytes_transmitted` | `u64` | Bytes enviados acumulados desde o boot |
| `gateway_ip` | `Option<String>` | IP do gateway padrão detectado em `/proc/net/route` |
| `gateway_latency_ms` | `Option<f32>` | Latência ICMP para o gateway em milissegundos |

### NetworkInterface

| Campo | Tipo | Descrição |
|---|---|---|
| `bytes_received` | `u64` | Bytes recebidos acumulados |
| `bytes_transmitted` | `u64` | Bytes enviados acumulados |
| `packets_received` | `u64` | Pacotes recebidos |
| `packets_transmitted` | `u64` | Pacotes enviados |
| `errors_received` | `u64` | Erros de recepção |
| `errors_transmitted` | `u64` | Erros de transmissão |
| `is_up` | `bool` | `true` quando `operstate` é `up` ou `unknown` |

---

## SensorMetrics

| Campo | Tipo | Descrição |
|---|---|---|
| `temperatures` | `Vec<TemperatureSensor>` | Sensores térmicos |
| `average_temperature_celsius` | `Option<f32>` | Média de todas as temperaturas |
| `hottest_temperature_celsius` | `Option<f32>` | Maior temperatura global |
| `hottest_label` | `String` | Label do sensor mais quente global |
| `hottest_cpu_celsius` | `Option<f32>` | Maior temperatura entre sensores de CPU |
| `hottest_cpu_label` | `String` | Label do sensor de CPU mais quente |
| `hottest_gpu_celsius` | `Option<f32>` | Maior temperatura entre sensores de GPU |
| `hottest_gpu_label` | `String` | Label do sensor de GPU mais quente |
| `fans` | `Vec<FanSensor>` | Ventiladores expostos em hwmon |
| `voltages` | `Vec<VoltageSensor>` | Tensões expostas em hwmon |
| `currents` | `Vec<CurrentSensor>` | Correntes expostas em hwmon |
| `powers` | `Vec<PowerSensor>` | Potências expostas em hwmon |

### TemperatureSensor

| Campo | Tipo | Descrição |
|---|---|---|
| `label` | `String` | Nome do sensor |
| `chip` | `String` | Nome do chip hwmon |
| `temperature_celsius` | `f32` | Temperatura atual em °C |
| `max_celsius` | `Option<f32>` | Limite máximo quando exposto |
| `critical_celsius` | `Option<f32>` | Limite crítico quando exposto |

### FanSensor

| Campo | Tipo | Descrição |
|---|---|---|
| `label` | `String` | Nome do ventilador |
| `rpm` | `u64` | Rotações por minuto |
| `duty_percent` | `Option<f32>` | Duty cycle do fan em `%` |

### VoltageSensor

| Campo | Tipo | Descrição |
|---|---|---|
| `label` | `String` | Nome do sensor |
| `volts` | `f32` | Tensão em volts |

### CurrentSensor

| Campo | Tipo | Descrição |
|---|---|---|
| `label` | `String` | Nome do sensor |
| `amps` | `f32` | Corrente em ampères |

### PowerSensor

| Campo | Tipo | Descrição |
|---|---|---|
| `label` | `String` | Nome do sensor |
| `watts` | `f32` | Potência em watts |

---

## ProcessInfo

Representa um processo individual em `top_processes`.

| Campo | Tipo | Unidade | Descrição |
|---|---|---|---|
| `pid` | `u32` | — | PID do processo |
| `name` | `String` | — | Nome do processo |
| `cpu_percent` | `f32` | % | Uso de CPU **normalizado para 0–100% do sistema total** |
| `memory_mb` | `f64` | MB | Uso de memória residente |

---

## GpuInfo

| Campo | Tipo | Unidade | Disponibilidade |
|---|---|---|---|
| `name` | `String` | — | Sempre |
| `vendor` | `GpuVendor` | — | Sempre (`amd`, `nvidia`, `intel`, `unknown`) |
| `usage_percent` | `Option<f32>` | % | AMD ✅ NVIDIA ✅ Intel ❌ |
| `vram_used_gb` | `Option<f64>` | GB | AMD ✅ NVIDIA ✅ Intel ❌ |
| `vram_total_gb` | `Option<f64>` | GB | AMD ✅ NVIDIA ✅ Intel ❌ |
| `vram_usage_percent` | `Option<f32>` | % | AMD ✅ NVIDIA ✅ Intel ❌ |
| `shader_clock_mhz` | `Option<u64>` | MHz | AMD ✅ NVIDIA ✅ Intel ⚠️ |
| `memory_clock_mhz` | `Option<u64>` | MHz | AMD ✅ NVIDIA ✅ Intel ❌ |
| `temperature_celsius` | `Option<f32>` | °C | AMD ✅ NVIDIA ✅ Intel ⚠️ |
| `power_watts` | `Option<f32>` | W | AMD ✅ NVIDIA ✅ Intel ❌ |
| `fan_rpm` | `Option<u64>` | RPM | AMD ✅ NVIDIA ❌ Intel ❌ |
| `fan_duty_percent` | `Option<f32>` | % | AMD ✅ NVIDIA ❌ Intel ❌ |

### GpuVendor

Enum serializado em `snake_case`:

- `amd`
- `nvidia`
- `intel`
- `unknown`

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
