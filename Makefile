# Makefile — Karma build pipeline
# Usage:
#   make sidecar      Build PyInstaller sidecar binary → src-tauri/binaries/
#   make sidecar-dev  Create dev wrapper (runs Python directly, no PyInstaller)
#   make app          Build Tauri app (DMG)
#   make all          Full pipeline: sidecar → app
#   make clean        Remove all build artifacts

SHELL := /bin/bash
.DEFAULT_GOAL := all

# ---------- Paths ----------
SIDECAR_SRC   := src-tauri/sidecar-src
BINARIES_DIR  := src-tauri/binaries
TARGET_TRIPLE := aarch64-apple-darwin
SIDECAR_NAME  := opf-mlx-$(TARGET_TRIPLE)

# ---------- Sidecar (PyInstaller) ----------
.PHONY: sidecar
sidecar:
	@echo "==> Building MLX sidecar with PyInstaller..."
	cd $(SIDECAR_SRC) && \
		uv run pyinstaller \
			--onefile \
			--name opf-mlx \
			--collect-all mlx \
			--collect-all mlx_embeddings \
			--console \
			opf_mlx/__main__.py
	@mkdir -p $(BINARIES_DIR)
	cp $(SIDECAR_SRC)/dist/opf-mlx $(BINARIES_DIR)/$(SIDECAR_NAME)
	@# Clean PyInstaller intermediate artifacts
	rm -rf $(SIDECAR_SRC)/dist $(SIDECAR_SRC)/build $(SIDECAR_SRC)/*.spec
	@echo "==> Sidecar binary: $(BINARIES_DIR)/$(SIDECAR_NAME)"

# ---------- Tauri App ----------
.PHONY: app
app:
	@echo "==> Building Tauri app..."
	npm run tauri build
	@echo "==> Done. DMG is in src-tauri/target/release/bundle/dmg/"

# ---------- Full Pipeline ----------
.PHONY: all
all: sidecar app

# ---------- Clean ----------
.PHONY: clean
clean:
	rm -rf $(SIDECAR_SRC)/dist $(SIDECAR_SRC)/build $(SIDECAR_SRC)/*.spec
	rm -rf $(BINARIES_DIR)
	@echo "==> Cleaned build artifacts"

# ---------- Dev helpers ----------
.PHONY: sidecar-dev
sidecar-dev:
	@echo "==> Creating dev wrapper for sidecar..."
	@mkdir -p $(BINARIES_DIR)
	@echo '#!/bin/bash' > $(BINARIES_DIR)/$(SIDECAR_NAME)
	@echo 'cd "$(CURDIR)/$(SIDECAR_SRC)" && uv run python -m opf_mlx "$$@"' >> $(BINARIES_DIR)/$(SIDECAR_NAME)
	@chmod +x $(BINARIES_DIR)/$(SIDECAR_NAME)
	@echo "==> Dev wrapper: $(BINARIES_DIR)/$(SIDECAR_NAME)"
