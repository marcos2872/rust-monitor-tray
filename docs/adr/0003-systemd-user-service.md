# 0003 — Serviço systemd do usuário para o backend

## Status

aceito

## Contexto

O backend Rust precisa estar em execução antes de o Plasmoid iniciar, e deve reiniciar automaticamente em caso de falha. As alternativas foram:

- **Autostart KDE** (`~/.config/autostart/`): simples, mas sem restart automático e sem log estruturado.
- **Daemon no próprio plasmoid** (processo filho do QML): não suportado pela API do Plasma; aumentaria o acoplamento.
- **`systemd --user`** (escolhido): restart automático, logs via `journalctl --user`, ativado junto com a sessão gráfica.

## Decisão

O `install-kde.sh` cria e ativa uma unit `systemd --user`:

```ini
[Unit]
Description=Monitor Tray DBus Backend
After=graphical-session.target

[Service]
ExecStart=~/.local/bin/monitor-tray --dbus
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
```

## Consequências

- (+) Restart automático em até 2s após falha
- (+) `journalctl --user -u monitor-tray` para diagnóstico
- (+) Inicia automaticamente com a sessão do usuário
- (-) Requer `systemd` (padrão em Fedora, Ubuntu, Arch; ausente em algumas distros mínimas)
- (-) O binário precisa estar em `~/.local/bin` ou o `ExecStart` precisa ser ajustado

Ver também: [0001 — Backend Rust com interface DBus](0001-backend-rust-dbus.md)
