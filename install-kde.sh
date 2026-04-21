#!/usr/bin/env bash
set -euo pipefail

APP_NAME="monitor-tray"
PLASMOID_DIR="plasma"
PLASMOID_ID="com.monitortray.plasmoid"
BIN_DIR="${HOME}/.local/bin"
BIN_PATH="${BIN_DIR}/${APP_NAME}"
SERVICE_DIR="${HOME}/.config/systemd/user"
SERVICE_PATH="${SERVICE_DIR}/${APP_NAME}.service"
RELOAD_PLASMA=1

usage() {
  cat <<EOF
Instala o monitor-tray para KDE/Plasma no usuário atual.

Uso:
  ./install-kde.sh [--no-plasma-reload] [--bin-dir <diretório>]

Opções:
  --no-plasma-reload   Não reinicia o plasmashell ao final
  --bin-dir DIR        Diretório de instalação do binário (padrão: ~/.local/bin)
  -h, --help           Exibe esta ajuda
EOF
}

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Erro: comando não encontrado: $command_name" >&2
    exit 1
  fi
}

find_kpackagetool() {
  if command -v kpackagetool6 >/dev/null 2>&1; then
    echo "kpackagetool6"
  elif command -v kpackagetool5 >/dev/null 2>&1; then
    echo "kpackagetool5"
  else
    echo ""
  fi
}

reload_plasma() {
  if [[ "$RELOAD_PLASMA" -eq 0 ]]; then
    return 0
  fi

  if ! command -v plasmashell >/dev/null 2>&1; then
    echo "Aviso: plasmashell não encontrado; pulando reload do Plasma."
    return 0
  fi

  echo "↻ Recarregando plasmashell..."
  kquitapp6 plasmashell >/dev/null 2>&1 || kquitapp5 plasmashell >/dev/null 2>&1 || true
  nohup plasmashell >/dev/null 2>&1 &
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-plasma-reload)
      RELOAD_PLASMA=0
      shift
      ;;
    --bin-dir)
      if [[ $# -lt 2 ]]; then
        echo "Erro: --bin-dir exige um valor" >&2
        exit 1
      fi
      BIN_DIR="$2"
      BIN_PATH="${BIN_DIR}/${APP_NAME}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Erro: opção inválida: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_command cargo
require_command systemctl
require_command gdbus

KPACKAGETOOL="$(find_kpackagetool)"
if [[ -z "$KPACKAGETOOL" ]]; then
  echo "Erro: kpackagetool5 ou kpackagetool6 não encontrado." >&2
  exit 1
fi

if [[ ! -d "$PLASMOID_DIR" ]]; then
  echo "Erro: diretório do plasmoid não encontrado: $PLASMOID_DIR" >&2
  exit 1
fi

echo "🔨 Compilando backend Rust em release..."
cargo build --release

echo "📦 Instalando binário em $BIN_PATH..."
mkdir -p "$BIN_DIR"
install -Dm755 "target/release/${APP_NAME}" "$BIN_PATH"

echo "🧩 Instalando/atualizando plasmoid $PLASMOID_ID..."
"$KPACKAGETOOL" --type Plasma/Applet --upgrade "$PLASMOID_DIR" >/dev/null 2>&1 || \
  "$KPACKAGETOOL" --type Plasma/Applet --install "$PLASMOID_DIR"

echo "🖼️  Instalando ícone no tema hicolor do usuário..."
ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
mkdir -p "$ICON_DIR"
cp "${PLASMOID_DIR}/contents/icons/com.monitortray.plasmoid.png" \
   "${ICON_DIR}/com.monitortray.plasmoid.png"
if command -v kbuildsycoca6 >/dev/null 2>&1; then
  kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
elif command -v kbuildsycoca5 >/dev/null 2>&1; then
  kbuildsycoca5 --noincremental >/dev/null 2>&1 || true
fi

echo "📝 Instalando serviço systemd do usuário em $SERVICE_PATH..."
mkdir -p "$SERVICE_DIR"
cat > "$SERVICE_PATH" <<EOF
[Unit]
Description=Monitor Tray DBus Backend
After=graphical-session.target
Wants=graphical-session.target

[Service]
ExecStart=${BIN_PATH} --dbus
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
EOF

echo "🚀 Ativando backend DBus do usuário..."
systemctl --user daemon-reload
systemctl --user enable --now "${APP_NAME}.service"

reload_plasma

cat <<EOF

✅ Instalação concluída.

Resumo:
- Binário: $BIN_PATH
- Serviço systemd: $SERVICE_PATH
- Plasmoid: $PLASMOID_ID

Verificações úteis:
  systemctl --user status ${APP_NAME}.service
  gdbus call --session --dest com.monitortray.Backend --object-path /com/monitortray/Backend --method com.monitortray.Backend.Ping

Próximo passo no KDE:
- Adicione o widget "Monitor Tray" ao painel pelo menu de widgets.
EOF
