SHELL := /bin/bash

APP_NAME := monitor-tray
CARGO := cargo
CARGO_WATCH := $(CARGO) watch

.PHONY: help build build-autostart test dev run check-tools check-watch install-dev-tools

help:
	@echo "Targets disponíveis:"
	@echo "  make build            - Gera o pacote .deb sem autostart"
	@echo "  make build-autostart  - Gera o pacote .deb com autostart global"
	@echo "  make run              - Executa o app em modo de desenvolvimento"
	@echo "  make test             - Executa a suíte de testes"
	@echo "  make dev              - Executa em modo dev com hot-reload via cargo-watch"
	@echo "  make check-tools      - Verifica se cargo está instalado"
	@echo "  make check-watch      - Verifica se cargo-watch está instalado"
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

build: check-tools
	@chmod +x build.sh
	@./build.sh

build-autostart: check-tools
	@chmod +x build-autostart.sh
	@./build-autostart.sh

run: check-tools
	@$(CARGO) run

test: check-tools
	@$(CARGO) test

dev: check-watch
	@$(CARGO_WATCH) -q -x run
