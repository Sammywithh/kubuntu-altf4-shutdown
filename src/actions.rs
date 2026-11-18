//! System actions for Kubuntu Shutdown Dialog.
//! Handles power and session management via `systemctl`, `loginctl`, and KDE D-Bus APIs.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

/// Identifiers for the supported shutdown and session actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionId {
    Shutdown,
    Reboot,
    Suspend,
    Hibernate,
    Logout,
    Lock,
    SwitchUser,
}

/// Metadata and layout positioning for a single dialog action tile.
#[derive(Debug, Clone, Copy)]
pub struct ActionInfo {
    pub id: ActionId,
    #[cfg_attr(not(test), expect(dead_code))]
    pub key: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub fallback_icon: &'static str,
    pub hotkey: char,
    #[cfg_attr(not(test), expect(dead_code))]
    pub destructive: bool,
    pub row: usize,
    pub col: usize,
}

/// All available actions in grid order (row 1: 4 tiles, row 2: 3 tiles).
pub const ACTIONS: [ActionInfo; 7] = [
    ActionInfo {
        id: ActionId::Shutdown,
        key: "shutdown",
        title: "Herunterfahren",
        description: "Beendet alle geöffneten Anwendungen und schaltet den Computer vollständig aus.",
        icon: "system-shutdown",
        fallback_icon: "preferences-system-power-management",
        hotkey: 'H',
        destructive: true,
        row: 0,
        col: 0,
    },
    ActionInfo {
        id: ActionId::Reboot,
        key: "reboot",
        title: "Neu starten",
        description: "Beendet alle Anwendungen und startet den Computer neu.",
        icon: "system-reboot",
        fallback_icon: "view-refresh",
        hotkey: 'N',
        destructive: true,
        row: 0,
        col: 1,
    },
    ActionInfo {
        id: ActionId::Suspend,
        key: "suspend",
        title: "Energie sparen",
        description: "Versetzt den Computer in den Standby-Modus. Programme und Dokumente bleiben im Arbeitsspeicher geöffnet.",
        icon: "system-suspend",
        fallback_icon: "weather-clear-night",
        hotkey: 'E',
        destructive: false,
        row: 0,
        col: 2,
    },
    ActionInfo {
        id: ActionId::Hibernate,
        key: "hibernate",
        title: "Ruhezustand",
        description: "Speichert die aktuelle Arbeitssitzung auf der Festplatte und schaltet den Computer aus.",
        icon: "system-suspend-hibernate",
        fallback_icon: "media-floppy",
        hotkey: 'R',
        destructive: false,
        row: 0,
        col: 3,
    },
    ActionInfo {
        id: ActionId::Logout,
        key: "logout",
        title: "Abmelden",
        description: "Schließt alle Anwendungen und meldet das aktuelle Benutzerkonto ab.",
        icon: "system-log-out",
        fallback_icon: "go-home",
        hotkey: 'A',
        destructive: true,
        row: 1,
        col: 0,
    },
    ActionInfo {
        id: ActionId::Lock,
        key: "lock",
        title: "Sperren",
        description: "Sperrt den Bildschirm zum Schutz vor unbefugtem Zugriff.",
        icon: "system-lock-screen",
        fallback_icon: "object-locked",
        hotkey: 'S',
        destructive: false,
        row: 1,
        col: 1,
    },
    ActionInfo {
        id: ActionId::SwitchUser,
        key: "switch_user",
        title: "Benutzer wechseln",
        description: "Ermöglicht es einem anderen Benutzer, sich anzumelden, während Ihre Sitzung weiterläuft.",
        icon: "system-switch-user",
        fallback_icon: "system-users",
        hotkey: 'B',
        destructive: false,
        row: 1,
        col: 2,
    },
];

/// Finds the action index matching a hotkey character (case-insensitive).
pub fn index_for_hotkey(ch: char) -> Option<usize> {
    let upper = ch.to_ascii_uppercase();
    ACTIONS.iter().position(|action| action.hotkey == upper)
}

/// Checks whether a command exists and is executable on `PATH` (or as an explicit path).
pub fn command_exists(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }

    let path = Path::new(cmd);
    if path.components().count() > 1 {
        return is_executable_file(path);
    }

    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path_var).any(|dir| is_executable_file(&dir.join(cmd)))
}

fn is_executable_file(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(meta) => meta.is_file() && (meta.permissions().mode() & 0o111 != 0),
        Err(_) => false,
    }
}

/// Executes a system command safely and returns whether it exited successfully.
pub fn run_cmd(program: &str, args: &[&str]) -> bool {
    match Command::new(program).args(args).status() {
        Ok(status) => {
            if !status.success() {
                eprintln!(
                    "Error executing {} {}: exit status {}",
                    program,
                    args.join(" "),
                    status
                );
            }
            status.success()
        }
        Err(err) => {
            eprintln!("Error executing {} {}: {}", program, args.join(" "), err);
            false
        }
    }
}

fn current_uid() -> Option<String> {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("Uid:")
                && let Some(uid) = rest.split_whitespace().next()
                && uid.chars().all(|c| c.is_ascii_digit())
            {
                return Some(uid.to_string());
            }
        }
    }

    let output = Command::new("id").arg("-u").output().ok()?;
    if output.status.success() {
        let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !uid.is_empty() && uid.chars().all(|c| c.is_ascii_digit()) {
            return Some(uid);
        }
    }

    None
}

/// Executes the specified power or session action.
pub fn execute_action(id: ActionId) -> bool {
    match id {
        ActionId::Shutdown => action_shutdown(),
        ActionId::Reboot => action_reboot(),
        ActionId::Suspend => action_suspend(),
        ActionId::Hibernate => action_hibernate(),
        ActionId::Logout => action_logout(),
        ActionId::Lock => action_lock(),
        ActionId::SwitchUser => action_switch_user(),
    }
}

/// Shuts down the system.
pub fn action_shutdown() -> bool {
    if command_exists("systemctl") {
        return run_cmd("systemctl", &["poweroff"]);
    }
    run_cmd("poweroff", &[])
}

/// Reboots the system.
pub fn action_reboot() -> bool {
    if command_exists("systemctl") {
        return run_cmd("systemctl", &["reboot"]);
    }
    run_cmd("reboot", &[])
}

/// Suspends the system (Sleep / Standby).
pub fn action_suspend() -> bool {
    if command_exists("systemctl") {
        return run_cmd("systemctl", &["suspend"]);
    }
    false
}

/// Puts the system into hibernation.
pub fn action_hibernate() -> bool {
    if command_exists("systemctl") {
        return run_cmd("systemctl", &["hibernate"]);
    }
    false
}

/// Logs out the current KDE Plasma user.
pub fn action_logout() -> bool {
    if command_exists("qdbus6")
        && run_cmd(
            "qdbus6",
            &["org.kde.Shutdown", "/Shutdown", "org.kde.Shutdown.logout"],
        )
    {
        return true;
    }

    if command_exists("qdbus")
        && run_cmd(
            "qdbus",
            &["org.kde.Shutdown", "/Shutdown", "org.kde.Shutdown.logout"],
        )
    {
        return true;
    }

    if let Some(uid) = current_uid() {
        return run_cmd("loginctl", &["terminate-user", &uid]);
    }
    false
}

/// Locks the screen.
pub fn action_lock() -> bool {
    if command_exists("loginctl") {
        return run_cmd("loginctl", &["lock-session"]);
    }
    if command_exists("qdbus6") {
        return run_cmd(
            "qdbus6",
            &["org.freedesktop.ScreenSaver", "/ScreenSaver", "Lock"],
        );
    }
    false
}

/// Switches the current user.
pub fn action_switch_user() -> bool {
    if command_exists("qdbus6")
        && run_cmd(
            "qdbus6",
            &[
                "org.kde.ksmserver",
                "/KSMServer",
                "org.kde.KSMServerInterface.openSwitchUser",
            ],
        )
    {
        return true;
    }

    if command_exists("qdbus")
        && run_cmd(
            "qdbus",
            &[
                "org.kde.ksmserver",
                "/KSMServer",
                "org.kde.KSMServerInterface.openSwitchUser",
            ],
        )
    {
        return true;
    }

    if command_exists("dm-tool") {
        return run_cmd("dm-tool", &["switch-to-greeter"]);
    }
    false
}

/// Opens the official KDE Plasma 6 Logout greeter.
pub fn action_kde_native_prompt() -> bool {
    if command_exists("qdbus6") {
        return run_cmd(
            "qdbus6",
            &[
                "org.kde.LogoutPrompt",
                "/LogoutPrompt",
                "org.kde.LogoutPrompt.promptAll",
            ],
        );
    }

    if command_exists("qdbus") {
        return run_cmd(
            "qdbus",
            &[
                "org.kde.LogoutPrompt",
                "/LogoutPrompt",
                "org.kde.LogoutPrompt.promptAll",
            ],
        );
    }

    let greeter = "/usr/lib/x86_64-linux-gnu/libexec/ksmserver-logout-greeter";
    if Path::new(greeter).exists() {
        return run_cmd(greeter, &[]);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_layout_and_hotkeys_match_spec() {
        let expected = [
            (ActionId::Shutdown, "shutdown", 'H', 0, 0, true),
            (ActionId::Reboot, "reboot", 'N', 0, 1, true),
            (ActionId::Suspend, "suspend", 'E', 0, 2, false),
            (ActionId::Hibernate, "hibernate", 'R', 0, 3, false),
            (ActionId::Logout, "logout", 'A', 1, 0, true),
            (ActionId::Lock, "lock", 'S', 1, 1, false),
            (ActionId::SwitchUser, "switch_user", 'B', 1, 2, false),
        ];

        assert_eq!(ACTIONS.len(), expected.len());
        for (idx, (id, key, hotkey, row, col, destructive)) in expected.into_iter().enumerate() {
            let info = &ACTIONS[idx];
            assert_eq!(info.id, id);
            assert_eq!(info.key, key);
            assert_eq!(info.hotkey, hotkey);
            assert_eq!(info.row, row);
            assert_eq!(info.col, col);
            assert_eq!(info.destructive, destructive);
            assert!(!info.title.is_empty());
            assert!(!info.description.is_empty());
            assert_eq!(index_for_hotkey(hotkey), Some(idx));
            assert_eq!(index_for_hotkey(hotkey.to_ascii_lowercase()), Some(idx));
        }

        assert_eq!(index_for_hotkey('X'), None);
    }

    #[test]
    fn command_exists_detects_standard_tools() {
        assert!(command_exists("sh"));
        assert!(!command_exists("definitely_nonexistent_command_xyz_12345"));
        assert!(!command_exists(""));
    }

    #[test]
    fn current_uid_returns_numeric_uid() {
        let uid = current_uid().expect("current_uid should return a UID on Linux");
        assert!(!uid.is_empty());
        assert!(uid.chars().all(|c| c.is_ascii_digit()));
    }
}
