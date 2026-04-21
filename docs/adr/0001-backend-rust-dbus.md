# 0001 — Backend Rust com interface DBus

## Status

aceito

## Contexto

O projeto precisa coletar métricas do sistema Linux (CPU, RAM, disco, rede, sensores) e exibi-las num widget KDE Plasma. As alternativas consideradas foram:

- **Python puro no QML via subprocess**: simples, mas sem tipagem, lento para cálculos de delta e frágil em distribuições com versões diferentes de Python.
- **C++ nativo KDE**: integração nativa, porém maior curva de desenvolvimento e sem ecossistema de crates para sysinfo.
- **Rust + DBus (escolhido)**: binário único, compilado, com crates maduros (`sysinfo`, `zbus`), tipagem forte e acesso seguro a `/sys` e `/proc`.

O KDE Plasma expõe `Plasma5Support.DataSource` com engine `"executable"` que permite chamar `gdbus` do QML — tornando a comunicação DBus transparente para o plasmoid sem necessidade de bindings QML/C++ customizados.

## Decisão

Adotamos um binário Rust standalone que:
1. Coleta métricas periodicamente com delta de 200 ms
2. Expõe `GetMetricsJson()` via Session DBus usando `zbus 4`
3. Roda como serviço `systemd --user`

O Plasmoid QML chama `gdbus` a cada 1500 ms via `Plasma5Support.DataSource`, deserializa o JSON e atualiza o estado local.

## Consequências

- (+) Backend isolado: falhas no Rust não derrubam o Plasma
- (+) Dados fortemente tipados em `SystemMetrics`; QML trata como objeto JS dinâmico
- (+) Binário pode ser testado independentemente com `--json`
- (-) Requer `gdbus` disponível no sistema (presente em todas as distros com GLib)
- (-) Latência mínima de 1500 ms entre atualizações (aceitável para monitoramento)
- (-) Parsing do output gdbus (`('...',)`) é frágil; tratado em `extractJsonPayload()`
