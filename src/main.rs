//! Kubuntu 26.04 Style Alt+F4 Smart Shutdown Dialog.
//! Native Rust implementation using `eframe`/`egui`, `resvg`, and `zbus` in KDE Breeze design.

mod actions;
mod dbus;
mod icons;
mod style;

use std::env;
use std::fs;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontData, FontDefinitions, FontFamily, FontId,
    Frame, Key, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, ViewportBuilder,
    ViewportCommand, X11WindowType,
};

use crate::actions::{ACTIONS, ActionId, ActionInfo, action_kde_native_prompt, execute_action};
use crate::dbus::{
    GuiDialogController, run_daemon_loop, spawn_gui_dbus_service, try_activate_existing_instance,
};
use crate::icons::{IconSet, build_window_icon};
use crate::style::ThemePalette;

const WINDOW_WIDTH: f32 = 652.0;
const WINDOW_HEIGHT: f32 = 420.0;
const OUTER_PAD_X: i8 = 28;
const OUTER_PAD_Y: i8 = 24;
const SECTION_GAP: f32 = 16.0;
const HEADER_HEIGHT: f32 = 44.0;
const TILE_WIDTH: f32 = 140.0;
const TILE_HEIGHT: f32 = 95.0;
const TILE_GAP: f32 = 12.0;
const DESCRIPTION_HEIGHT: f32 = 44.0;
const CANCEL_WIDTH: f32 = 126.0;
const CANCEL_HEIGHT: f32 = 34.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    Action(usize),
    Cancel,
}

impl FocusTarget {
    fn move_left(self) -> Self {
        match self {
            Self::Action(idx) => Self::Action(idx.saturating_sub(1)),
            Self::Cancel => Self::Action(ACTIONS.len() - 1),
        }
    }

    fn move_right(self) -> Self {
        match self {
            Self::Action(idx) if idx + 1 < ACTIONS.len() => Self::Action(idx + 1),
            Self::Action(_) => Self::Cancel,
            Self::Cancel => Self::Cancel,
        }
    }

    fn move_down(self) -> Self {
        match self {
            Self::Action(idx) => {
                let current = &ACTIONS[idx];
                if current.row == 0 {
                    let target = ACTIONS
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| a.row == 1)
                        .min_by_key(|(_, a)| a.col.abs_diff(current.col))
                        .map(|(i, _)| i);
                    target.map_or(Self::Cancel, Self::Action)
                } else {
                    Self::Cancel
                }
            }
            Self::Cancel => Self::Cancel,
        }
    }

    fn move_up(self) -> Self {
        match self {
            Self::Action(idx) => {
                let current = &ACTIONS[idx];
                if current.row > 0 {
                    let target = ACTIONS
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| a.row == current.row - 1)
                        .min_by_key(|(_, a)| a.col.abs_diff(current.col))
                        .map(|(i, _)| i);
                    target.map_or(Self::Action(idx), Self::Action)
                } else {
                    Self::Action(idx)
                }
            }
            Self::Cancel => Self::Action(ACTIONS.len() - 1),
        }
    }

    fn move_tab(self, reverse: bool) -> Self {
        let count = ACTIONS.len();
        if reverse {
            match self {
                Self::Action(0) => Self::Cancel,
                Self::Action(idx) => Self::Action(idx - 1),
                Self::Cancel => Self::Action(count - 1),
            }
        } else {
            match self {
                Self::Action(idx) if idx + 1 < count => Self::Action(idx + 1),
                Self::Action(_) => Self::Cancel,
                Self::Cancel => Self::Action(0),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CliCommand {
    ShowDialog { register_dbus: bool },
    NativePrompt,
    Daemon,
    Help,
    Version,
}

fn parse_cli_args<I, S>(args: I) -> Result<CliCommand, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut native = false;
    let mut daemon = false;
    let mut no_dbus = false;

    for arg in args.into_iter().skip(1) {
        match arg.as_ref() {
            "--native" => native = true,
            "--daemon" => daemon = true,
            "--no-dbus" => no_dbus = true,
            "-h" | "--help" => return Ok(CliCommand::Help),
            "-V" | "--version" => return Ok(CliCommand::Version),
            other => {
                return Err(format!(
                    "Unbekannte Option: {other}\nNutze --help für verfügbare Optionen."
                ));
            }
        }
    }

    if native {
        Ok(CliCommand::NativePrompt)
    } else if daemon {
        Ok(CliCommand::Daemon)
    } else {
        Ok(CliCommand::ShowDialog {
            register_dbus: !no_dbus,
        })
    }
}

fn print_help() {
    println!(
        concat!(
            "Kubuntu Alt+F4 Shutdown Menu ({})\n\n",
            "VERWENDUNG:\n",
            "  kubuntu-shutdown-dialog [OPTIONEN]\n\n",
            "OPTIONEN:\n",
            "      --native    Offizielles KDE Plasma 6 Abmeldemenü direkt öffnen\n",
            "      --daemon    Als D-Bus-Dienst im Hintergrund starten\n",
            "      --no-dbus   Dialog ohne D-Bus-Registrierung öffnen\n",
            "  -h, --help      Diese Hilfe anzeigen\n",
            "  -V, --version   Versionsnummer anzeigen"
        ),
        env!("CARGO_PKG_VERSION")
    );
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    let regular_candidates = [
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    let bold_candidates = [
        "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    ];

    if let Some(bytes) = regular_candidates.iter().find_map(|p| fs::read(p).ok()) {
        fonts
            .font_data
            .insert("system_regular".to_owned(), Arc::new(FontData::from_owned(bytes)));
        if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
            family.insert(0, "system_regular".to_owned());
        }
    }

    let mut bold_chain = Vec::new();
    if let Some(bytes) = bold_candidates.iter().find_map(|p| fs::read(p).ok()) {
        fonts
            .font_data
            .insert("system_bold".to_owned(), Arc::new(FontData::from_owned(bytes)));
        bold_chain.push("system_bold".to_owned());
    }
    if let Some(proportional) = fonts.families.get(&FontFamily::Proportional) {
        bold_chain.extend(proportional.iter().cloned());
    }
    fonts
        .families
        .insert(FontFamily::Name("Bold".into()), bold_chain);

    ctx.set_fonts(fonts);
}

struct KubuntuShutdownApp {
    palette: ThemePalette,
    icons: IconSet,
    focused_target: FocusTarget,
    hovered_action: Option<usize>,
    selected_action: Arc<Mutex<Option<ActionId>>>,
    focus_requested: Arc<AtomicBool>,
}

impl KubuntuShutdownApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        selected_action: Arc<Mutex<Option<ActionId>>>,
        focus_requested: Arc<AtomicBool>,
        gui_controller: Option<Arc<GuiDialogController>>,
    ) -> Self {
        configure_fonts(&cc.egui_ctx);

        let palette = ThemePalette::detect();
        let icons = IconSet::load(&cc.egui_ctx, &palette);

        let mut visuals = if palette.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = palette.bg_main;
        visuals.window_fill = palette.bg_main;
        cc.egui_ctx.set_visuals(visuals);

        if let Some(controller) = gui_controller {
            controller.attach_context(cc.egui_ctx.clone());
        }

        Self {
            palette,
            icons,
            focused_target: FocusTarget::Action(0),
            hovered_action: None,
            selected_action,
            focus_requested,
        }
    }

    fn active_description(&self) -> &'static str {
        if let Some(idx) = self.hovered_action {
            return ACTIONS[idx].description;
        }
        match self.focused_target {
            FocusTarget::Action(idx) => ACTIONS[idx].description,
            FocusTarget::Cancel => ACTIONS[0].description,
        }
    }

    fn trigger_action(&mut self, ctx: &egui::Context, action_id: ActionId) {
        if let Ok(mut guard) = self.selected_action.lock() {
            *guard = Some(action_id);
        }
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        let mut close_without_action = false;
        let mut chosen_action: Option<(usize, ActionId)> = None;

        ctx.input(|input| {
            if input.key_pressed(Key::Escape) {
                close_without_action = true;
                return;
            }

            if input.key_pressed(Key::Tab) {
                self.focused_target = self.focused_target.move_tab(input.modifiers.shift);
            } else if input.key_pressed(Key::ArrowLeft) {
                self.focused_target = self.focused_target.move_left();
            } else if input.key_pressed(Key::ArrowRight) {
                self.focused_target = self.focused_target.move_right();
            } else if input.key_pressed(Key::ArrowDown) {
                self.focused_target = self.focused_target.move_down();
            } else if input.key_pressed(Key::ArrowUp) {
                self.focused_target = self.focused_target.move_up();
            }

            if input.key_pressed(Key::Enter) || input.key_pressed(Key::Space) {
                match self.focused_target {
                    FocusTarget::Action(idx) => {
                        chosen_action = Some((idx, ACTIONS[idx].id));
                    }
                    FocusTarget::Cancel => {
                        close_without_action = true;
                    }
                }
                return;
            }

            if !input.modifiers.alt && !input.modifiers.ctrl && !input.modifiers.mac_cmd {
                for event in &input.events {
                    if let egui::Event::Text(text) = event {
                        for ch in text.chars() {
                            if let Some(idx) = actions::index_for_hotkey(ch) {
                                chosen_action = Some((idx, ACTIONS[idx].id));
                                return;
                            }
                        }
                    }
                }

                let key_hotkeys = [
                    (Key::H, 0),
                    (Key::N, 1),
                    (Key::E, 2),
                    (Key::R, 3),
                    (Key::A, 4),
                    (Key::S, 5),
                    (Key::B, 6),
                ];
                for (key, idx) in key_hotkeys {
                    if input.key_pressed(key) {
                        chosen_action = Some((idx, ACTIONS[idx].id));
                        return;
                    }
                }
            }
        });

        if close_without_action {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        if let Some((idx, action_id)) = chosen_action {
            self.focused_target = FocusTarget::Action(idx);
            self.trigger_action(ctx, action_id);
        }
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        let (header_rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), HEADER_HEIGHT),
            Sense::hover(),
        );
        let painter = ui.painter();

        let icon_pos = Pos2::new(header_rect.left(), header_rect.center().y - 20.0);
        let icon_rect = Rect::from_min_size(icon_pos, Vec2::splat(40.0));
        painter.image(
            self.icons.kde_logo.id(),
            icon_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let text_left = icon_rect.right() + 14.0;
        painter.text(
            Pos2::new(text_left, header_rect.top() + 2.0),
            Align2::LEFT_TOP,
            "Was möchten Sie tun?",
            FontId::new(17.0, FontFamily::Name("Bold".into())),
            self.palette.text_primary,
        );
        painter.text(
            Pos2::new(text_left, header_rect.top() + 24.0),
            Align2::LEFT_TOP,
            "Wählen Sie eine Aktion für Ihre Kubuntu-Sitzung:",
            FontId::new(13.0, FontFamily::Proportional),
            self.palette.text_secondary,
        );
    }

    fn draw_action_tile(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        action: &ActionInfo,
    ) -> Option<ActionId> {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(TILE_WIDTH, TILE_HEIGHT),
            Sense::click(),
        );
        let hovered = response.hovered();
        let pressed = response.is_pointer_button_down_on();
        let focused = self.focused_target == FocusTarget::Action(idx);

        if hovered {
            self.hovered_action = Some(idx);
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        let bg = if pressed {
            self.palette.bg_card_pressed
        } else if hovered || focused {
            self.palette.bg_card_hover
        } else {
            self.palette.bg_card
        };

        let border = if focused {
            Stroke::new(2.0, self.palette.border_hover)
        } else if hovered || pressed {
            Stroke::new(1.5, self.palette.border_hover)
        } else {
            Stroke::new(1.0, self.palette.border_color)
        };

        let painter = ui.painter();
        let radius = CornerRadius::same(10);
        painter.rect_filled(rect, radius, bg);
        painter.rect_stroke(rect, radius, border, StrokeKind::Inside);

        if let Some(texture) = self.icons.action_icon(action.icon) {
            let icon_rect = Rect::from_min_size(
                Pos2::new(rect.center().x - 18.0, rect.top() + 13.0),
                Vec2::splat(36.0),
            );
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        painter.text(
            Pos2::new(rect.center().x, rect.top() + 55.0),
            Align2::CENTER_TOP,
            action.title,
            FontId::new(12.5, FontFamily::Proportional),
            self.palette.text_primary,
        );
        painter.text(
            Pos2::new(rect.center().x, rect.top() + 73.0),
            Align2::CENTER_TOP,
            format!("({})", action.hotkey),
            FontId::new(12.0, FontFamily::Proportional),
            self.palette.text_primary,
        );

        if response.clicked() {
            self.focused_target = FocusTarget::Action(idx);
            return Some(action.id);
        }

        None
    }

    fn draw_description_box(&self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(width, DESCRIPTION_HEIGHT),
            Sense::hover(),
        );
        let painter = ui.painter();
        let radius = CornerRadius::same(8);

        painter.rect_filled(rect, radius, self.palette.bg_card);
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(1.0, self.palette.border_color),
            StrokeKind::Inside,
        );

        let wrap_width = (width - 28.0).max(100.0);
        let galley = painter.layout(
            self.active_description().to_owned(),
            FontId::new(12.5, FontFamily::Proportional),
            self.palette.text_primary,
            wrap_width,
        );
        let text_y = rect.center().y - galley.size().y * 0.5;
        painter.galley(
            Pos2::new(rect.left() + 14.0, text_y),
            galley,
            self.palette.text_primary,
        );
    }

    fn draw_footer(&mut self, ui: &mut egui::Ui) -> bool {
        let row_width = ui.available_width();
        let (row_rect, _) = ui.allocate_exact_size(
            Vec2::new(row_width, CANCEL_HEIGHT),
            Sense::hover(),
        );

        let btn_rect = Rect::from_min_size(
            Pos2::new(row_rect.right() - CANCEL_WIDTH, row_rect.top()),
            Vec2::new(CANCEL_WIDTH, CANCEL_HEIGHT),
        );
        let response = ui.allocate_rect(btn_rect, Sense::click());
        let hovered = response.hovered();
        let focused = self.focused_target == FocusTarget::Cancel;

        if hovered {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        let bg = if hovered || focused {
            self.palette.btn_cancel_hover
        } else {
            self.palette.btn_cancel_bg
        };
        let border = if focused {
            Stroke::new(2.0, self.palette.border_hover)
        } else if hovered {
            Stroke::new(1.0, self.palette.border_hover)
        } else {
            Stroke::new(1.0, self.palette.border_color)
        };

        let painter = ui.painter();
        let radius = CornerRadius::same(6);
        painter.rect_filled(btn_rect, radius, bg);
        painter.rect_stroke(btn_rect, radius, border, StrokeKind::Inside);

        let label_galley = painter.layout_no_wrap(
            "Abbrechen".to_owned(),
            FontId::new(13.0, FontFamily::Proportional),
            self.palette.text_primary,
        );
        let icon_size = 16.0;
        let icon_gap = 6.0;
        let total_content_width = icon_size + icon_gap + label_galley.size().x;
        let start_x = btn_rect.center().x - total_content_width * 0.5;

        let icon_rect = Rect::from_min_size(
            Pos2::new(start_x, btn_rect.center().y - icon_size * 0.5),
            Vec2::splat(icon_size),
        );
        painter.image(
            self.icons.cancel.id(),
            icon_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        painter.galley(
            Pos2::new(
                icon_rect.right() + icon_gap,
                btn_rect.center().y - label_galley.size().y * 0.5,
            ),
            label_galley,
            self.palette.text_primary,
        );

        response.clicked()
    }
}

impl eframe::App for KubuntuShutdownApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.palette.bg_main.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if self.focus_requested.swap(false, Ordering::SeqCst) {
            self.palette = ThemePalette::detect();
            self.icons = IconSet::load(&ctx, &self.palette);
            self.focused_target = FocusTarget::Action(0);
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }

        self.handle_keyboard_input(&ctx);
        self.hovered_action = None;

        let mut clicked_action: Option<ActionId> = None;
        let mut cancel_clicked = false;

        let panel_frame = Frame::NONE
            .fill(self.palette.bg_main)
            .inner_margin(Margin::symmetric(OUTER_PAD_X, OUTER_PAD_Y));

        egui::CentralPanel::default().frame(panel_frame).show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;

            self.draw_header(ui);
            ui.add_space(SECTION_GAP);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(TILE_GAP, 0.0);
                for (idx, action) in ACTIONS.iter().enumerate().filter(|(_, a)| a.row == 0) {
                    if let Some(id) = self.draw_action_tile(ui, idx, action) {
                        clicked_action = Some(id);
                    }
                }
            });

            ui.add_space(TILE_GAP);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(TILE_GAP, 0.0);
                for (idx, action) in ACTIONS.iter().enumerate().filter(|(_, a)| a.row == 1) {
                    if let Some(id) = self.draw_action_tile(ui, idx, action) {
                        clicked_action = Some(id);
                    }
                }
            });

            ui.add_space(SECTION_GAP);
            self.draw_description_box(ui);
            ui.add_space(SECTION_GAP);
            cancel_clicked = self.draw_footer(ui);
        });

        if let Some(action_id) = clicked_action {
            self.trigger_action(&ctx, action_id);
        } else if cancel_clicked {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

fn configure_display_backend() {
    // Prefer X11 / XWayland when DISPLAY is available so KWin centers the dialog window,
    // honors AlwaysOnTop / _NET_WM_WINDOW_TYPE_DIALOG, and draws native Breeze borders.
    if env::var_os("WINIT_UNIX_BACKEND").is_none()
        && env::var_os("DISPLAY").is_some_and(|v| !v.is_empty())
    {
        // Safety: called at startup on the main thread before spawning any worker threads.
        unsafe {
            env::set_var("WINIT_UNIX_BACKEND", "x11");
        }
    }
}

fn run_dialog(register_dbus: bool) -> Result<Option<ActionId>, eframe::Error> {
    configure_display_backend();

    if register_dbus && try_activate_existing_instance() {
        return Ok(None);
    }

    let focus_requested = Arc::new(AtomicBool::new(false));
    let selected_action = Arc::new(Mutex::new(None));

    let gui_controller = if register_dbus {
        let controller = Arc::new(GuiDialogController::new(Arc::clone(&focus_requested)));
        spawn_gui_dbus_service(Arc::clone(&controller));
        Some(controller)
    } else {
        None
    };

    let mut viewport = ViewportBuilder::default()
        .with_title("Kubuntu – Sitzung beenden")
        .with_app_id("kubuntu-shutdown-dialog")
        .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
        .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
        .with_max_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
        .with_resizable(false)
        .with_minimize_button(false)
        .with_maximize_button(false)
        .with_window_type(X11WindowType::Dialog)
        .with_always_on_top();

    if let Some(icon) = build_window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        run_and_return: true,
        ..Default::default()
    };

    let selected_for_app = Arc::clone(&selected_action);
    let focus_for_app = Arc::clone(&focus_requested);

    eframe::run_native(
        "kubuntu-shutdown-dialog",
        options,
        Box::new(move |cc| {
            Ok(Box::new(KubuntuShutdownApp::new(
                cc,
                selected_for_app,
                focus_for_app,
                gui_controller,
            )))
        }),
    )?;

    let chosen = selected_action.lock().ok().and_then(|guard| *guard);
    Ok(chosen)
}

fn main() -> ExitCode {
    let command = match parse_cli_args(env::args()) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(2);
        }
    };

    match command {
        CliCommand::Help => {
            print_help();
            ExitCode::SUCCESS
        }
        CliCommand::Version => {
            println!("kubuntu-shutdown-dialog {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        CliCommand::NativePrompt => {
            if action_kde_native_prompt() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        CliCommand::Daemon => match run_daemon_loop() {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("D-Bus daemon error: {err}");
                ExitCode::FAILURE
            }
        },
        CliCommand::ShowDialog { register_dbus } => match run_dialog(register_dbus) {
            Ok(Some(action_id)) => {
                let _ = execute_action(action_id);
                ExitCode::SUCCESS
            }
            Ok(None) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Dialog error: {err}");
                ExitCode::FAILURE
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_argument_parsing_matches_expected_modes() {
        assert_eq!(
            parse_cli_args(["prog"]).unwrap(),
            CliCommand::ShowDialog {
                register_dbus: true
            }
        );
        assert_eq!(
            parse_cli_args(["prog", "--no-dbus"]).unwrap(),
            CliCommand::ShowDialog {
                register_dbus: false
            }
        );
        assert_eq!(
            parse_cli_args(["prog", "--native"]).unwrap(),
            CliCommand::NativePrompt
        );
        assert_eq!(parse_cli_args(["prog", "--daemon"]).unwrap(), CliCommand::Daemon);
        assert_eq!(parse_cli_args(["prog", "--help"]).unwrap(), CliCommand::Help);
        assert_eq!(parse_cli_args(["prog", "--version"]).unwrap(), CliCommand::Version);
        assert!(parse_cli_args(["prog", "--unknown"]).is_err());
    }

    #[test]
    fn focus_navigation_covers_grid_and_cancel_button() {
        assert_eq!(FocusTarget::Action(0).move_right(), FocusTarget::Action(1));
        assert_eq!(FocusTarget::Action(6).move_right(), FocusTarget::Cancel);
        assert_eq!(FocusTarget::Cancel.move_left(), FocusTarget::Action(6));

        assert_eq!(FocusTarget::Action(0).move_down(), FocusTarget::Action(4));
        assert_eq!(FocusTarget::Action(1).move_down(), FocusTarget::Action(5));
        assert_eq!(FocusTarget::Action(2).move_down(), FocusTarget::Action(6));
        assert_eq!(FocusTarget::Action(3).move_down(), FocusTarget::Action(6));
        assert_eq!(FocusTarget::Action(4).move_down(), FocusTarget::Cancel);

        assert_eq!(FocusTarget::Cancel.move_up(), FocusTarget::Action(6));
        assert_eq!(FocusTarget::Action(6).move_up(), FocusTarget::Action(2));
        assert_eq!(FocusTarget::Action(5).move_up(), FocusTarget::Action(1));
        assert_eq!(FocusTarget::Action(4).move_up(), FocusTarget::Action(0));

        assert_eq!(FocusTarget::Action(6).move_tab(false), FocusTarget::Cancel);
        assert_eq!(FocusTarget::Cancel.move_tab(false), FocusTarget::Action(0));
        assert_eq!(FocusTarget::Action(0).move_tab(true), FocusTarget::Cancel);
    }
}
