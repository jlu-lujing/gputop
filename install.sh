#!/usr/bin/env bash
# gputop installer — installs the latest release binary for Linux x86_64
set -euo pipefail

BINARY="gputop"
REPO="jlu-lujing/gputop"
INSTALL_DIR="${GPUTOP_INSTALL_DIR:-/usr/local/bin}"

# ── Color helpers ──────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

die() { printf "${RED}error: %s${NC}\n" "$1" >&2; exit 1; }
info() { printf "${CYAN}→ %s${NC}\n" "$1"; }
ok()   { printf "${GREEN}✓ %s${NC}\n" "$1"; }
warn() { printf "${YELLOW}⚠ %s${NC}\n" "$1"; }

# ── Checks ─────────────────────────────────────────────────────
uname_os="$(uname -s)"
uname_arch="$(uname -m)"

[[ "$uname_os" == "Linux" ]] || die "Only Linux is supported at this time (detected: $uname_os)"
[[ "$uname_arch" == "x86_64" ]] || die "Only x86_64 is supported at this time (detected: $uname_arch)"

if ! command -v nvidia-smi &>/dev/null; then
    warn "nvidia-smi not found — make sure you have an NVIDIA GPU and driver installed"
fi

# ── Download ───────────────────────────────────────────────────
TAG="v0.2.3"
URL="https://github.com/${REPO}/releases/download/${TAG}/${BINARY}-x86_64-unknown-linux-gnu.tar.gz"

info "Downloading ${BINARY} from GitHub Releases..."

TMPDIR_INSTALL="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_INSTALL"' EXIT

if ! curl -fsSL "$URL" -o "${TMPDIR_INSTALL}/${BINARY}.tar.gz"; then
    die "Failed to download from $URL\n\nCheck that the release exists at:\nhttps://github.com/${REPO}/releases"
fi

# ── Extract ────────────────────────────────────────────────────
info "Extracting..."
tar -xzf "${TMPDIR_INSTALL}/${BINARY}.tar.gz" -C "$TMPDIR_INSTALL"

# Find the binary
NEW_BINARY=""
for f in "$TMPDIR_INSTALL"/${BINARY} \
         "$TMPDIR_INSTALL"/gputop-*/target/release/${BINARY} \
         "$TMPDIR_INSTALL"/${BINARY}-x86_64-unknown-linux-gnu/${BINARY}; do
    if [[ -f "$f" ]]; then
        NEW_BINARY="$f"
        break
    fi
done

[[ -n "$NEW_BINARY" ]] || die "Could not find ${BINARY} binary in the downloaded archive"

# ── Install ────────────────────────────────────────────────────
if [[ "$EUID" -ne 0 ]]; then
    info "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo_install=true
else
    sudo_install=false
fi

if $sudo_install; then
    sudo install -Dm755 "$NEW_BINARY" "${INSTALL_DIR}/${BINARY}"
else
    install -Dm755 "$NEW_BINARY" "${INSTALL_DIR}/${BINARY}"
fi

ok "${BINARY} installed to ${INSTALL_DIR}/${BINARY}"

# ── Verify ─────────────────────────────────────────────────────
if "${INSTALL_DIR}/${BINARY}" --version 2>/dev/null; then
    ok "Installation verified"
else
    info "Installation complete. Run '${BINARY}' to start."
fi
