# Documentação — Monitor Tray

Widget de monitoramento de sistema para KDE Plasma com backend Rust + DBus.

## Índice

| Documento | Descrição |
|---|---|
| [Arquitetura](architecture.md) | Visão geral C4, fluxo de dados, inventário de módulos |
| [Backend](backend.md) | Interface DBus, ciclo de atualização, coleta por subsistema, testes |
| [Modelos de dados](models.md) | Referência completa de todos os structs do payload JSON |
| [Frontend](frontend.md) | Plasmoid QML: fluxo, abas, design system |
| [Componentes](components.md) | Referência de todos os componentes reutilizáveis QML |

## Decisões de Arquitetura (ADRs)

| ADR | Título | Status |
|---|---|---|
| [0001](adr/0001-backend-rust-dbus.md) | Backend Rust com interface DBus | aceito |
| [0002](adr/0002-gpu-sysfs-nvidia-smi.md) | Monitoramento de GPU via sysfs e nvidia-smi | aceito |
| [0003](adr/0003-systemd-user-service.md) | Serviço systemd do usuário para o backend | aceito |
