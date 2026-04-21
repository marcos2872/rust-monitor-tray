## Relatório de QA — Bugs, Segurança e Regras de Negócio
**Data:** 2026-04-21
**Escopo:** repositório completo
**Analista:** Agente QA

---

### 1. Resumo da Funcionalidade Analisada
O projeto implementa um monitor de sistema para KDE Plasma com backend em Rust e frontend em QML.
O backend coleta métricas de CPU, memória, disco, rede, sensores e sistema, expõe os dados via DBus e o plasmoid consulta esse backend para renderizar o widget compacto e a visualização expandida.
Também foram analisados os scripts de instalação e remoção do ambiente KDE.

---

### 2. Resultado dos Testes Automáticos
Comando executado: `make test`

Resultado resumido:
- 10 testes executados
- 10 testes passaram
- 0 falhas

Saída relevante:
- `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

---

### 3. Bugs e Condições de Erro

#### Risco ALTO
Nenhum encontrado.

#### Risco MÉDIO

- [MÉDIO] `src/dbus.rs:33-34`, `src/lib.rs:13,22`, `src/monitor/collector.rs:50-58` — o backend mantém o `Mutex<SystemMonitor>` bloqueado durante toda a coleta, incluindo `sleep(200ms)`.
  Risco: chamadas DBus concorrentes ficam serializadas e acumulam latência; o plasmoid pode aparentar travamento ou lentidão quando houver mais de um consumidor.
  Cenário de reprodução: manter o plasmoid consultando o backend e disparar chamadas paralelas ao método `GetMetricsJson`; cada requisição aguarda a anterior concluir.
  Sugestão: manter um snapshot em cache atualizado em background, ou evitar `await` dentro da seção protegida pelo mutex.

- [MÉDIO] `src/monitor/collector.rs:51-58` — o uso de `refresh(false)` em discos, interfaces e componentes preserva itens removidos da lista.
  Risco: dispositivos hotplug removidos podem continuar aparecendo nas métricas; totais agregados de disco, rede e sensores podem permanecer incorretos até reiniciar o backend.
  Cenário de reprodução: iniciar o serviço, conectar e remover uma interface/disco USB/dispositivo com sensores expostos e consultar novamente as métricas; entradas obsoletas podem continuar presentes.
  Sugestão: usar refresh com remoção de itens não listados quando apropriado, ou reconciliar a lista antes de publicar o snapshot.

- [MÉDIO] `install-kde.sh:122` — o script grava `ExecStart=${BIN_PATH} --dbus` no unit file sem escape/quoting do caminho.
  Risco: a instalação falha quando `--bin-dir` contém espaços ou caracteres especiais; o serviço user do systemd não inicia corretamente.
  Cenário de reprodução: executar `./install-kde.sh --bin-dir "$HOME/Meu Bin"` e tentar iniciar o serviço gerado.
  Sugestão: escapar corretamente o caminho ao gerar o unit file, ou validar/rejeitar caminhos incompatíveis.

#### Risco BAIXO

- [BAIXO] `plasma/contents/ui/FullRepresentation.qml:35` — o estado de loading depende de `metrics.cpu.name === ""`.
  Risco: a interface pode ficar presa em “Coletando métricas...” mesmo quando o backend já retornou dados válidos, se a marca da CPU vier vazia.
  Cenário de reprodução: ambiente em que `sysinfo` retorna `cpu.brand()` vazio.
  Sugestão: usar um flag explícito de “primeira resposta recebida” em vez de inferir pelo nome da CPU.

- [BAIXO] `plasma/contents/ui/tabs/NetworkTab.qml:77,86` — a taxa de rede é exibida com unidade duplicada, porque `fmtBytes()` já retorna unidade e o componente ainda adiciona `unit: "/s"`.
  Risco: a interface mostra valores incorretos, como `12.3 MB /s` ou equivalente com formatação redundante.
  Cenário de reprodução: abrir a aba de rede com tráfego ativo.
  Sugestão: usar `fmtRate()` no valor ou remover `unit: "/s"` do componente `HeroMetric`.

---

### 4. Vulnerabilidades de Segurança
Nenhuma vulnerabilidade de segurança identificada no código analisado.

---

### 5. Falhas na Regra de Negócio
Nenhuma falha evidente de regra de negócio foi identificada no escopo analisado.

---

### 6. Resumo Final
- ALTO: 0
- MÉDIO: 3
- BAIXO: 2

Principais riscos atuais:
1. serialização de chamadas DBus por lock prolongado durante coleta assíncrona;
2. persistência de dispositivos removidos nas métricas por uso de `refresh(false)`;
3. falha de instalação do serviço systemd em caminhos com espaços.
