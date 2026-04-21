SHELL := /bin/bash

APP_NAME := monitor-tray
PLASMOID_ID := com.monitortray.plasmoid
PLASMOID_DIR := plasma
CARGO := cargo
CARGO_WATCH := $(CARGO) watch
KPACKAGETOOL := $(shell command -v kpackagetool6 >/dev/null 2>&1 && echo kpackagetool6 || echo kpackagetool5)

.PHONY: help build test lint qml-lint dev run-json run-dbus check-tools check-watch check-kde-tools install-dev-tools plasmoid-install plasmoid-remove plasmoid-upgrade plasmoid-reload plasmoid-run kde-refresh kde-dev

help:
	@echo "Targets disponíveis:"
	@echo "  make build            - Compila o backend Rust em release"
	@echo "  make test             - Executa a suíte de testes"
	@echo "  make lint             - Executa clippy e valida QML com qmllint"
	@echo "  make qml-lint         - Valida os arquivos QML do plasmoid"
	@echo "  make run-json         - Imprime métricas em JSON"
	@echo "  make run-dbus         - Inicia o backend DBus para o Plasmoid KDE"
	@echo "  make kde-refresh      - Faz build, instala e recarrega o Plasmoid"
	@echo "  make dev              - Hot-reload do backend DBus via cargo-watch"
	@echo "  make plasmoid-install - Instala/atualiza o Plasmoid no Plasma"
	@echo "  make plasmoid-remove  - Remove o Plasmoid do Plasma"
	@echo "  make plasmoid-reload  - Reinicia o plasmashell"
	@echo "  make plasmoid-run     - Faz build, instala e recarrega o Plasmoid"
	@echo "  make kde-dev          - Faz build, instala/recarrega e sobe o backend DBus"
	@echo "  make check-tools      - Verifica se cargo está instalado"
	@echo "  make check-watch      - Verifica se cargo-watch está instalado"
	@echo "  make check-kde-tools  - Verifica ferramentas do KDE/Plasma"
	@echo "  make install-dev-tools - Instala cargo-watch para hot-reload"

check-tools:
	@command -v $(CARGO) >/dev/null 2>&1 || { \
		echo "Erro: cargo não encontrado. Instale o Rust via https://rustup.rs/"; \
		exit 1; \
	}

check-watch: check-tools
	@$(CARGO) watch --version >/dev/null 2>&1 || { \
		echo "Erro: cargo-watch não encontrado."; \
		echo "Instale com: cargo install cargo-watch"; \
		echo "Ou rode: make install-dev-tools"; \
		exit 1; \
	}

install-dev-tools: check-tools
	@$(CARGO) install cargo-watch

check-kde-tools:
	@command -v $(KPACKAGETOOL) >/dev/null 2>&1 || { \
		echo "Erro: kpackagetool5/6 não encontrado."; \
		exit 1; \
	}
	@command -v plasmashell >/dev/null 2>&1 || { \
		echo "Erro: plasmashell não encontrado."; \
		exit 1; \
	}

build: check-tools
	@$(CARGO) build --release

run-json: check-tools
	@$(CARGO) run -- --json

run-dbus: check-tools
	@$(CARGO) run -- --dbus

test: check-tools
	@$(CARGO) test

qml-lint:
	@command -v qmllint >/dev/null 2>&1 || { \
		echo "Erro: qmllint não encontrado."; \
		exit 1; \
	}
	@qmllint plasma/contents/ui/main.qml plasma/contents/ui/FullRepresentation.qml plasma/contents/ui/components/*.qml plasma/contents/ui/tabs/*.qml plasma/contents/ui/Theme.qml

lint: check-tools qml-lint
	@$(CARGO) clippy --version >/dev/null 2>&1 || { \
		echo "Erro: cargo clippy não encontrado."; \
		echo "Instale o componente com: rustup component add clippy"; \
		exit 1; \
	}
	@$(CARGO) clippy --all-targets --all-features -- -D warnings

dev: check-watch
	@$(CARGO_WATCH) -q -x 'run -- --dbus'

plasmoid-install: check-kde-tools
	@$(KPACKAGETOOL) --type Plasma/Applet --upgrade $(PLASMOID_DIR) >/dev/null 2>&1 || \
	 $(KPACKAGETOOL) --type Plasma/Applet --install $(PLASMOID_DIR)

plasmoid-remove: check-kde-tools
	@$(KPACKAGETOOL) --type Plasma/Applet --remove $(PLASMOID_ID) || true

plasmoid-upgrade: plasmoid-install

plasmoid-reload: check-kde-tools
	@kquitapp5 plasmashell >/dev/null 2>&1 || kquitapp6 plasmashell >/dev/null 2>&1 || true
	@nohup plasmashell >/dev/null 2>&1 &

plasmoid-run: check-tools plasmoid-install plasmoid-reload
	@echo "Plasmoid instalado e recarregado."

kde-refresh: check-tools
	@$(CARGO) build
	@$(MAKE) plasmoid-install
	@$(MAKE) plasmoid-reload
	@echo "Build concluído e Plasmoid recarregado no KDE."

kde-dev: kde-refresh
	@echo "Iniciando backend DBus do monitor-tray..."
	@$(CARGO) run -- --dbus
