# 0002 — Monitoramento de GPU via sysfs e nvidia-smi

## Status

aceito

## Contexto

Monitorar GPUs no Linux sem depender de bibliotecas proprietárias ou crates com linking dinâmico obrigatório. As opções avaliadas foram:

- **`nvml-wrapper` crate**: wrapper da NVML da NVIDIA. Funciona bem para NVIDIA, mas faz link dinâmico com `libnvidia-ml.so` — o binário falha ao carregar em sistemas sem driver NVIDIA instalado.
- **subprocess `nvidia-smi`**: presente em qualquer sistema com driver NVIDIA instalado; parse simples de CSV; fallback gracioso se não encontrado. Sem linking dinâmico.
- **Sysfs direto** (`/sys/class/drm/`): disponível para AMD e Intel sem dependências; leitura de arquivos texto simples.

## Decisão

Usamos **três backends independentes**:

1. **AMD** (`amdgpu`/`radeon`): leitura direta de `/sys/class/drm/cardN/device/` — `gpu_busy_percent`, `mem_info_vram_*`, `pp_dpm_sclk/mclk`, hwmon para temperatura/potência/fan.
2. **Intel** (`i915`/`xe`): leitura de `/sys/class/drm/cardN/gt/gt0/rps_cur_freq_mhz` para clock e hwmon para temperatura quando disponível.
3. **NVIDIA**: subprocess assíncrono (`tokio::process::Command`) executando `nvidia-smi --format=csv,noheader,nounits`; fallback silencioso se não encontrado.

A detecção é automática via `/sys/class/drm/cardN/device/uevent` (campo `DRIVER=`).

## Consequências

- (+) Zero dependências extras no `Cargo.toml`
- (+) Binário carrega normalmente em qualquer Linux, independente de GPU presente
- (+) AMD tem cobertura completa (uso, VRAM, clocks, temp, potência, fan)
- (+) NVIDIA funciona em qualquer versão de driver que inclua `nvidia-smi`
- (-) Intel tem cobertura limitada: sem uso%, sem VRAM (UMA), clock só em kernel ≥ 5.16
- (-) NVIDIA via subprocess: latência extra (~50-150ms) e dependência do PATH
- (-) Sem suporte a múltiplas GPUs NVIDIA via sysfs (apenas via nvidia-smi)

Ver também: [0001 — Backend Rust com interface DBus](0001-backend-rust-dbus.md)
