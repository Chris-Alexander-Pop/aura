#!/usr/bin/env bash
# Build hyprtoolkit (git) + hyprpolkitagent (git) into ~/.config/ags/hypr/dist/
# Arch extra/hyprpolkitagent 0.1.3 is Qt/QML; git main is hyprtoolkit-native.
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
DIST="$AURA_DIR/hypr/dist"
BUILD_ROOT="${XDG_CACHE_HOME:-$HOME/.cache}/aura-hypr-polkit-build"
JOBS="${JOBS:-$(nproc 2>/dev/null || echo 4)}"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: missing $1" >&2
    exit 1
  }
}

need git
need cmake
need pkg-config
need make

for pkg in hyprutils hyprgraphics hyprlang pixman-1 libdrm sdbus-c++; do
  pkg-config --exists "$pkg" 2>/dev/null || {
    echo "error: pkg-config missing $pkg (install Hyprland build deps)" >&2
    exit 1
  }
done

clone_or_pull() {
  local url="$1"
  local dir="$2"
  if [[ -d "$dir/.git" ]]; then
    git -C "$dir" fetch --depth 1 origin
    git -C "$dir" checkout -f FETCH_HEAD
  else
    mkdir -p "$(dirname "$dir")"
    git clone --depth 1 "$url" "$dir"
  fi
}

clone_or_pull https://github.com/hyprwm/hyprtoolkit.git "$BUILD_ROOT/hyprtoolkit"
clone_or_pull https://github.com/hyprwm/hyprpolkitagent.git "$BUILD_ROOT/hyprpolkitagent"

PATCH_DIR="$AURA_DIR/hypr/patches"
if [[ -d "$PATCH_DIR" ]]; then
  for patch in "$PATCH_DIR"/hyprtoolkit-*.patch; do
    [[ -f "$patch" ]] || continue
    echo "==> applying $(basename "$patch") (hyprtoolkit)"
    git -C "$BUILD_ROOT/hyprtoolkit" apply --check "$patch"
    git -C "$BUILD_ROOT/hyprtoolkit" apply "$patch"
  done
  for patch in "$PATCH_DIR"/polkit-*.patch; do
    [[ -f "$patch" ]] || continue
    echo "==> applying $(basename "$patch") (hyprpolkitagent)"
    git -C "$BUILD_ROOT/hyprpolkitagent" apply --check "$patch"
    git -C "$BUILD_ROOT/hyprpolkitagent" apply "$patch"
  done
fi

rm -rf "$DIST"
mkdir -p "$DIST"

echo "==> hyprtoolkit -> $DIST"
cmake -S "$BUILD_ROOT/hyprtoolkit" -B "$BUILD_ROOT/hyprtoolkit/build" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$DIST"
cmake --build "$BUILD_ROOT/hyprtoolkit/build" -j"$JOBS"
cmake --install "$BUILD_ROOT/hyprtoolkit/build"

echo "==> hyprpolkitagent -> $DIST"
export PKG_CONFIG_PATH="$DIST/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
cmake -S "$BUILD_ROOT/hyprpolkitagent" -B "$BUILD_ROOT/hyprpolkitagent/build" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$DIST" \
  -DCMAKE_INSTALL_RPATH="$DIST/lib" \
  -DCMAKE_BUILD_WITH_INSTALL_RPATH=ON
cmake --build "$BUILD_ROOT/hyprpolkitagent/build" -j"$JOBS"
cmake --install "$BUILD_ROOT/hyprpolkitagent/build"

BIN="$DIST/libexec/hyprpolkitagent"
if [[ ! -x "$BIN" ]]; then
  echo "error: expected binary at $BIN" >&2
  exit 1
fi

if ldd "$BIN" | grep -qi qt; then
  echo "error: built binary still links Qt — wrong source tree?" >&2
  exit 1
fi

echo "OK: $BIN (hyprtoolkit-native, no Qt)"
echo "Restart: systemctl --user restart hyprpolkitagent"
