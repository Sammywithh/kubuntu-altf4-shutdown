//! Pure-Rust D-Bus session service for `org.kubuntu.ShutdownDialog`.
//! Serves both `org.kubuntu.ShutdownDialog` and `local.DBusService` at `/Dialog`
//! to preserve full compatibility with `kwin-script/contents/code/main.js`.

use std::env;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::egui;
use zbus::fdo::DBusProxy;
use zbus::{Connection, Proxy, connection, interface};

use crate::actions::action_kde_native_prompt;

pub const SERVICE_NAME: &str = "org.kubuntu.ShutdownDialog";
pub const OBJECT_PATH: &str = "/Dialog";
pub const IFACE_PRIMARY: &str = "org.kubuntu.ShutdownDialog";
pub const IFACE_LOCAL: &str = "local.DBusService";

/// Trait implemented by D-Bus execution modes (GUI focus wake-up or background daemon spawner).
pub trait DialogController: Send + Sync {
    fn show_dialog(&self);
    fn show_native_prompt(&self);
}

struct PrimaryInterface {
    controller: Arc<dyn DialogController>,
}

#[interface(name = "org.kubuntu.ShutdownDialog")]
impl PrimaryInterface {
    #[zbus(name = "showDialog")]
    fn show_dialog(&self) {
        self.controller.show_dialog();
    }

    #[zbus(name = "showNativePrompt")]
    fn show_native_prompt(&self) {
        self.controller.show_native_prompt();
    }
}

struct LocalInterface {
    controller: Arc<dyn DialogController>,
}

#[interface(name = "local.DBusService")]
impl LocalInterface {
    #[zbus(name = "showDialog")]
    fn show_dialog(&self) {
        self.controller.show_dialog();
    }

    #[zbus(name = "showNativePrompt")]
    fn show_native_prompt(&self) {
        self.controller.show_native_prompt();
    }
}

/// Controller used while a GUI dialog window is open.
pub struct GuiDialogController {
    focus_requested: Arc<AtomicBool>,
    egui_ctx: Mutex<Option<egui::Context>>,
}

impl GuiDialogController {
    pub fn new(focus_requested: Arc<AtomicBool>) -> Self {
        Self {
            focus_requested,
            egui_ctx: Mutex::new(None),
        }
    }

    /// Attaches the `egui::Context` once the window has initialized so incoming D-Bus calls
    /// can wake the UI loop immediately.
    pub fn attach_context(&self, ctx: egui::Context) {
        if let Ok(mut guard) = self.egui_ctx.lock() {
            *guard = Some(ctx);
        }
    }
}

impl DialogController for GuiDialogController {
    fn show_dialog(&self) {
        self.focus_requested.store(true, Ordering::SeqCst);
        if let Ok(guard) = self.egui_ctx.lock()
            && let Some(ctx) = guard.as_ref()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            ctx.request_repaint();
        }
    }

    fn show_native_prompt(&self) {
        let _ = action_kde_native_prompt();
    }
}

/// Controller used in `--daemon` mode to spawn the dialog on demand without keeping a window open.
pub struct DaemonController {
    active_child: Mutex<Option<Child>>,
}

impl DaemonController {
    pub fn new() -> Self {
        Self {
            active_child: Mutex::new(None),
        }
    }

    fn has_active_child(guard: &mut Option<Child>) -> bool {
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(None) => return true,
                Ok(Some(_)) | Err(_) => {
                    *guard = None;
                }
            }
        }
        false
    }
}

impl DialogController for DaemonController {
    fn show_dialog(&self) {
        let Ok(mut guard) = self.active_child.lock() else {
            return;
        };
        if Self::has_active_child(&mut guard) {
            return;
        }

        let Ok(exe) = env::current_exe() else {
            eprintln!("Note: Unable to resolve executable path for daemon spawn.");
            return;
        };

        match Command::new(exe).arg("--no-dbus").spawn() {
            Ok(child) => {
                *guard = Some(child);
            }
            Err(err) => {
                eprintln!("Note: Failed to spawn dialog process: {err}");
            }
        }
    }

    fn show_native_prompt(&self) {
        let _ = action_kde_native_prompt();
    }
}

async fn build_service_connection(controller: Arc<dyn DialogController>) -> zbus::Result<Connection> {
    let primary = PrimaryInterface {
        controller: Arc::clone(&controller),
    };
    let local = LocalInterface { controller };

    connection::Builder::session()
        .name(SERVICE_NAME)
        .serve_at(OBJECT_PATH, primary)
        .serve_at(OBJECT_PATH, local)
        .build()
        .await
}

/// Checks if another `org.kubuntu.ShutdownDialog` instance already owns the session bus name.
/// If so, invokes `showDialog()` on it and returns `true` so the caller can exit cleanly.
pub fn try_activate_existing_instance() -> bool {
    zbus::block_on(async {
        let Ok(conn) = Connection::session().await else {
            return false;
        };
        let Ok(dbus) = DBusProxy::new(&conn).await else {
            return false;
        };
        let Ok(bus_name) = SERVICE_NAME.try_into() else {
            return false;
        };
        let Ok(true) = dbus.name_has_owner(bus_name).await else {
            return false;
        };

        if let Ok(proxy) = Proxy::new(&conn, SERVICE_NAME, OBJECT_PATH, IFACE_PRIMARY).await {
            let _ = proxy.call::<_, _, ()>("showDialog", &()).await;
            return true;
        }

        false
    })
}

/// Spawns a background thread hosting the D-Bus service while the GUI dialog is open.
pub fn spawn_gui_dbus_service(controller: Arc<GuiDialogController>) {
    let controller_dyn: Arc<dyn DialogController> = controller;
    let _ = thread::Builder::new()
        .name("dbus-service".to_string())
        .spawn(move || {
            zbus::block_on(async move {
                match build_service_connection(controller_dyn).await {
                    Ok(_conn) => {
                        std::future::pending::<()>().await;
                    }
                    Err(err) => {
                        eprintln!("Note: D-Bus service registration: {err}");
                    }
                }
            });
        });
}

/// Runs the headless `--daemon` D-Bus service loop on the current thread.
pub fn run_daemon_loop() -> Result<(), Box<dyn std::error::Error>> {
    let controller: Arc<dyn DialogController> = Arc::new(DaemonController::new());
    zbus::block_on(async move {
        let _conn = build_service_connection(controller).await?;
        std::future::pending::<()>().await;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_controller_sets_focus_flag_on_show_dialog() {
        let focus_flag = Arc::new(AtomicBool::new(false));
        let controller = GuiDialogController::new(Arc::clone(&focus_flag));
        assert!(!focus_flag.load(Ordering::SeqCst));

        controller.show_dialog();
        assert!(focus_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn daemon_controller_detects_exited_child() {
        let mut child_slot = Some(
            Command::new("true")
                .spawn()
                .expect("`true` should spawn in test"),
        );
        // Wait for `true` to exit so has_active_child clears the slot.
        if let Some(child) = child_slot.as_mut() {
            let _ = child.wait();
        }
        assert!(!DaemonController::has_active_child(&mut child_slot));
        assert!(child_slot.is_none());
    }
}
