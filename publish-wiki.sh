#!/usr/bin/env bash
# Publica as páginas do diretório wiki/ no GitHub Wiki do repositório.
#
# Pré-requisito: ativar o Wiki no GitHub antes de rodar este script.
#   1. Acesse: https://github.com/marcos2872/rust-monitor-tray/settings
#   2. Em "Features", marque "Wikis"
#   3. Acesse: https://github.com/marcos2872/rust-monitor-tray/wiki
#   4. Crie a primeira página (qualquer conteúdo) para inicializar o repo do wiki
#   5. Execute este script
set -euo pipefail

REPO="marcos2872/rust-monitor-tray"
WIKI_URL="https://github.com/${REPO}.wiki.git"
WIKI_DIR="$(dirname "$0")/wiki"
TMP_DIR="$(mktemp -d)"

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

echo "📥 Clonando wiki de ${WIKI_URL}..."
git clone "$WIKI_URL" "$TMP_DIR"

echo "📄 Copiando páginas..."
cp "$WIKI_DIR"/*.md "$TMP_DIR/"

cd "$TMP_DIR"
git add -A

if git diff --cached --quiet; then
  echo "ℹ️  Nenhuma mudança para publicar."
  exit 0
fi

git commit -m "docs(wiki): update pages from wiki/ directory"

echo "📤 Enviando para o GitHub Wiki..."
git push

echo "✅ Wiki atualizado: https://github.com/${REPO}/wiki"
