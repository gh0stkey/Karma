SHELL := /bin/bash
.DEFAULT_GOAL := all

SIDECAR_SRC   := src-tauri/sidecar-src
BINARIES_DIR  := src-tauri/binaries

UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
  TARGET_TRIPLE := aarch64-apple-darwin
  SIDECAR_PKG   := opf_mlx
  PYPROJECT     := pyproject.toml
else ifeq ($(UNAME_S),Linux)
  TARGET_TRIPLE := x86_64-unknown-linux-gnu
  SIDECAR_PKG   := opf_onnx
  PYPROJECT     := pyproject_onnx.toml
else
  TARGET_TRIPLE := x86_64-pc-windows-msvc
  SIDECAR_PKG   := opf_onnx
  PYPROJECT     := pyproject_onnx.toml
endif

SIDECAR_NAME  := opf-mlx-$(TARGET_TRIPLE)

.PHONY: sidecar
sidecar:
	@echo "==> Building sidecar [$(SIDECAR_PKG)] for $(TARGET_TRIPLE)..."
	cd $(SIDECAR_SRC) && \
		UV_PROJECT_FILE=$(PYPROJECT) uv run pyinstaller \
			--onefile \
			--name opf-mlx \
			--collect-all $(SIDECAR_PKG) \
			--collect-all opf_common \
			$(if $(filter opf_mlx,$(SIDECAR_PKG)),--collect-all mlx --collect-all mlx_embeddings,) \
			--console \
			$(SIDECAR_PKG)/__main__.py
	@mkdir -p $(BINARIES_DIR)
	cp $(SIDECAR_SRC)/dist/opf-mlx $(BINARIES_DIR)/$(SIDECAR_NAME)
	rm -rf $(SIDECAR_SRC)/dist $(SIDECAR_SRC)/build $(SIDECAR_SRC)/*.spec
	@echo "==> Sidecar binary: $(BINARIES_DIR)/$(SIDECAR_NAME)"

.PHONY: app
app:
	@echo "==> Building Tauri app..."
	npm run tauri build
	@echo "==> Done. Output is in src-tauri/target/release/bundle/"

.PHONY: all
all: sidecar app

.PHONY: clean
clean:
	rm -rf $(SIDECAR_SRC)/dist $(SIDECAR_SRC)/build $(SIDECAR_SRC)/*.spec
	rm -rf $(BINARIES_DIR)
	@echo "==> Cleaned build artifacts"

.PHONY: sidecar-dev
sidecar-dev:
	@echo "==> Creating dev wrapper for sidecar [$(SIDECAR_PKG)]..."
	@mkdir -p $(BINARIES_DIR)
	@echo '#!/bin/bash' > $(BINARIES_DIR)/$(SIDECAR_NAME)
	@echo 'cd "$(CURDIR)/$(SIDECAR_SRC)" && UV_PROJECT_FILE=$(PYPROJECT) uv run python -m $(SIDECAR_PKG) "$$@"' >> $(BINARIES_DIR)/$(SIDECAR_NAME)
	@chmod +x $(BINARIES_DIR)/$(SIDECAR_NAME)
	@echo "==> Dev wrapper: $(BINARIES_DIR)/$(SIDECAR_NAME)"
