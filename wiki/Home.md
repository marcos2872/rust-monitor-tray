# Monitor Tray

Widget de monitoramento de sistema para **KDE Plasma** — backend Rust + DBus + Plasmoid.

<p align="center">
  <img src="https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/assets/icon.png" width="96"/>
</p>

---

## Instalação rápida

```bash
curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh
```

> Não requer Rust instalado. Baixa o binário pré-compilado da última release.

---

## Navegação

| Página | Descrição |
|---|---|
| [Instalação](Installation) | Instalação rápida, manual e a partir do código-fonte |
| [Desenvolvimento](Development) | Fluxo de desenvolvimento, Makefile, testes, lint |
| [Arquitetura](Architecture) | Visão geral técnica, C4, fluxo de dados |
| [Roadmap](Roadmap) | Funcionalidades planejadas e status atual |
| [Troubleshooting](Troubleshooting) | Problemas comuns e soluções |

---

## O que é

Widget para o painel do KDE Plasma que exibe métricas do sistema em tempo real. O binário Rust coleta os dados e os expõe via **DBus**; o Plasmoid QML consome essa interface e renderiza a UI com 7 abas.

| Aba | Métricas |
|---|---|
| **CPU** | Uso total, user/system/idle/steal, load average, histórico, por núcleo |
| **RAM** | Usada/total, swap, histórico |
| **GPU** | Uso, VRAM, clocks, temperatura, potência, fan — AMD, NVIDIA e Intel |
| **Disk** | Uso por partição, I/O read/write em tempo real, histórico |
| **Network** | Download/upload instantâneo, histórico, status das interfaces |
| **Sensors** | Temperaturas por chip (CPU/GPU/NVMe), fans, tensão, corrente, potência |
| **System** | Hostname, OS, kernel, arquitetura, processos, load avg, resumo geral |

---

## Status atual

✅ **v0.1.0** — primeira release pública  
Veja a [página de releases](https://github.com/marcos2872/rust-monitor-tray/releases) para o binário mais recente.
