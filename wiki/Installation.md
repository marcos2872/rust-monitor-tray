# Instalação

---

## Opção 1 — One-liner (recomendado)

Não requer Rust instalado. Baixa o binário pré-compilado da última release.

```bash
curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh
```

**Pré-requisitos:**

| Ferramenta | Verificar |
|---|---|
| KDE Plasma 6 (ou 5) | `plasmashell --version` |
| `kpackagetool6` ou `kpackagetool5` | `which kpackagetool6` |
| `systemctl --user` | `systemctl --user status` |
| `gdbus` | `which gdbus` |
| `curl` | `which curl` |

---

## Opção 2 — A partir do código-fonte

Requer Rust/Cargo instalado.

```bash
git clone https://github.com/marcos2872/rust-monitor-tray.git
cd rust-monitor-tray
./install-kde.sh
```

---

## O que o instalador faz

1. **Compila** o backend Rust em release (ou baixa binário pré-compilado)
2. **Instala o binário** em `~/.local/bin/monitor-tray`
3. **Instala o plasmoid** via `kpackagetool` em `~/.local/share/plasma/plasmoids/`
4. **Instala o ícone** em `~/.local/share/icons/hicolor/256x256/apps/` e atualiza o cache KDE
5. **Cria e ativa** o serviço `systemd --user` para o backend DBus
6. **Recarrega** o `plasmashell`

Após a instalação, adicione o widget "Monitor Tray" ao painel pelo menu **Adicionar Widgets** do KDE.

---

## Remoção

```bash
./uninstall-kde.sh
```

Ou, se instalou via curl (sem ter o repositório local):

```bash
# para o serviço
systemctl --user disable --now monitor-tray.service

# remove os arquivos
rm -f ~/.local/bin/monitor-tray
rm -f ~/.config/systemd/user/monitor-tray.service
rm -f ~/.local/share/icons/hicolor/256x256/apps/com.monitortray.plasmoid.png
kpackagetool6 --type Plasma/Applet --remove com.monitortray.plasmoid

# atualiza o cache
kbuildsycoca6 --noincremental
```

---

## Instalar o Rust (se necessário)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```
