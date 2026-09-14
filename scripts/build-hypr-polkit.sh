#!/usr/bin/env bash
# Build Aura's hyprtoolkit-native hyprpolkitagent into ~/.config/ags/hypr/dist/
#
# Why this exists: Arch's hyprpolkitagent is still Qt/QML. We ship a git-built
# hyprtoolkit agent. When hyprland-patched (or aquamarine/hyprutils) updates,
# SONAMEs bump and a stale agent exits 127 → polkit prompts vanish.
#
# Usage:
#   ./scripts/build-hypr-polkit.sh              # force rebuild
#   ./scripts/build-hypr-polkit.sh --check      # exit 0 only if agent matches system ABI
#   ./scripts/build-hypr-polkit.sh --ensure     # rebuild only when check fails (idempotent)
#   ./scripts/build-hypr-polkit.sh --restart    # ensure + restart user unit
#   ./scripts/build-hypr-polkit.sh --system-toolkit  # use pacman hyprtoolkit (often too old)
#
# Do not run --ensure from ExecStartPre. cmake of hyprtoolkit is minutes of
# -j$(nproc) cc1plus. systemd TimeoutStartSec defaults to 90s, so the unit
# kills the build, Restart=on-failure starts another, and dist/ is already
# gone (rebuild used to rm -rf it first). Login then melts the CPU forever.
# The user unit ExecStart is scripts/hyprpolkitagent-start.sh (Aura dist or
# Arch Qt fallback). --ensure belongs in hyprland-patched/update.sh.
#
set -euo pipefail

AURA_DIR="${AURA_DIR:-$HOME/.config/ags}"
DIST="$AURA_DIR/hypr/dist"
STAMP="$DIST/.abi-stamp"
BUILD_ROOT="${XDG_CACHE_HOME:-$HOME/.cache}/aura-hypr-polkit-build"
JOBS="${JOBS:-$(nproc 2>/dev/null || echo 4)}"
PATCH_DIR="$AURA_DIR/hypr/patches"
BIN="$DIST/libexec/hyprpolkitagent"

# Prefer bundling hyprtoolkit from git so the agent and toolkit APIs stay paired.
# System hyprtoolkit often lags hyprpolkitagent main (missing setText/setPassword/…).
# Both still link against *system* aquamarine/hyprutils, so SONAME bumps are picked
# up on every rebuild. Use --system-toolkit to force the pacman hyprtoolkit.
USE_SYSTEM_TOOLKIT=0
MODE=rebuild
for arg in "$@"; do
  case "$arg" in
    --check) MODE=check ;;
    --ensure) MODE=ensure ;;
    --restart) MODE=restart ;;
    --bundle-toolkit) USE_SYSTEM_TOOLKIT=0 ;; # default; kept for clarity
    --system-toolkit) USE_SYSTEM_TOOLKIT=1 ;;
    -h|--help)
      sed -n '2,18p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *)
      echo "error: unknown argument: $arg (try --help)" >&2
      exit 1
      ;;
  esac
done

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: missing $1" >&2
    exit 1
  }
}

log() { echo "==> $*"; }
ok() { echo "OK: $*"; }
warn() { echo "warning: $*" >&2; }

# Canonical SONAME path for a library stem (aquamarine, hyprutils, …).
soname_path() {
  local stem=$1
  local link="/usr/lib/lib${stem}.so"
  if [[ -e "$link" ]]; then
    readlink -f "$link"
    return 0
  fi
  # Fall back to the highest numbered lib${stem}.so.*
  local found
  found=$(ls -1 /usr/lib/lib"${stem}".so.* 2>/dev/null | rg -v '\.a$' | sort -V | tail -1 || true)
  [[ -n "$found" ]] || return 1
  readlink -f "$found"
}

# Fingerprint of the Hypr stack the agent must match.
abi_fingerprint() {
  local aq hu ht hg hl
  aq=$(soname_path aquamarine 2>/dev/null || echo missing-aquamarine)
  hu=$(soname_path hyprutils 2>/dev/null || echo missing-hyprutils)
  ht=$(soname_path hyprtoolkit 2>/dev/null || echo missing-hyprtoolkit)
  hg=$(soname_path hyprgraphics 2>/dev/null || echo missing-hyprgraphics)
  hl=$(soname_path hyprlang 2>/dev/null || echo missing-hyprlang)
  local pkgs
  pkgs=$(pacman -Q aquamarine aquamarine-patched hyprutils hyprtoolkit hyprgraphics hyprlang hyprland-patched hyprland 2>/dev/null | sort || true)
  local agent_rev toolkit_rev="system"
  if [[ -d "$BUILD_ROOT/hyprpolkitagent/.git" ]]; then
    agent_rev=$(git -C "$BUILD_ROOT/hyprpolkitagent" rev-parse HEAD 2>/dev/null || echo unknown)
  else
    agent_rev=unbuilt
  fi
  if [[ "$USE_SYSTEM_TOOLKIT" -eq 0 ]]; then
    if [[ -d "$BUILD_ROOT/hyprtoolkit/.git" ]]; then
      toolkit_rev=$(git -C "$BUILD_ROOT/hyprtoolkit" rev-parse HEAD 2>/dev/null || echo unknown)
    else
      toolkit_rev=unbuilt
    fi
  fi
  cat <<EOF
aquamarine=$aq
hyprutils=$hu
hyprtoolkit=$ht
hyprgraphics=$hg
hyprlang=$hl
pkgs:
$pkgs
agent_rev=$agent_rev
toolkit_rev=$toolkit_rev
use_system_toolkit=$USE_SYSTEM_TOOLKIT
EOF
}

agent_link_ok() {
  [[ -x "$BIN" ]] || return 1
  if ldd "$BIN" 2>/dev/null | grep -qi qt; then
    return 1
  fi
  ! ldd "$BIN" 2>/dev/null | grep -q 'not found'
}

stamp_matches() {
  [[ -f "$STAMP" ]] || return 1
  local current
  current=$(abi_fingerprint)
  [[ "$(cat "$STAMP")" == "$current" ]]
}

verify_agent() {
  if ! agent_link_ok; then
    if [[ -x "$BIN" ]]; then
      warn "agent link broken:"
      ldd "$BIN" 2>/dev/null | grep 'not found' || true
    else
      warn "agent missing: $BIN"
    fi
    return 1
  fi
  if ! stamp_matches; then
    warn "ABI stamp stale or missing (system Hypr libs changed since last build)"
    return 1
  fi
  return 0
}

write_stamp() {
  mkdir -p "$DIST"
  abi_fingerprint >"$STAMP"
}

clone_or_pull() {
  local url=$1
  local dir=$2
  if [[ -d "$dir/.git" ]]; then
    # Drop local patch state, then hard-reset to origin/HEAD tip.
    git -C "$dir" fetch --depth 1 origin
    git -C "$dir" reset --hard FETCH_HEAD
    git -C "$dir" clean -fdx
  else
    mkdir -p "$(dirname "$dir")"
    git clone --depth 1 "$url" "$dir"
  fi
}

apply_patches() {
  local repo=$1
  local glob=$2
  local patch
  [[ -d "$PATCH_DIR" ]] || return 0
  shopt -s nullglob
  for patch in "$PATCH_DIR"/$glob; do
    log "applying $(basename "$patch") → $(basename "$repo")"
    if git -C "$repo" apply --check "$patch" 2>/dev/null; then
      git -C "$repo" apply "$patch"
    else
      # Never block an ABI rebuild on a drifted personal patch — agent uptime
      # beats optional UX patches. Rebase patches after upstream moves.
      warn "patch does not apply (skipped): $(basename "$patch")"
      warn "  rebase it under $PATCH_DIR when you can"
    fi
  done
  shopt -u nullglob
}

use_system_toolkit() {
  [[ "$USE_SYSTEM_TOOLKIT" -eq 1 ]] \
    && pkg-config --exists hyprtoolkit \
    && [[ -e /usr/lib/libhyprtoolkit.so ]] \
    && ! ldd /usr/lib/libhyprtoolkit.so 2>/dev/null | grep -q 'not found'
}

rebuild() {
  need git
  need cmake
  need pkg-config
  need make
  need rg

  local lock="${XDG_RUNTIME_DIR:-/tmp}/aura-hypr-polkit-build.lock"
  mkdir -p "$(dirname "$lock")"
  exec 9>"$lock"
  if ! flock -n 9; then
    echo "error: another hyprpolkitagent build is already running" >&2
    exit 1
  fi

  for pkg in hyprutils hyprgraphics hyprlang pixman-1 libdrm sdbus-c++; do
    pkg-config --exists "$pkg" 2>/dev/null || {
      echo "error: pkg-config missing $pkg (install Hyprland stack deps)" >&2
      exit 1
    }
  done

  clone_or_pull https://github.com/hyprwm/hyprpolkitagent.git "$BUILD_ROOT/hyprpolkitagent"
  apply_patches "$BUILD_ROOT/hyprpolkitagent" "polkit-*.patch"

  # Install into a staging prefix. Never rm -rf "$DIST" first: a killed or
  # timed-out build used to leave login with no agent and a restart loop.
  local stage="$BUILD_ROOT/dist-staging"
  rm -rf "$stage"
  mkdir -p "$stage"

  local bundled=0
  local pkg_config_path=""
  if use_system_toolkit; then
    log "using system hyprtoolkit ($(pkg-config --modversion hyprtoolkit)) + system aquamarine/hyprutils"
  else
    bundled=1
    log "bundling hyprtoolkit from git (linked against current system aquamarine/hyprutils)"
    clone_or_pull https://github.com/hyprwm/hyprtoolkit.git "$BUILD_ROOT/hyprtoolkit"
    apply_patches "$BUILD_ROOT/hyprtoolkit" "hyprtoolkit-*.patch"
    rm -rf "$BUILD_ROOT/hyprtoolkit/build"
    cmake -S "$BUILD_ROOT/hyprtoolkit" -B "$BUILD_ROOT/hyprtoolkit/build" \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX="$stage"
    cmake --build "$BUILD_ROOT/hyprtoolkit/build" -j"$JOBS"
    cmake --install "$BUILD_ROOT/hyprtoolkit/build"
    pkg_config_path="$stage/lib/pkgconfig"
    # Sanity: bundled toolkit must resolve against *current* system SONAMEs.
    local toolkit_lib
    toolkit_lib=$(ls "$stage"/lib/libhyprtoolkit.so.* 2>/dev/null | head -1 || true)
    if [[ -z "$toolkit_lib" ]] || ldd "$toolkit_lib" | grep -q 'not found'; then
      echo "error: bundled hyprtoolkit has unresolved libs (aquamarine/hyprutils mismatch?):" >&2
      ldd "$toolkit_lib" 2>/dev/null | grep 'not found' >&2 || true
      exit 1
    fi
  fi

  log "hyprpolkitagent → $DIST (via $stage)"
  # Fresh build dir every time so cmake cannot cache old SONAME paths.
  rm -rf "$BUILD_ROOT/hyprpolkitagent/build"
  local cmake_env=()
  if [[ -n "$pkg_config_path" ]]; then
    cmake_env+=(env "PKG_CONFIG_PATH=${pkg_config_path}${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}")
  fi
  local rpath_args=()
  if [[ "$bundled" -eq 1 ]]; then
    rpath_args+=(
      -DCMAKE_INSTALL_RPATH="$DIST/lib"
      -DCMAKE_BUILD_WITH_INSTALL_RPATH=ON
    )
  fi
  "${cmake_env[@]}" cmake -S "$BUILD_ROOT/hyprpolkitagent" -B "$BUILD_ROOT/hyprpolkitagent/build" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX="$stage" \
    "${rpath_args[@]}"
  cmake --build "$BUILD_ROOT/hyprpolkitagent/build" -j"$JOBS"
  cmake --install "$BUILD_ROOT/hyprpolkitagent/build"

  local staged_bin="$stage/libexec/hyprpolkitagent"
  if [[ ! -x "$staged_bin" ]]; then
    echo "error: expected binary at $staged_bin" >&2
    exit 1
  fi
  # RPATH points at the final DIST/lib, so ldd needs the staging lib dir now.
  local ldd_env=()
  if [[ "$bundled" -eq 1 ]]; then
    ldd_env+=(env "LD_LIBRARY_PATH=${stage}/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}")
  fi
  if "${ldd_env[@]}" ldd "$staged_bin" | grep -qi qt; then
    echo "error: built binary still links Qt — wrong source tree?" >&2
    exit 1
  fi
  if "${ldd_env[@]}" ldd "$staged_bin" | grep -q 'not found'; then
    echo "error: built binary has unresolved libraries:" >&2
    "${ldd_env[@]}" ldd "$staged_bin" | grep 'not found' >&2
    exit 1
  fi

  rm -rf "$DIST"
  mv "$stage" "$DIST"

  write_systemd_dropin "$bundled"
  write_stamp

  ok "$BIN (hyprtoolkit-native, no Qt)"
  log "linked against:"
  ldd "$BIN" | rg 'hyprtoolkit|aquamarine|hyprutils' || true
}

# Keep the user unit on the start wrapper (Aura dist, else Arch Qt).
# Never ExecStartPre --ensure: that compiled hyprtoolkit during login.
write_systemd_dropin() {
  local bundled=${1:-0}
  local dropin_dir="$AURA_DIR/hypr/systemd/user/hyprpolkitagent.service.d"
  mkdir -p "$dropin_dir"
  cat >"$dropin_dir/override.conf" <<EOF
[Service]
# Aura wrapper: hyprtoolkit agent when dist/ links, else Arch Qt agent.
# Rebuilds happen via scripts/build-hypr-polkit.sh --ensure (update.sh),
# never from ExecStartPre (90s timeout + Restart=on-failure = cc1plus loop).
ExecStart=
ExecStart=%h/.config/ags/scripts/hyprpolkitagent-start.sh
Environment=
TimeoutStartSec=30
RestartSec=2s
EOF
  if systemctl --user list-unit-files hyprpolkitagent.service &>/dev/null; then
    systemctl --user daemon-reload 2>/dev/null || true
  fi
}

restart_agent() {
  systemctl --user reset-failed hyprpolkitagent.service 2>/dev/null || true
  systemctl --user restart hyprpolkitagent.service
  systemctl --user --no-pager --full status hyprpolkitagent.service | head -20
}

case "$MODE" in
  check)
    if verify_agent; then
      ok "hyprpolkitagent ABI matches system Hypr stack"
      exit 0
    fi
    echo "error: hyprpolkitagent out of date or broken — run: $AURA_DIR/scripts/build-hypr-polkit.sh --ensure" >&2
    exit 1
    ;;
  ensure)
    if verify_agent; then
      ok "hyprpolkitagent already current"
      exit 0
    fi
    log "rebuilding hyprpolkitagent to match current Hypr ABI..."
    rebuild
    ;;
  restart)
    if ! verify_agent; then
      log "rebuilding hyprpolkitagent to match current Hypr ABI..."
      rebuild
    fi
    restart_agent
    ;;
  rebuild)
    rebuild
    ;;
esac
