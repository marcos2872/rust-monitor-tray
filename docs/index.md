# Documentação técnica — Monitor Tray

Referência técnica do projeto. Esta pasta documenta **o contrato de dados**, **os detalhes de implementação** e **os fluxos internos** do backend Rust e do frontend QML.

> Para visão geral de contribuição, instalação e navegação do projeto, consulte a `wiki/`.

## Índice

| Documento | Descrição |
|---|---|
| [Arquitetura](architecture.md) | Visão técnica detalhada, C4, fluxo de dados e inventário de módulos |
| [Backend](backend.md) | Interface DBus, ciclo de coleta, fontes de dados Linux e decisões de implementação |
| [Modelos de dados](models.md) | Referência completa do payload JSON serializado em `GetMetricsJson` |
| [Frontend](frontend.md) | Estado global em QML, polling DBus, histórico e comportamento das abas |
| [Componentes](components.md) | Referência dos componentes reutilizáveis em `plasma/contents/ui/components/` |

## Quando atualizar esta pasta

Atualize `docs/` quando houver mudanças em:

- contrato JSON entre backend e plasmoid;
- coleta de métricas, fontes Linux ou subprocessos usados pelo backend;
- estrutura das abas, estado global ou fluxo do frontend;
- ADRs e decisões arquiteturais.

## Decisões de Arquitetura (ADRs)

| ADR | Título | Status |
|---|---|---|
| [0001](adr/0001-backend-rust-dbus.md) | Backend Rust com interface DBus | aceito |
| [0002](adr/0002-gpu-sysfs-nvidia-smi.md) | Monitoramento de GPU via sysfs e nvidia-smi | aceito |
| [0003](adr/0003-systemd-user-service.md) | Serviço systemd do usuário para o backend | aceito |
