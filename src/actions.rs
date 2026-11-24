//! System actions for Kubuntu Shutdown Dialog.
//! Handles power and session management via `zbus` D-Bus calls (`org.freedesktop.login1`
//! and KDE Plasma session services) with fixed-path system binary fallbacks.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path};
use std::process::Command;

use zbus::{Connection, Proxy};

const LOGIN1_DEST: &str = "org.freedesktop.login1";
const LOGIN1_PATH: &str = "/org/freedesktop/login1";
const LOGIN1_MANAGER_IFACE: &str = "org.freedesktop.login1.Manager";
const LOGIN1_SESSION_IFACE: &str = "org.freedesktop.login1.Session";
const LOGIN1_SESSION_AUTO_PATH: &str = "/org/freedesktop/login1/session/auto";
const LOGIN1_SESSION_SELF_PATH: &str = "/org/freedesktop/login1/session/self";

const KDE_SHUTDOWN_DEST: &str = "org.kde.Shutdown";
const KDE_SHUTDOWN_PATH: &str = "/Shutdown";
const KDE_SHUTDOWN_IFACE: &str = "org.kde.Shutdown";

const SCREENSAVER_DEST: &str = "org.freedesktop.ScreenSaver";
const SCREENSAVER_PATH: &str = "/ScreenSaver";
const SCREENSAVER_IFACE: &str = "org.freedesktop.ScreenSaver";

const KSMSERVER_DEST: &str = "org.kde.ksmserver";
const KSMSERVER_PATH: &str = "/KSMServer";
const KSMSERVER_IFACE: &str = "org.kde.KSMServerInterface";

const KDE_LOGOUT_PROMPT_DEST: &str = "org.kde.LogoutPrompt";
const KDE_LOGOUT_PROMPT_PATH: &str = "/LogoutPrompt";
const KDE_LOGOUT_PROMPT_IFACE: &str = "org.kde.LogoutPrompt";

const TRUSTED_BIN_PREFIXES: [&str; 6] = [
    "/usr/bin",
    "/bin",
    "/usr/sbin",
    "/sbin",
    "/usr/lib",
    "/usr/libexec",
];

const SYSTEMCTL_CANDIDATES: [&str; 2] = ["/usr/bin/systemctl", "/bin/systemctl"];
const LOGINCTL_CANDIDATES: [&str; 2] = ["/usr/bin/loginctl", "/bin/loginctl"];
const POWEROFF_CANDIDATES: [&str; 2] = ["/usr/sbin/poweroff", "/sbin/poweroff"];
const REBOOT_CANDIDATES: [&str; 2] = ["/usr/sbin/reboot", "/sbin/reboot"];
const DM_TOOL_CANDIDATES: [&str; 2] = ["/usr/bin/dm-tool", "/bin/dm-tool"];
const KDE_GREETER_CANDIDATES: [&str; 3] = [
    "/usr/lib/x86_64-linux-gnu/libexec/ksmserver-logout-greeter",
    "/usr/lib/libexec/ksmserver-logout-greeter",
    "/usr/libexec/ksmserver-logout-greeter",
];

const MAX_SESSION_ID_LEN: usize = 64;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BusKind {
    System,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DbusArgs<'a> {
    Empty,
    Bool(bool),
    Str(&'a str),
}

trait ActionBackend {
    fn call_dbus(
        &self,
        bus: BusKind,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        args: DbusArgs<'_>,
    ) -> bool;

    fn run_fixed_cmd(&self, candidates: &[&str], args: &[&str]) -> bool;

    fn session_id(&self) -> Option<String>;
}

struct SystemActionBackend;

impl ActionBackend for SystemActionBackend {
    fn call_dbus(
        &self,
        bus: BusKind,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        args: DbusArgs<'_>,
    ) -> bool {
        call_dbus_method(bus, destination, path, interface, method, args)
    }

    fn run_fixed_cmd(&self, candidates: &[&str], args: &[&str]) -> bool {
        run_fixed_system_cmd(candidates, args)
    }

    fn session_id(&self) -> Option<String> {
        current_session_id()
    }
}

/// Finds the action index matching a hotkey character (case-insensitive).
pub fn index_for_hotkey(ch: char) -> Option<usize> {
    let upper = ch.to_ascii_uppercase();
    ACTIONS.iter().position(|action| action.hotkey == upper)
}

fn is_trusted_executable(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    if path
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return false;
    }
    if !TRUSTED_BIN_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        return false;
    }
    match fs::metadata(path) {
        Ok(meta) => meta.is_file() && (meta.permissions().mode() & 0o111 != 0),
        Err(_) => false,
    }
}

fn resolve_fixed_binary<'a>(candidates: &'a [&'a str]) -> Option<&'a Path> {
    candidates
        .iter()
        .map(Path::new)
        .find(|path| is_trusted_executable(path))
}

fn run_fixed_system_cmd(candidates: &[&str], args: &[&str]) -> bool {
    let Some(program) = resolve_fixed_binary(candidates) else {
        return false;
    };

    match Command::new(program).args(args).status() {
        Ok(status) => {
            if !status.success() {
                eprintln!(
                    "Error executing {} {}: exit status {}",
                    program.display(),
                    args.join(" "),
                    status
                );
            }
            status.success()
        }
        Err(err) => {
            eprintln!(
                "Error executing {} {}: {}",
                program.display(),
                args.join(" "),
                err
            );
            false
        }
    }
}

fn validated_session_id(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_SESSION_ID_LEN {
        return None;
    }
    if trimmed
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

fn current_session_id() -> Option<String> {
    let val = env::var("XDG_SESSION_ID").ok();
    validated_session_id(val.as_deref())
}

fn call_dbus_method(
    bus: BusKind,
    destination: &str,
    path: &str,
    interface: &str,
    method: &str,
    args: DbusArgs<'_>,
) -> bool {
    zbus::block_on(async {
        let conn_res = match bus {
            BusKind::System => Connection::system().await,
            BusKind::Session => Connection::session().await,
        };
        let Ok(conn) = conn_res else {
            return false;
        };
        let Ok(proxy) = Proxy::new(&conn, destination, path, interface).await else {
            return false;
        };
        let call_res = match args {
            DbusArgs::Empty => proxy.call::<_, _, ()>(method, &()).await,
            DbusArgs::Bool(flag) => proxy.call::<_, _, ()>(method, &(flag,)).await,
            DbusArgs::Str(value) => proxy.call::<_, _, ()>(method, &(value,)).await,
        };
        if let Err(err) = &call_res {
            eprintln!("Note: D-Bus call {destination} {path} {interface}.{method} failed: {err}");
        }
        call_res.is_ok()
    })
}

fn shutdown_with<B: ActionBackend>(backend: &B) -> bool {
    if backend.call_dbus(
        BusKind::System,
        LOGIN1_DEST,
        LOGIN1_PATH,
        LOGIN1_MANAGER_IFACE,
        "PowerOff",
        DbusArgs::Bool(true),
    ) {
        return true;
    }
    if backend.run_fixed_cmd(&SYSTEMCTL_CANDIDATES, &["poweroff"]) {
        return true;
    }
    backend.run_fixed_cmd(&POWEROFF_CANDIDATES, &[])
}

fn reboot_with<B: ActionBackend>(backend: &B) -> bool {
    if backend.call_dbus(
        BusKind::System,
        LOGIN1_DEST,
        LOGIN1_PATH,
        LOGIN1_MANAGER_IFACE,
        "Reboot",
        DbusArgs::Bool(true),
    ) {
        return true;
    }
    if backend.run_fixed_cmd(&SYSTEMCTL_CANDIDATES, &["reboot"]) {
        return true;
    }
    backend.run_fixed_cmd(&REBOOT_CANDIDATES, &[])
}

fn suspend_with<B: ActionBackend>(backend: &B) -> bool {
    if backend.call_dbus(
        BusKind::System,
        LOGIN1_DEST,
        LOGIN1_PATH,
        LOGIN1_MANAGER_IFACE,
        "Suspend",
        DbusArgs::Bool(true),
    ) {
        return true;
    }
    backend.run_fixed_cmd(&SYSTEMCTL_CANDIDATES, &["suspend"])
}

fn hibernate_with<B: ActionBackend>(backend: &B) -> bool {
    if backend.call_dbus(
        BusKind::System,
        LOGIN1_DEST,
        LOGIN1_PATH,
        LOGIN1_MANAGER_IFACE,
        "Hibernate",
        DbusArgs::Bool(true),
    ) {
        return true;
    }
    backend.run_fixed_cmd(&SYSTEMCTL_CANDIDATES, &["hibernate"])
}

fn logout_with<B: ActionBackend>(backend: &B) -> bool {
    if backend.call_dbus(
        BusKind::Session,
        KDE_SHUTDOWN_DEST,
        KDE_SHUTDOWN_PATH,
        KDE_SHUTDOWN_IFACE,
        "logout",
        DbusArgs::Empty,
    ) {
        return true;
    }

    let session_id = backend.session_id();
    if let Some(sid) = session_id.as_deref()
        && backend.call_dbus(
            BusKind::System,
            LOGIN1_DEST,
            LOGIN1_PATH,
            LOGIN1_MANAGER_IFACE,
            "TerminateSession",
            DbusArgs::Str(sid),
        )
    {
        return true;
    }

    if backend.call_dbus(
        BusKind::System,
        LOGIN1_DEST,
        LOGIN1_SESSION_AUTO_PATH,
        LOGIN1_SESSION_IFACE,
        "Terminate",
        DbusArgs::Empty,
    ) {
        return true;
    }

    if backend.call_dbus(
        BusKind::System,
        LOGIN1_DEST,
        LOGIN1_SESSION_SELF_PATH,
        LOGIN1_SESSION_IFACE,
        "Terminate",
        DbusArgs::Empty,
    ) {
        return true;
    }

    match session_id.as_deref() {
        Some(sid) => backend.run_fixed_cmd(&LOGINCTL_CANDIDATES, &["terminate-session", sid]),
        None => backend.run_fixed_cmd(&LOGINCTL_CANDIDATES, &["terminate-session", ""]),
    }
}

fn lock_with<B: ActionBackend>(backend: &B) -> bool {
    let session_id = backend.session_id();
    if let Some(sid) = session_id.as_deref()
        && backend.call_dbus(
            BusKind::System,
            LOGIN1_DEST,
            LOGIN1_PATH,
            LOGIN1_MANAGER_IFACE,
            "LockSession",
            DbusArgs::Str(sid),
        )
    {
        return true;
    }

    if backend.call_dbus(
        BusKind::System,
        LOGIN1_DEST,
        LOGIN1_SESSION_AUTO_PATH,
        LOGIN1_SESSION_IFACE,
        "Lock",
        DbusArgs::Empty,
    ) {
        return true;
    }

    if backend.call_dbus(
        BusKind::Session,
        SCREENSAVER_DEST,
        SCREENSAVER_PATH,
        SCREENSAVER_IFACE,
        "Lock",
        DbusArgs::Empty,
    ) {
        return true;
    }

    match session_id.as_deref() {
        Some(sid) => backend.run_fixed_cmd(&LOGINCTL_CANDIDATES, &["lock-session", sid]),
        None => backend.run_fixed_cmd(&LOGINCTL_CANDIDATES, &["lock-session"]),
    }
}

fn switch_user_with<B: ActionBackend>(backend: &B) -> bool {
    if backend.call_dbus(
        BusKind::Session,
        KSMSERVER_DEST,
        KSMSERVER_PATH,
        KSMSERVER_IFACE,
        "openSwitchUser",
        DbusArgs::Empty,
    ) {
        return true;
    }

    backend.run_fixed_cmd(&DM_TOOL_CANDIDATES, &["switch-to-greeter"])
}

fn kde_native_prompt_with<B: ActionBackend>(backend: &B) -> bool {
    if backend.call_dbus(
        BusKind::Session,
        KDE_LOGOUT_PROMPT_DEST,
        KDE_LOGOUT_PROMPT_PATH,
        KDE_LOGOUT_PROMPT_IFACE,
        "promptAll",
        DbusArgs::Empty,
    ) {
        return true;
    }

    backend.run_fixed_cmd(&KDE_GREETER_CANDIDATES, &[])
}

#[cfg(test)]
fn execute_action_with<B: ActionBackend>(backend: &B, id: ActionId) -> bool {
    match id {
        ActionId::Shutdown => shutdown_with(backend),
        ActionId::Reboot => reboot_with(backend),
        ActionId::Suspend => suspend_with(backend),
        ActionId::Hibernate => hibernate_with(backend),
        ActionId::Logout => logout_with(backend),
        ActionId::Lock => lock_with(backend),
        ActionId::SwitchUser => switch_user_with(backend),
    }
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
    shutdown_with(&SystemActionBackend)
}

/// Reboots the system.
pub fn action_reboot() -> bool {
    reboot_with(&SystemActionBackend)
}

/// Suspends the system (Sleep / Standby).
pub fn action_suspend() -> bool {
    suspend_with(&SystemActionBackend)
}

/// Puts the system into hibernation.
pub fn action_hibernate() -> bool {
    hibernate_with(&SystemActionBackend)
}

/// Logs out the current KDE Plasma user session.
pub fn action_logout() -> bool {
    logout_with(&SystemActionBackend)
}

/// Locks the screen.
pub fn action_lock() -> bool {
    lock_with(&SystemActionBackend)
}

/// Switches the current user.
pub fn action_switch_user() -> bool {
    switch_user_with(&SystemActionBackend)
}

/// Opens the official KDE Plasma 6 Logout greeter.
pub fn action_kde_native_prompt() -> bool {
    kde_native_prompt_with(&SystemActionBackend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RecordedCall {
        Dbus {
            bus: BusKind,
            destination: String,
            path: String,
            interface: String,
            method: String,
            arg: String,
        },
        Cmd {
            candidates: Vec<String>,
            args: Vec<String>,
        },
    }

    struct MockBackend {
        session_id: Option<String>,
        dbus_succeeds_on: Option<(&'static str, &'static str)>,
        cmd_succeeds: bool,
        calls: RefCell<Vec<RecordedCall>>,
    }

    impl MockBackend {
        fn failing(session_id: Option<&str>) -> Self {
            Self {
                session_id: session_id.map(str::to_owned),
                dbus_succeeds_on: None,
                cmd_succeeds: false,
                calls: RefCell::new(Vec::new()),
            }
        }

        fn succeeding_dbus(
            interface: &'static str,
            method: &'static str,
            session_id: Option<&str>,
        ) -> Self {
            Self {
                session_id: session_id.map(str::to_owned),
                dbus_succeeds_on: Some((interface, method)),
                cmd_succeeds: false,
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl ActionBackend for MockBackend {
        fn call_dbus(
            &self,
            bus: BusKind,
            destination: &str,
            path: &str,
            interface: &str,
            method: &str,
            args: DbusArgs<'_>,
        ) -> bool {
            let arg = match args {
                DbusArgs::Empty => String::new(),
                DbusArgs::Bool(v) => v.to_string(),
                DbusArgs::Str(s) => s.to_owned(),
            };
            self.calls.borrow_mut().push(RecordedCall::Dbus {
                bus,
                destination: destination.to_owned(),
                path: path.to_owned(),
                interface: interface.to_owned(),
                method: method.to_owned(),
                arg,
            });
            self.dbus_succeeds_on
                .is_some_and(|(iface, m)| iface == interface && m == method)
        }

        fn run_fixed_cmd(&self, candidates: &[&str], args: &[&str]) -> bool {
            self.calls.borrow_mut().push(RecordedCall::Cmd {
                candidates: candidates.iter().map(|s| (*s).to_owned()).collect(),
                args: args.iter().map(|s| (*s).to_owned()).collect(),
            });
            self.cmd_succeeds
        }

        fn session_id(&self) -> Option<String> {
            self.session_id.clone()
        }
    }

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
    fn trusted_executable_rejects_relative_untrusted_and_parent_dir_paths() {
        assert!(is_trusted_executable(Path::new("/bin/sh")));
        assert!(resolve_fixed_binary(&["/nonexistent/bin", "/bin/sh"]).is_some());

        // Relative names must be rejected (no PATH lookup).
        assert!(!is_trusted_executable(Path::new("sh")));
        assert!(!is_trusted_executable(Path::new("")));

        // Paths outside trusted system directories must be rejected.
        assert!(!is_trusted_executable(Path::new("/tmp/sh")));
        assert!(!is_trusted_executable(Path::new(
            "/home/user/bin/systemctl"
        )));

        // ParentDir (`..`) traversal must be rejected even if the resolved target exists.
        assert!(!is_trusted_executable(Path::new("/usr/bin/../../bin/sh")));
        assert!(!is_trusted_executable(Path::new("/bin/../bin/sh")));
        assert!(!is_trusted_executable(Path::new("../bin/sh")));
    }

    #[test]
    fn session_id_validation_accepts_safe_ids_and_rejects_invalid_input() {
        assert_eq!(validated_session_id(Some("2")), Some("2".to_owned()));
        assert_eq!(validated_session_id(Some(" c1 ")), Some("c1".to_owned()));
        assert_eq!(
            validated_session_id(Some("seat0_session-42")),
            Some("seat0_session-42".to_owned())
        );

        assert_eq!(validated_session_id(None), None);
        assert_eq!(validated_session_id(Some("")), None);
        assert_eq!(validated_session_id(Some("   ")), None);
        assert_eq!(validated_session_id(Some("../1")), None);
        assert_eq!(validated_session_id(Some("1; rm -rf /")), None);
        assert_eq!(validated_session_id(Some("session/2")), None);
        let too_long = "a".repeat(MAX_SESSION_ID_LEN + 1);
        assert_eq!(validated_session_id(Some(&too_long)), None);
    }

    #[test]
    fn power_actions_use_login1_system_dbus_before_fixed_cli_fallbacks() {
        let cases = [
            (ActionId::Shutdown, "PowerOff", "poweroff"),
            (ActionId::Reboot, "Reboot", "reboot"),
            (ActionId::Suspend, "Suspend", "suspend"),
            (ActionId::Hibernate, "Hibernate", "hibernate"),
        ];

        for (action_id, method, systemctl_arg) in cases {
            let ok_backend = MockBackend::succeeding_dbus(LOGIN1_MANAGER_IFACE, method, None);
            assert!(execute_action_with(&ok_backend, action_id));
            let calls = ok_backend.calls.borrow();
            assert_eq!(calls.len(), 1);
            assert_eq!(
                calls[0],
                RecordedCall::Dbus {
                    bus: BusKind::System,
                    destination: LOGIN1_DEST.to_owned(),
                    path: LOGIN1_PATH.to_owned(),
                    interface: LOGIN1_MANAGER_IFACE.to_owned(),
                    method: method.to_owned(),
                    arg: "true".to_owned(),
                }
            );
            drop(calls);

            let fallback_backend = MockBackend::failing(None);
            assert!(!execute_action_with(&fallback_backend, action_id));
            let fallback_calls = fallback_backend.calls.borrow();
            assert!(fallback_calls.len() >= 2);
            assert_eq!(
                fallback_calls[1],
                RecordedCall::Cmd {
                    candidates: SYSTEMCTL_CANDIDATES
                        .iter()
                        .map(|s| (*s).to_owned())
                        .collect(),
                    args: vec![systemctl_arg.to_owned()],
                }
            );
        }
    }

    #[test]
    fn logout_uses_session_scoped_termination_and_never_terminate_user() {
        let with_sid = MockBackend::failing(Some("42"));
        assert!(!execute_action_with(&with_sid, ActionId::Logout));
        let calls = with_sid.calls.borrow();

        assert_eq!(
            calls[0],
            RecordedCall::Dbus {
                bus: BusKind::Session,
                destination: KDE_SHUTDOWN_DEST.to_owned(),
                path: KDE_SHUTDOWN_PATH.to_owned(),
                interface: KDE_SHUTDOWN_IFACE.to_owned(),
                method: "logout".to_owned(),
                arg: String::new(),
            }
        );
        assert_eq!(
            calls[1],
            RecordedCall::Dbus {
                bus: BusKind::System,
                destination: LOGIN1_DEST.to_owned(),
                path: LOGIN1_PATH.to_owned(),
                interface: LOGIN1_MANAGER_IFACE.to_owned(),
                method: "TerminateSession".to_owned(),
                arg: "42".to_owned(),
            }
        );
        assert_eq!(
            calls.last().unwrap(),
            &RecordedCall::Cmd {
                candidates: LOGINCTL_CANDIDATES
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect(),
                args: vec!["terminate-session".to_owned(), "42".to_owned()],
            }
        );

        for call in calls.iter() {
            if let RecordedCall::Cmd { args, .. } = call {
                assert!(
                    !args.iter().any(|a| a == "terminate-user"),
                    "logout fallback must never invoke terminate-user"
                );
            }
        }
        drop(calls);

        let without_sid = MockBackend::failing(None);
        assert!(!execute_action_with(&without_sid, ActionId::Logout));
        let calls_no_sid = without_sid.calls.borrow();
        assert_eq!(
            calls_no_sid.last().unwrap(),
            &RecordedCall::Cmd {
                candidates: LOGINCTL_CANDIDATES
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect(),
                args: vec!["terminate-session".to_owned(), String::new()],
            }
        );
    }

    #[test]
    fn lock_switch_user_and_native_prompt_use_expected_dbus_interfaces() {
        let lock_backend =
            MockBackend::succeeding_dbus(LOGIN1_MANAGER_IFACE, "LockSession", Some("7"));
        assert!(execute_action_with(&lock_backend, ActionId::Lock));
        assert_eq!(lock_backend.calls.borrow().len(), 1);

        let lock_screensaver = MockBackend::succeeding_dbus(SCREENSAVER_IFACE, "Lock", None);
        assert!(execute_action_with(&lock_screensaver, ActionId::Lock));
        assert_eq!(lock_screensaver.calls.borrow().len(), 2);

        let switch_backend = MockBackend::succeeding_dbus(KSMSERVER_IFACE, "openSwitchUser", None);
        assert!(execute_action_with(&switch_backend, ActionId::SwitchUser));
        assert_eq!(switch_backend.calls.borrow().len(), 1);

        let prompt_backend =
            MockBackend::succeeding_dbus(KDE_LOGOUT_PROMPT_IFACE, "promptAll", None);
        assert!(kde_native_prompt_with(&prompt_backend));
        assert_eq!(prompt_backend.calls.borrow().len(), 1);
    }
}
