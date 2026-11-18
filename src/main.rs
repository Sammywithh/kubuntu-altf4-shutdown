mod actions;
mod dbus;
mod icons;
mod style;

fn main() {
    let _palette = style::ThemePalette::detect();
    let _count = actions::ACTIONS.len();
    let _icon = icons::build_window_icon();
    if false {
        let _ = dbus::run_daemon_loop();
    }
}
