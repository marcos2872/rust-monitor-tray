# Monitor Tray

Monitor de sistema para **KDE Plasma** com arquitetura **backend Rust + DBus + Plasmoid**.

O binário Rust coleta métricas do sistema e expõe os dados via DBus. O frontend em `plasma/` consome essas métricas para renderizar o widget compacto no painel e o popup expandido com tabs para CPU, RAM, Disk, Network, Sensors e System.

## Visão geral

- **Foco exclusivo em KDE/Plasma**
- **Backend Rust** com coleta via `sysinfo` e leitura de sensores `hwmon`
- **Serviço DBus**: `com.monitortray.Backend`
- **Plasmoid KDE** em `plasma/`
- **Popup expandido com tabs**
- **Histórico local de 5 minutos** para CPU, RAM e Network
- **Sem código legado de GTK/AppIndicator** — o projeto agora é KDE-first

## Modos do binário

```bash
# inicia o backend DBus para o Plasmoid KDE
cargo run -- --dbus

# imprime uma amostra de métricas em JSON
cargo run -- --json

# sem argumentos, também inicia em modo DBus
cargo run
```

## Interface DBus

- **Serviço:** `com.monitortray.Backend`
- **Path:** `/com/monitortray/Backend`
- **Interface:** `com.monitortray.Backend`
- **Métodos:**
  - `Ping`
  - `GetMetricsJson`

Exemplo de teste manual:

```bash
gdbus call --session \
  --dest com.monitortray.Backend \
  --object-path /com/monitortray/Backend \
  --method com.monitortray.Backend.GetMetricsJson
```

## Estrutura do projeto

```text
monitor-tray/
├─ plasma/
│  ├─ metadata.json
│  └─ contents/
│     ├─ config/config.qml
│     └─ ui/
│        ├─ main.qml
│        ├─ CompactRepresentation.qml
│        ├─ FullRepresentation.qml
│        ├─ Theme.qml
│        ├─ components/
│        └─ tabs/
├─ src/
│  ├─ dbus.rs
│  ├─ lib.rs
│  ├─ main.rs
│  └─ monitor/
│     ├─ collector.rs
│     ├─ hwmon.rs
│     ├─ models.rs
│     └─ mod.rs
├─ Makefile
├─ install-kde.sh
└─ Cargo.toml
```

## Dependências de desenvolvimento

### Rust
- Rust 1.70+
- Cargo
- componente `clippy` do Rust (`rustup component add clippy`)

### KDE / Plasma
Ferramentas esperadas no fluxo de desenvolvimento:
- `plasmashell`
- `kpackagetool5` ou `kpackagetool6`
- `gdbus`
- `qmllint` (para `make qml-lint` / `make lint`)

### Linux / sensores
Para melhor cobertura de sensores, o sistema deve expor dados em:
- `/sys/class/hwmon`

## Fluxo sugerido no KDE

```bash
# instala/recarrega o plasmoid e sobe o backend DBus
make kde-dev

# ou, separando os passos:
make kde-refresh
cargo run -- --dbus
```

## Targets úteis do Makefile

```bash
make build         # cargo build --release
make test          # cargo test
make lint          # cargo clippy + qmllint
make qml-lint      # valida os arquivos QML do plasmoid
make run-json      # imprime métricas em JSON
make run-dbus      # sobe o backend DBus
make kde-refresh   # build + instala/recarrega o plasmoid
make kde-dev       # kde-refresh + backend DBus
make dev           # hot reload do backend DBus com cargo-watch
```

> Os antigos scripts de empacotamento e a UI legada de tray foram removidos. O fluxo oficial agora é backend DBus + Plasmoid KDE.

## Sensores suportados hoje

O backend expõe, quando o hardware/drivers publicam os dados:

- **Temperaturas** via `sysinfo::Components`
- **Fans** via `hwmon`
- **Voltage** via `hwmon`
- **Current** via `hwmon`
- **Power** via `hwmon`

Ainda não cobre integralmente:
- **Energy**
- fontes extras como `powercap` / `power_supply`

## Instalação rápida no KDE

```bash
./install-kde.sh
```

O script:
- compila o backend em release
- instala o binário em `~/.local/bin`
- instala/atualiza o plasmoid
- cria e ativa um serviço `systemd --user` para o backend DBus

## Remoção

```bash
./uninstall-kde.sh
```

O script de remoção:
- desativa e remove o serviço `systemd --user`
- remove o binário instalado
- remove o plasmoid do Plasma
- recarrega o `plasmashell`

## Desenvolvimento

### Clonar e rodar

```bash
git clone <repository-url>
cd monitor-tray
make test
make lint
make kde-dev
```

### Validar o backend isoladamente

```bash
cargo run -- --json
cargo run -- --dbus
```

### Build de produção

```bash
cargo build --release
```

## Troubleshooting

### O popup do plasmoid não atualiza
- confirme que o backend DBus está rodando: `cargo run -- --dbus`
- teste o método DBus com `gdbus call`
- recarregue o shell com `make kde-refresh`

### O plasmoid não aparece ou não instala
- confirme que `kpackagetool5` ou `kpackagetool6` está disponível
- confirme que `plasmashell` está instalado
- rode `make plasmoid-install`

### Sensores não aparecem
- a disponibilidade depende do hardware e do que o kernel expõe em `/sys/class/hwmon`
- em algumas máquinas haverá somente temperaturas

## Tecnologias

- **Rust**
- **Tokio**
- **zbus**
- **sysinfo**
- **QML / Plasma**

## Licença

MIT.
