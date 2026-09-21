"""
System actions for Kubuntu Shutdown Dialog.
Handles power and session management via systemctl, loginctl, and KDE DBus APIs.
"""

import subprocess
import shutil
import os


def run_cmd(cmd_list):
    """Executes a system command safely."""
    try:
        subprocess.run(cmd_list, check=True)
        return True
    except Exception as e:
        print(f"Error executing {' '.join(cmd_list)}: {e}")
        return False


def action_shutdown():
    """Shuts down the system."""
    if shutil.which("systemctl"):
        return run_cmd(["systemctl", "poweroff"])
    return run_cmd(["poweroff"])


def action_reboot():
    """Reboots the system."""
    if shutil.which("systemctl"):
        return run_cmd(["systemctl", "reboot"])
    return run_cmd(["reboot"])


def action_suspend():
    """Suspends the system (Sleep / Standby)."""
    if shutil.which("systemctl"):
        return run_cmd(["systemctl", "suspend"])
    return False


def action_hibernate():
    """Puts the system to hibernation."""
    if shutil.which("systemctl"):
        return run_cmd(["systemctl", "hibernate"])
    return False


def action_logout():
    """Logs out the current KDE Plasma user."""
    # Try KDE Plasma 6 Shutdown DBus first
    if shutil.which("qdbus6"):
        if run_cmd(["qdbus6", "org.kde.Shutdown", "/Shutdown", "org.kde.Shutdown.logout"]):
            return True
    if shutil.which("qdbus"):
        if run_cmd(["qdbus", "org.kde.Shutdown", "/Shutdown", "org.kde.Shutdown.logout"]):
            return True
    # Fallback to loginctl
    user_id = os.getuid()
    return run_cmd(["loginctl", "terminate-user", str(user_id)])


def action_lock():
    """Locks the screen."""
    if shutil.which("loginctl"):
        return run_cmd(["loginctl", "lock-session"])
    if shutil.which("qdbus6"):
        return run_cmd(["qdbus6", "org.freedesktop.ScreenSaver", "/ScreenSaver", "Lock"])
    return False


def action_switch_user():
    """Switches the current user."""
    if shutil.which("qdbus6"):
        if run_cmd(["qdbus6", "org.kde.ksmserver", "/KSMServer", "org.kde.KSMServerInterface.openSwitchUser"]):
            return True
    if shutil.which("qdbus"):
        if run_cmd(["qdbus", "org.kde.ksmserver", "/KSMServer", "org.kde.KSMServerInterface.openSwitchUser"]):
            return True
    if shutil.which("dm-tool"):
        return run_cmd(["dm-tool", "switch-to-greeter"])
    return False


def action_kde_native_prompt():
    """Opens the official KDE Plasma 6 Logout greeter."""
    if shutil.which("qdbus6"):
        return run_cmd(["qdbus6", "org.kde.LogoutPrompt", "/LogoutPrompt", "org.kde.LogoutPrompt.promptAll"])
    if shutil.which("qdbus"):
        return run_cmd(["qdbus", "org.kde.LogoutPrompt", "/LogoutPrompt", "org.kde.LogoutPrompt.promptAll"])
    if os.path.exists("/usr/lib/x86_64-linux-gnu/libexec/ksmserver-logout-greeter"):
        return run_cmd(["/usr/lib/x86_64-linux-gnu/libexec/ksmserver-logout-greeter"])
    return False


ACTIONS = {
    "shutdown": {
        "id": "shutdown",
        "title": "Herunterfahren",
        "description": "Beendet alle geöffneten Anwendungen und schaltet den Computer vollständig aus.",
        "icon": "system-shutdown",
        "fallback_icon": "preferences-system-power-management",
        "hotkey": "H",
        "execute": action_shutdown,
        "destructive": True,
    },
    "reboot": {
        "id": "reboot",
        "title": "Neu starten",
        "description": "Beendet alle Anwendungen und startet den Computer neu.",
        "icon": "system-reboot",
        "fallback_icon": "view-refresh",
        "hotkey": "N",
        "execute": action_reboot,
        "destructive": True,
    },
    "suspend": {
        "id": "suspend",
        "title": "Energie sparen",
        "description": "Versetzt den Computer in den Standby-Modus. Programme und Dokumente bleiben im Arbeitsspeicher geöffnet.",
        "icon": "system-suspend",
        "fallback_icon": "weather-clear-night",
        "hotkey": "E",
        "execute": action_suspend,
        "destructive": False,
    },
    "hibernate": {
        "id": "hibernate",
        "title": "Ruhezustand",
        "description": "Speichert die aktuelle Arbeitssitzung auf der Festplatte und schaltet den Computer aus.",
        "icon": "system-suspend-hibernate",
        "fallback_icon": "media-floppy",
        "hotkey": "R",
        "execute": action_hibernate,
        "destructive": False,
    },
    "logout": {
        "id": "logout",
        "title": "Abmelden",
        "description": "Schließt alle Anwendungen und meldet das aktuelle Benutzerkonto ab.",
        "icon": "system-log-out",
        "fallback_icon": "go-home",
        "hotkey": "A",
        "execute": action_logout,
        "destructive": True,
    },
    "lock": {
        "id": "lock",
        "title": "Sperren",
        "description": "Sperrt den Bildschirm zum Schutz vor unbefugtem Zugriff.",
        "icon": "system-lock-screen",
        "fallback_icon": "object-locked",
        "hotkey": "S",
        "execute": action_lock,
        "destructive": False,
    },
    "switch_user": {
        "id": "switch_user",
        "title": "Benutzer wechseln",
        "description": "Ermöglicht es einem anderen Benutzer, sich anzumelden, während Ihre Sitzung weiterläuft.",
        "icon": "system-switch-user",
        "fallback_icon": "system-users",
        "hotkey": "B",
        "execute": action_switch_user,
        "destructive": False,
    },
}
