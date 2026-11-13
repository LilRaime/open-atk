#!/usr/bin/env bash
# Installation script for OpenATK & vxectl on Linux systems
# Usage: sudo ./packaging/install.sh

set -e

if [[ $EUID -ne 0 ]]; then
   echo "[!] This script must be run as root (use sudo)."
   exit 1
fi

echo "=== Installing OpenATK & vxectl for Linux ==="

# 1. Install binaries if built
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

TARGET_RELEASE="$ROOT_DIR/target/release"
INSTALL_BIN_DIR="/usr/local/bin"

if [[ -f "$TARGET_RELEASE/open-atk" ]]; then
    echo "[*] Installing open-atk to $INSTALL_BIN_DIR..."
    install -m 755 "$TARGET_RELEASE/open-atk" "$INSTALL_BIN_DIR/open-atk"
else
    echo "[i] open-atk binary not found in target/release. Run 'cargo build --release' first if you wish to install the binary."
fi

if [[ -f "$TARGET_RELEASE/vxectl" ]]; then
    echo "[*] Installing vxectl to $INSTALL_BIN_DIR..."
    install -m 755 "$TARGET_RELEASE/vxectl" "$INSTALL_BIN_DIR/vxectl"
else
    echo "[i] vxectl binary not found in target/release. Run 'cargo build --release' first if you wish to install the binary."
fi

# 2. Install udev rules
UDEV_DEST="/etc/udev/rules.d/71-open-atk.rules"
echo "[*] Installing udev rules to $UDEV_DEST..."
install -m 644 "$SCRIPT_DIR/71-open-atk.rules" "$UDEV_DEST"

echo "[*] Reloading and triggering udev rules..."
udevadm control --reload-rules || true
udevadm trigger || true

# 3. Install desktop application launcher & icon
ICON_SRC="$ROOT_DIR/crates/app/assets/app_icon.png"
ICON_DEST="/usr/share/icons/hicolor/256x256/apps/open-atk.png"
DESKTOP_SRC="$SCRIPT_DIR/open-atk.desktop"
DESKTOP_DEST="/usr/share/applications/open-atk.desktop"

if [[ -f "$ICON_SRC" ]]; then
    echo "[*] Installing application icon to $ICON_DEST..."
    mkdir -p "/usr/share/icons/hicolor/256x256/apps"
    install -m 644 "$ICON_SRC" "$ICON_DEST"
    gtk-update-icon-cache /usr/share/icons/hicolor || true
fi

if [[ -f "$DESKTOP_SRC" ]]; then
    echo "[*] Installing desktop entry to $DESKTOP_DEST..."
    install -m 644 "$DESKTOP_SRC" "$DESKTOP_DEST"
    update-desktop-database || true
fi

# 4. Install shell auto-completions
if [[ -x "$INSTALL_BIN_DIR/vxectl" ]]; then
    echo "[*] Installing shell completions for bash and zsh..."
    if [[ -d "/usr/share/bash-completion/completions" ]]; then
        "$INSTALL_BIN_DIR/vxectl" completions bash > "/usr/share/bash-completion/completions/vxectl" || true
    fi
    if [[ -d "/usr/share/zsh/site-functions" ]]; then
        "$INSTALL_BIN_DIR/vxectl" completions zsh > "/usr/share/zsh/site-functions/_vxectl" || true
    fi
fi

echo "=== Installation Completed Successfully! ==="
echo "You can now launch 'open-atk' from your desktop application launcher or run 'vxectl' in terminal."
