<p align="center">
  <img src="assets/icon.png" width="120" alt="Monitor Tray"/>
</p>

<h1 align="center">Monitor Tray</h1>

<p align="center">
  Monitor de sistema para <strong>KDE Plasma</strong> — CPU, RAM, GPU, Disco, Rede, Sensores e Sistema em um widget de painel.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/KDE_Plasma-6-3daee9?logo=kde&logoColor=white"/>
  <img src="https://img.shields.io/badge/Rust-1.70+-f74c00?logo=rust&logoColor=white"/>
  <img src="https://img.shields.io/badge/License-MIT-green"/>
</p>

---

## Instalação

```bash
curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh
```

Para instalar também a dependência opcional do **teste manual de velocidade**:

```bash
curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh -s -- --with-speedtest
```

> Não requer Rust instalado. Baixa o binário pré-compilado da última release.
>
> **Requisitos:** KDE Plasma, `kpackagetool6`, `systemctl --user`, `gdbus`
>
> **Opcional:** `speedtest` ou `speedtest-cli` para habilitar o teste manual de velocidade na aba Network.

Para instalar a partir do código-fonte ou outras opções, consulte o [Wiki](../../wiki) ou o arquivo [`install-kde.sh`](install-kde.sh).
