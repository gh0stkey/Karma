SHELL := /bin/bash
.DEFAULT_GOAL := all

UNAME_S := $(shell uname -s)
DEPS :=
ifeq ($(UNAME_S),Darwin)
DEPS += fetch-mlx
endif

.PHONY: fetch-mlx
fetch-mlx:
	@echo "==> Ensuring MLX runtime (macOS)..."
	bash src-tauri/vendor/fetch-mlx-dist.sh

.PHONY: app
app: $(DEPS)
	@echo "==> Building Tauri app..."
	npm run tauri build
	@echo "==> Done. Output is in src-tauri/target/release/bundle/"

.PHONY: all
all: app

.PHONY: dev
dev: $(DEPS)
	npm run tauri dev

.PHONY: test
test:
	cd src-tauri && cargo test

.PHONY: clean
clean:
	cd src-tauri && cargo clean
	@echo "==> Cleaned build artifacts"

.PHONY: distclean
distclean: clean
	rm -rf src-tauri/vendor/mlx-dist
	@echo "==> Removed downloaded MLX runtime"
