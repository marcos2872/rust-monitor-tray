#!/usr/bin/env bash
# Monitor Tray — instalador one-liner (sem necessidade de Rust)
# Uso: curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh
#       curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh -s -- --with-speedtest
set -euo pipefail

REPO="marcos2872/rust-monitor-tray"
RELEASES_API="https://api.github.com/repos/${REPO}/releases/latest"
TMP_DIR="$(mktemp -d)"

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

# Verifica dependências mínimas
for cmd in curl tar kpackagetool6 kpackagetool5 systemctl gdbus; do
  if command -v "$cmd" >/dev/null 2>&1; then
    continue
  fi
done

if ! command -v kpackagetool6 >/dev/null 2>&1 && ! command -v kpackagetool5 >/dev/null 2>&1; then
  echo "Erro: kpackagetool6 ou kpackagetool5 não encontrado." >&2
  echo "Instale o KDE Plasma SDK antes de continuar." >&2
  exit 1
fi

# Detecta a URL do release mais recente via API do GitHub
echo "🔍 Consultando a versão mais recente..."
RELEASE_JSON="$(curl -fsSL "$RELEASES_API")"
TAG="$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\(.*\)".*/\1/')"

if [[ -z "$TAG" ]]; then
  echo "Erro: não foi possível obter a versão mais recente da API do GitHub." >&2
  echo "Verifique se há releases publicadas em https://github.com/${REPO}/releases" >&2
  exit 1
fi

BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"

echo "⬇️  Baixando monitor-tray ${TAG}..."

# Baixa o binário
curl -fsSL --progress-bar "${BASE_URL}/monitor-tray" -o "${TMP_DIR}/monitor-tray"
chmod +x "${TMP_DIR}/monitor-tray"

# Baixa e extrai o pacote do plasmoid
curl -fsSL --progress-bar "${BASE_URL}/monitor-tray-plasmoid.tar.gz" \
  | tar -xz -C "$TMP_DIR"

# Executa o instalador com o binário pré-compilado
echo "🔨 Instalando..."
cd "$TMP_DIR"
bash install-kde.sh --binary "${TMP_DIR}/monitor-tray" "$@"
