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
| [Desenvolvimento](Development) | Fluxo de desenvolvimento, Makefile, testes e convenções |
| [Arquitetura](Architecture) | Visão geral da arquitetura e links para a referência técnica |
| [Roadmap](Roadmap) | Funcionalidades implementadas e próximas prioridades |
| [Troubleshooting](Troubleshooting) | Problemas comuns e soluções |

---

## O que é

Widget para o painel do KDE Plasma que exibe métricas do sistema em tempo real. O binário Rust coleta os dados e os expõe via **DBus** em snapshots rápidos/lentos; o caminho rápido usa cache quente no backend, e o Plasmoid QML consome essa interface por um cliente DBus persistente e renderiza a UI com 7 abas.

| Aba | Métricas principais |
|---|---|
| **CPU** | Uso total, user/system/idle/steal, load average, histórico e temperatura principal da CPU |
| **RAM** | RAM usada/total, swap e histórico |
| **GPU** | Uso, VRAM, clocks, temperatura, potência e fan da GPU (RPM e duty% quando disponível) |
| **Disk** | Uso por partição, I/O de leitura/escrita em tempo real e histórico |
| **Network** | Download/upload instantâneo, histórico, interfaces, gateway padrão, latência e teste manual de velocidade |
| **Sensors** | Temperaturas por chip, fans, tensão, corrente e potência |
| **System** | Hostname, kernel, load average e top processos por CPU |

---

## Como a documentação está organizada

- **`wiki/`** → visão geral para usuários e contribuidores
- **`docs/`** → referência técnica detalhada do repositório

Referência técnica no repositório:

- [`docs/index.md`](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/index.md)
- [`docs/models.md`](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/models.md)
- [`docs/backend.md`](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/backend.md)
- [`docs/frontend.md`](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/frontend.md)

---

## Status atual

✅ Release `v0.1.1` publicada

Veja a [página de releases](https://github.com/marcos2872/rust-monitor-tray/releases) para o binário mais recente.
