# Plano: Migração para integração profunda com KDE via Plasmoid

**Data:** 2026-04-20
**Autor:** agente-plan
**Status:** aprovado

---

## Objetivo

Migrar a aplicação atual para uma arquitetura com backend Rust reutilizável e frontend nativo do KDE/Plasma, preferencialmente como Plasmoid, para obter popup ancorado corretamente, visual integrado ao tema Breeze e melhor experiência em Wayland/KWin.

## Escopo

**Dentro do escopo:**
- separar coleta de métricas da UI GTK atual
- definir contrato estável de métricas para consumo externo
- preparar backend Rust reutilizável
- escolher e estruturar comunicação com frontend KDE
- criar base de um frontend KDE nativo
- planejar substituição gradual do tray/menu atuais

**Fora do escopo:**
- suporte a GNOME/XFCE/Unity
- GPU
- persistência de histórico longo
- alertas/notificações avançadas
- painel de configuração completo

---

## Arquivos Afetados

| Arquivo | Ação | Motivo |
|---|---|---|
| `src/monitor.rs` | modificar | virar núcleo reutilizável de coleta |
| `src/main.rs` | modificar ou reduzir drasticamente | deixar de ser UI principal |
| `Cargo.toml` | modificar | adicionar crates da nova integração |
| `src/lib.rs` | criar | expor API interna do backend |
| `src/ipc.rs` ou `src/dbus.rs` | criar | ponte Rust ↔ frontend KDE |
| `plasma/metadata.json` | criar | manifesto do Plasmoid |
| `plasma/contents/ui/main.qml` | criar | UI principal do widget |
| `plasma/contents/ui/Popup.qml` | criar | popup nativo |
| `plasma/contents/config/config.qml` | criar | config básica futura |
| `README.md` | modificar | documentar modo KDE |
| `Makefile` | modificar | targets para plasmoid/dev/install |

---

## Sequência de Execução

### 1. Extrair backend reutilizável
**Arquivos:** `src/monitor.rs`, `src/lib.rs`, `Cargo.toml`
**O que fazer:** mover a lógica de coleta para uma API clara, separada da UI GTK.
**Dependências:** nenhuma

### 2. Definir contrato de métricas
**Arquivos:** `src/monitor.rs`, `src/lib.rs`
**O que fazer:** padronizar structs serializáveis para CPU, memória, disco, rede, uptime e load average.
**Dependências:** passo 1

### 3. Criar camada de comunicação KDE
**Arquivos:** `src/dbus.rs` ou `src/ipc.rs`, `Cargo.toml`
**O que fazer:** expor métricas para a UI KDE; recomendação: DBus com `zbus`.
**Dependências:** passos 1 e 2

### 4. Reduzir `main.rs` a modo legado ou launcher
**Arquivos:** `src/main.rs`
**O que fazer:** transformar o app GTK atual em modo legado temporário ou remover sua responsabilidade principal.
**Dependências:** passo 3

### 5. Criar estrutura do Plasmoid
**Arquivos:** `plasma/metadata.json`, `plasma/contents/ui/main.qml`
**O que fazer:** criar widget do Plasma com item no painel e popup nativo.
**Dependências:** passo 3

### 6. Implementar popup em QML/Kirigami
**Arquivos:** `plasma/contents/ui/main.qml`, `plasma/contents/ui/Popup.qml`
**O que fazer:** construir UI com cards/seções para CPU, memória, disco, rede e sistema.
**Dependências:** passo 5

### 7. Consumir métricas no frontend KDE
**Arquivos:** `plasma/contents/ui/*.qml`, `src/dbus.rs`
**O que fazer:** ler dados do backend e atualizar UI periodicamente.
**Dependências:** passos 3 e 6

### 8. Adicionar ergonomia de desenvolvimento
**Arquivos:** `Makefile`, `README.md`
**O que fazer:** criar comandos para empacotar, instalar e recarregar o Plasmoid em ambiente KDE.
**Dependências:** passos 5 a 7

### 9. Validar substituição do menu atual
**Arquivos:** `README.md`, possivelmente `src/main.rs`
**O que fazer:** decidir se o app GTK antigo será removido, mantido como fallback ou usado apenas em debug.
**Dependências:** passo 8

---

## Riscos e Mitigações

| Risco | Probabilidade | Mitigação |
|---|---|---|
| Complexidade de QML/Kirigami | média | começar com popup simples e iterar |
| DBus adicionar complexidade desnecessária | média | manter interface mínima, só leitura |
| Duplicação temporária entre UI GTK e Plasmoid | alta | assumir fase de transição curta |
| Quebra no fluxo atual de build | média | adicionar targets novos sem remover os antigos de início |
| Integração com Plasma variar por versão | média | testar em Plasma 5/6 e manter escopo inicial enxuto |

---

## Critérios de Conclusão

- [ ] Backend Rust coleta métricas sem depender da UI GTK
- [ ] Métricas expostas por interface estável para o frontend KDE
- [ ] Plasmoid aparece no painel do Plasma
- [ ] Popup abre ancorado e segue o tema do KDE
- [ ] CPU, memória, disco, rede e sistema aparecem no popup
- [ ] `cargo test` continua passando
- [ ] build do projeto continua funcionando
- [ ] README documenta o fluxo KDE
