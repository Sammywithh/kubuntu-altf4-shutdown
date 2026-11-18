#!/usr/bin/env bash
# ==============================================================================
# Uninstallation Script: Kubuntu 26.04 Alt+F4 Smart Shutdown Dialog
# ==============================================================================

set -e

BIN_DIR="${HOME}/.local/bin"
KWIN_SCRIPT_DIR="${HOME}/.local/share/kwin/scripts/kubuntu-altf4-shutdown"
DBUS_SERVICE_DIR="${HOME}/.local/share/dbus-1/services"
DESKTOP_DIR="${HOME}/.local/share/applications"

echo "=== Kubuntu Alt+F4 Shutdown Dialog Deinstallation ==="
echo ""

# 1. Restore original KWin shortcut
echo "[1/4] Stelle originale Alt+F4 Tastenkombination wieder her..."
if command -v kwriteconfig6 &> /dev/null; then
    kwriteconfig6 --file kglobalshortcutsrc --group kwin --key "Window Close" "Alt+F4,Alt+F4,Fenster schließen"
    kwriteconfig6 --file kglobalshortcutsrc --group kwin --key "AltF4DesktopShutdown" "none,Alt+F4,Fenster schließen oder Herunterfahren-Menü anzeigen"
    kwriteconfig6 --file kglobalshortcutsrc --group kwin --key "kubuntu-altf4-shutdown:AltF4DesktopShutdown" "none,none,"
    kwriteconfig6 --file kwinrc --group Plugins --key "kubuntu-altf4-shutdownEnabled" "false"

    if command -v qdbus6 &> /dev/null; then
        qdbus6 org.kde.KWin /KWin org.kde.KWin.reconfigure 2>/dev/null || true
    fi
elif command -v kwriteconfig5 &> /dev/null; then
    kwriteconfig5 --file kglobalshortcutsrc --group kwin --key "Window Close" "Alt+F4,Alt+F4,Fenster schließen"
    kwriteconfig5 --file kglobalshortcutsrc --group kwin --key "AltF4DesktopShutdown" "none,Alt+F4,Fenster schließen oder Herunterfahren-Menü anzeigen"
    kwriteconfig5 --file kwinrc --group Plugins --key "kubuntu-altf4-shutdownEnabled" "false"
fi

# 2. Remove KWin Script
echo "[2/4] Entferne KWin Skript..."
rm -rf "${KWIN_SCRIPT_DIR}"

# 3. Remove D-Bus Service & Desktop starter
echo "[3/4] Entferne D-Bus Service und Desktop-Starter..."
rm -f "${DBUS_SERVICE_DIR}/org.kubuntu.ShutdownDialog.service"
rm -f "${DESKTOP_DIR}/kubuntu-shutdown-dialog.desktop"

# 4. Remove Binary
echo "[4/4] Entferne Programmdatei..."
rm -f "${BIN_DIR}/kubuntu-shutdown-dialog"
rm -rf "${HOME}/.local/share/kubuntu-shutdown-dialog"

echo ""
echo "============================================================"
echo "  Deinstallation erfolgreich abgeschlossen!"
echo "  KDE Plasma Standardverhalten wurde wiederhergestellt."
echo "============================================================"
echo ""
