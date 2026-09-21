#!/usr/bin/env bash
# ==============================================================================
# Installation Script: Kubuntu 26.04 Alt+F4 Smart Shutdown Dialog
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
KWIN_SCRIPT_DIR="${HOME}/.local/share/kwin/scripts/kubuntu-altf4-shutdown"
DBUS_SERVICE_DIR="${HOME}/.local/share/dbus-1/services"
DESKTOP_DIR="${HOME}/.local/share/applications"

echo "=== Kubuntu 26.04 Alt+F4 Shutdown Dialog Installation ==="
echo ""

# 1. Ensure required directories exist
mkdir -p "${BIN_DIR}" "${KWIN_SCRIPT_DIR}/contents/code" "${DBUS_SERVICE_DIR}" "${DESKTOP_DIR}"

# 2. Install Python Application Wrapper
echo "[1/5] Installiere Anwendungsdateien in ${BIN_DIR}..."
cat << 'EOF' > "${BIN_DIR}/kubuntu-shutdown-dialog"
#!/usr/bin/env bash
APP_DIR="$(dirname "$(readlink -f "$0")")"
PROJECT_DIR="${HOME}/KI-Agentic-Projects/kubuntu-altf4-shutdown/src"
if [ -d "${PROJECT_DIR}" ]; then
    exec /usr/bin/python3 "${PROJECT_DIR}/main.py" "$@"
else
    exec /usr/bin/python3 "${APP_DIR}/kubuntu-shutdown-dialog-src/main.py" "$@"
fi
EOF
chmod +x "${BIN_DIR}/kubuntu-shutdown-dialog"

# Copy source files to backup dir in ~/.local/share
mkdir -p "${HOME}/.local/share/kubuntu-shutdown-dialog"
cp -r "${SCRIPT_DIR}/src"/* "${HOME}/.local/share/kubuntu-shutdown-dialog/"

# 3. Install D-Bus Service File
echo "[2/5] Registriere D-Bus Service..."
sed "s|Exec=.*|Exec=${BIN_DIR}/kubuntu-shutdown-dialog|g" \
    "${SCRIPT_DIR}/dbus/org.kubuntu.ShutdownDialog.service" > "${DBUS_SERVICE_DIR}/org.kubuntu.ShutdownDialog.service"

# 4. Install Desktop Starter
echo "[3/5] Erstelle Anwendungsstarter..."
sed "s|Exec=.*|Exec=${BIN_DIR}/kubuntu-shutdown-dialog|g" \
    "${SCRIPT_DIR}/desktop/kubuntu-shutdown-dialog.desktop" > "${DESKTOP_DIR}/kubuntu-shutdown-dialog.desktop"
chmod +x "${DESKTOP_DIR}/kubuntu-shutdown-dialog.desktop"

# 5. Install KWin 6 Script
echo "[4/5] Installiere KWin 6 Skript..."
cp "${SCRIPT_DIR}/kwin-script/metadata.json" "${KWIN_SCRIPT_DIR}/"
cp "${SCRIPT_DIR}/kwin-script/contents/code/main.js" "${KWIN_SCRIPT_DIR}/contents/code/"

# 6. Configure KDE Plasma 6 KWin & Shortcuts
echo "[5/5] Konfiguriere KWin und Alt+F4 Tastenkombination..."
if command -v kwriteconfig6 &> /dev/null; then
    # Enable KWin script plugin
    kwriteconfig6 --file kwinrc --group Plugins --key "kubuntu-altf4-shutdownEnabled" "true"
    
    # Disable default Window Close shortcut conflict and route Alt+F4 to our script
    kwriteconfig6 --file kglobalshortcutsrc --group kwin --key "Window Close" "none,Alt+F4,Fenster schließen"
    kwriteconfig6 --file kglobalshortcutsrc --group kwin --key "AltF4DesktopShutdown" "Alt+F4,Alt+F4,Fenster schließen oder Herunterfahren-Menü anzeigen"
    
    # Reload KWin configuration
    if command -v qdbus6 &> /dev/null; then
        qdbus6 org.kde.KWin /KWin org.kde.KWin.reconfigure 2>/dev/null || true
    fi
elif command -v kwriteconfig5 &> /dev/null; then
    kwriteconfig5 --file kwinrc --group Plugins --key "kubuntu-altf4-shutdownEnabled" "true"
    kwriteconfig5 --file kglobalshortcutsrc --group kwin --key "Window Close" "none,Alt+F4,Fenster schließen"
    kwriteconfig5 --file kglobalshortcutsrc --group kwin --key "AltF4DesktopShutdown" "Alt+F4,Alt+F4,Fenster schließen oder Herunterfahren-Menü anzeigen"
fi

echo ""
echo "============================================================"
echo "  Installation erfolgreich abgeschlossen!"
echo "============================================================"
echo ""
echo "Funktionsweise:"
echo " • Wenn ein Fenster aktiv ist: Alt + F4 schließt das Fenster."
echo " • Wenn der Desktop aktiv ist: Alt + F4 öffnet das Kubuntu Herunterfahren-Menü."
echo ""
echo "Testen des Dialogs:"
echo " • Direkt ausführen:  ~/.local/bin/kubuntu-shutdown-dialog"
echo " • Oder über das Testskript: ./test_dialog.sh"
echo ""
