#!/bin/bash
set -euo pipefail

MLX_METAL_VERSION="0.31.1"
DEST="$(cd "$(dirname "$0")" && pwd)/mlx-dist"

if [ -f "$DEST/lib/libmlx.dylib" ] && [ -f "$DEST/lib/mlx.metallib" ]; then
    echo "==> mlx-dist already present ($MLX_METAL_VERSION), skipping"
    exit 0
fi

echo "==> Fetching mlx_metal $MLX_METAL_VERSION wheel from PyPI"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

python3 -m pip download "mlx_metal==$MLX_METAL_VERSION" \
    --no-deps --only-binary :all: --platform macosx_15_0_arm64 \
    --python-version 3.12 \
    -d "$TMP" -q

WHEEL="$(ls "$TMP"/mlx_metal-*.whl | head -1)"
python3 - "$WHEEL" "$TMP/x" <<'EOF'
import sys
import zipfile

zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])
EOF

X="$TMP/x"
rm -rf "$DEST"
mkdir -p "$DEST/lib" "$DEST/share/cmake"

cp "$(find "$X" -name libmlx.dylib | head -1)" "$DEST/lib/"
cp "$(find "$X" -name mlx.metallib | head -1)" "$DEST/lib/"
INCLUDE_DIR="$(dirname "$(dirname "$(find "$X" -name mlx.h -path '*mlx*' | head -1)")")"
cp -R "$INCLUDE_DIR" "$DEST/"
cp -R "$(dirname "$(find "$X" -name MLXConfig.cmake | head -1)")" "$DEST/share/cmake/MLX"

echo "==> Installed to $DEST"
ls -lh "$DEST/lib"
