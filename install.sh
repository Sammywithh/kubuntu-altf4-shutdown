#!/usr/bin/env bash
# ==============================================================================
# Installation Script: Kubuntu 26.04 Alt+F4 Smart Shutdown Dialog (Rust)
# ==============================================================================

set -euo pipefail

: "${HOME:?HOME ist nicht gesetzt}"
if [[ "${HOME}" != /* || "${HOME}" == "/" ]]; then
    echo "Fehler: Ungültiger HOME-Pfad: ${HOME}" >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
TARGET_BIN="${BIN_DIR}/kubuntu-shutdown-dialog"
KWIN_SCRIPT_DIR="${HOME}/.local/share/kwin/scripts/kubuntu-altf4-shutdown"
DBUS_SERVICE_DIR="${HOME}/.local/share/dbus-1/services"
DESKTOP_DIR="${HOME}/.local/share/applications"

render_template() {
    local src="$1"
    local dest="$2"
    local exec_path="$3"
    local tmp

    tmp="$(mktemp "${dest}.XXXXXX")"
    if (
        while IFS= read -r line || [[ -n "${line}" ]]; do
            rendered=""
            rest="${line}"
            while [[ "${rest}" == *"@EXEC_PATH@"* ]]; do
                rendered+="${rest%%@EXEC_PATH@*}${exec_path}"
                rest="${rest#*@EXEC_PATH@}"
            done
            rendered+="${rest}"
            printf '%s\n' "${rendered}" || exit 1
        done < "${src}" > "${tmp}"
    ) && chmod 0644 "${tmp}" && mv -f "${tmp}" "${dest}"; then
        return 0
    else
        rm -f "${tmp}"
        return 1
    fi
}

echo "=== Kubuntu 26.04 Alt+F4 Shutdown Dialog Installation ==="
echo ""

# 1. Ensure required directories exist
mkdir -p "${BIN_DIR}" "${KWIN_SCRIPT_DIR}/contents/code" "${DBUS_SERVICE_DIR}" "${DESKTOP_DIR}"

# 2. Build from source when Cargo.toml is present; fall back to prebuilt binary only in release archives
echo "[1/5] Installiere native Rust-Anwendung in ${BIN_DIR}..."
SOURCE_BIN=""
if [[ -f "${SCRIPT_DIR}/Cargo.toml" ]]; then
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Fehler: 'cargo' wurde nicht gefunden, obwohl Cargo.toml vorliegt." >&2
        echo "Bitte installiere Rust (https://rustup.rs) oder verwende ein Linux-Release-Archiv." >&2
        exit 1
    fi
    echo "Kompiliere Release-Binary aus dem Quellcode (cargo build --release --locked)..."
    cargo build --release --locked --manifest-path "${SCRIPT_DIR}/Cargo.toml"
    SOURCE_BIN="${SCRIPT_DIR}/target/release/kubuntu-shutdown-dialog"
elif [[ -x "${SCRIPT_DIR}/kubuntu-shutdown-dialog" ]]; then
    SOURCE_BIN="${SCRIPT_DIR}/kubuntu-shutdown-dialog"
else
    echo "Fehler: Weder Cargo.toml noch ausführbares kubuntu-shutdown-dialog Binary gefunden." >&2
    exit 1
fi

install -m 0755 "${SOURCE_BIN}" "${TARGET_BIN}"

# Remove legacy Python files if upgrading from an older version
rm -rf "${HOME}/.local/share/kubuntu-shutdown-dialog"

# 3. Install D-Bus Service File (mode 0644)
echo "[2/5] Registriere D-Bus Service..."
render_template \
    "${SCRIPT_DIR}/dbus/org.kubuntu.ShutdownDialog.service" \
    "${DBUS_SERVICE_DIR}/org.kubuntu.ShutdownDialog.service" \
    "${TARGET_BIN}"

# 4. Install Desktop Starter (mode 0644)
echo "[3/5] Erstelle Anwendungsstarter..."
render_template \
    "${SCRIPT_DIR}/desktop/kubuntu-shutdown-dialog.desktop" \
    "${DESKTOP_DIR}/kubuntu-shutdown-dialog.desktop" \
    "${TARGET_BIN}"

# 5. Install KWin 6 Script
echo "[4/5] Installiere KWin 6 Skript..."
install -m 0644 "${SCRIPT_DIR}/kwin-script/metadata.json" "${KWIN_SCRIPT_DIR}/metadata.json"
install -m 0644 "${SCRIPT_DIR}/kwin-script/contents/code/main.js" "${KWIN_SCRIPT_DIR}/contents/code/main.js"

# 6. Configure KDE Plasma 6 KWin & Shortcuts
echo "[5/5] Konfiguriere KWin und Alt+F4 Tastenkombination..."
if command -v kwriteconfig6 >/dev/null 2>&1; then
    kwriteconfig6 --file kwinrc --group Plugins --key "kubuntu-altf4-shutdownEnabled" "true"
    kwriteconfig6 --file kglobalshortcutsrc --group kwin --key "Window Close" "none,Alt+F4,Fenster schließen"
    kwriteconfig6 --file kglobalshortcutsrc --group kwin --key "AltF4DesktopShutdown" "Alt+F4,Alt+F4,Fenster schließen oder Herunterfahren-Menü anzeigen"

    if command -v qdbus6 >/dev/null 2>&1; then
        qdbus6 org.kde.KWin /KWin org.kde.KWin.reconfigure 2>/dev/null || true
    fi
elif command -v kwriteconfig5 >/dev/null 2>&1; then
    kwriteconfig5 --file kwinrc --group Plugins --key "kubuntu-altf4-shutdownEnabled" "true"
    kwriteconfig5 --file kglobalshortcutsrc --group kwin --key "Window Close" "none,Alt+F4,Fenster schließen"
    kwriteconfig5 --file kglobalshortcutsrc --group kwin --key "AltF4DesktopShutdown" "Alt+F4,Alt+F4,Fenster schließen oder Herunterfahren-Menü anzeigen"
fi

echo ""
echo "============================================================"
echo "  Installation erfolgreich abgeschlossen."
echo "============================================================"
echo ""
echo "Funktionsweise:"
echo " - Wenn ein Fenster aktiv ist: Alt + F4 schließt das Fenster."
echo " - Wenn der Desktop aktiv ist: Alt + F4 öffnet das Kubuntu Herunterfahren-Menü."
echo ""
echo "Testen des Dialogs:"
echo " - Direkt ausführen:  ~/.local/bin/kubuntu-shutdown-dialog"
echo " - Oder über das Testskript: ./test_dialog.sh"
echo ""
