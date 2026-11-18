//! KDE Breeze SVG icon loader and rasterizer using `resvg`.
//! Embeds authentic Breeze SVGs directly into the binary with optional runtime
//! lookup from `/usr/share/icons/breeze*`, dynamically recolored for Dark/Light mode.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use eframe::egui::{self, ColorImage, IconData, TextureHandle, TextureOptions};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;

use crate::actions::ACTIONS;
use crate::style::ThemePalette;

const SVG_KDE_LOGO: &str = r##"<svg height="48" width="48" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><linearGradient id="a"><stop offset="0"/><stop offset="1" stop-opacity="0"/></linearGradient><linearGradient id="b" gradientUnits="userSpaceOnUse" x1="16" x2="34.942" xlink:href="#a" y1="11" y2="29.942"/><linearGradient id="c" gradientTransform="matrix(.70710678 .70710678 -.70710678 .70710678 0 0)" gradientUnits="userSpaceOnUse" x1="31.021" x2="43.104" xlink:href="#a" y1="-9.896" y2="-9.896"/><linearGradient id="d" gradientUnits="userSpaceOnUse" x1="23.716" x2="26.348" xlink:href="#a" y1="13.219" y2="15.851"/><linearGradient id="e" gradientTransform="matrix(1.0454421 0 0 1.0298842 46.076843 3.856842)" gradientUnits="userSpaceOnUse" x1="-45.221867" x2=".691726" y1="-4.91011" y2="41.697075"><stop offset="0" stop-color="#2bc0ff"/><stop offset="1" stop-color="#1d99f3"/></linearGradient><path d="m5.9999998-.00000914h36.0000012c3.600001 0 6.000001 2.40000654 6.000001 6.00000714v36c0 3.600002-2.4 6.000008-6.000001 6.000008h-36.0000012c-3.6000002 0-6.00000022-2.400006-6.00000022-6.000008v-36c0-3.6000006 2.40000002-6.00000714 6.00000022-6.00000714z" fill="url(#e)" transform="matrix(.83333329 0 0 .83333329 4 4.000002)"/><g transform="translate(-38 -2)"><circle cx="24" cy="24" opacity=".539" r="7"/><circle cx="24" cy="24" opacity=".539" r="9"/><path d="m22.5 11c-.277 0-.5.223-.5.5l-1 4.5v8 8l1 4.5c0 .277.223.5.5.5h3c.277 0 .43991-.229596.5-.5l1-4.5v-8-8l-1-4.5c-.06009-.270404-.223-.5-.5-.5z" opacity=".539"/><path d="m13.746952 15.868272c-.195869.195869-.195869.511238 0 .707107l2.474873 3.889087 5.656855 5.656854 5.656854 5.656855 3.889087 2.474873c.195869.195869.511238.195869.707107 0l2.12132-2.12132c.195869-.195869.148715-.473412 0-.707107l-2.474873-3.889087-5.656855-5.656854-5.656854-5.656855-3.889087-2.474873c-.233695-.148715-.511238-.195869-.707107 0z" opacity=".539"/><path d="m34.235965 15.868272c.195869.195869.195869.511238 0 .707107l-2.474873 3.889087-5.656855 5.656854-5.656854 5.656855-3.889087 2.474873c-.195869.195869-.511238.195869-.707107 0l-2.12132-2.12132c-.195869-.195869-.148715-.473412 0-.707107l2.474873-3.889087 5.656855-5.656854 5.656854-5.656855 3.889087-2.474873c.233695-.148715.511238-.195869.707107 0z" opacity=".539"/><path d="m37 22.5c0-.277-.223-.5-.5-.5l-4.5-1h-8-8l-4.5 1c-.277 0-.5.223-.5.5v3c0 .277.229596.43991.5.5l4.5 1h8 8l4.5-1c.270404-.06009.5-.223.5-.5z" opacity=".539"/></g><path d="m22.162109 18.726562-2.193359-2.193359-2.304688 1.455078-.867187 1.992188-6.75 6.75 4.607422 4.607422-1.964844 1.748047 9.914063 9.914062h14.208984c2.874065 0 5.1875-2.280321 5.1875-5.111328-8.496284-10.482081-18-21.888672-19.837891-19.16211z" fill="url(#b)" opacity=".2"/><path d="m36.970703 10.988281-4.369141 1.177735-4.519531 9.5 3.917969 8.328125 11.285156 11.285156c.45228-.7842.714844-1.690387.714844-2.660156v-20.601563z" fill="url(#c)" opacity=".2"/><path d="m26.000804 9.9953918 4.531314 4.5313142-3.142254 6.655326-6.28785-6.28785z" fill="url(#d)" opacity=".2"/><path d="m15.208984 14.617188c-.123062-.0059-.241909.03097-.339843.128906l-2.123047 2.123047c-.195869.195869-.195869.511162 0 .707031l2.193359 3.445312c-.175549.354193-.327571.719568-.455078 1.09375l-3.984375.884766c-.277 0-.5.223-.5.5v3c0 .277.229596.43991.5.5l3.986328.886719c.12346.370523.270907.732616.441406 1.083984l-2.197265 3.453125c-.148715.233695-.195869.511162 0 .707031l2.121093 2.123047c.19587.195869.511163.195869.707032 0l3.453125-2.199218c.357212.177942.725864.331927 1.103515.460937l.884766 3.984375c0 .277.223.5.5.5h3c.277 0 .43991-.229596.5-.5l.886719-3.986328c.376029-.125362.743365-.275428 1.099609-.449219l3.4375 2.189453c.195869.195869.511162.195869.707031 0l2.123047-2.123047c.195869-.195869.148715-.473336 0-.707031l-.9375-1.472656-.699218.279297-2.240235-3.359375c-1.128979 2.510379-3.624392 4.126092-6.376953 4.128906-3.865993 0-7-3.134007-7-7 .0031-2.700827 1.559779-5.15891 4-6.316406v-2.152344c-.336475.11516-.665829.250162-.986328.404297l-3.4375-2.189453c-.116848-.07436-.244125-.123012-.367188-.128906z" fill="#fff"/><path d="m26 10-5 .462891v19.537109l5-1v-8l6 9 5-2-6.5-8.414062 6.503906-8.585938-5.236328-1-5.767578 8.5z" fill="#fff"/><path d="m4 38v1c0 3 2 5 5 5h30c3 0 5-2 5-5v-1c0 3-2 5-5 5h-30c-3 0-5-2-5-5z" opacity=".2"/></svg>"##;

const SVG_SYSTEM_SHUTDOWN: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
  <defs id="defs3051">
    <style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
  </defs>
  <path style="fill:currentColor;fill-opacity:1;stroke:none" d="M 15,4 15,4.05078 15,5.054686 15,13 17,13 17,5.050781 17,4.046875 17,4 16,4 15,4 Z m 3,0.169922 0,1.021484 A 11,11 0 0 1 27,16 11,11 0 0 1 16,27 11,11 0 0 1 5,16 11,11 0 0 1 14,5.189453 L 14,4.181641 A 12,12 0 0 0 4,16 12,12 0 0 0 16,28 12,12 0 0 0 28,16 12,12 0 0 0 18,4.169922 Z" class="ColorScheme-Text"/>
</svg>"##;

const SVG_SYSTEM_REBOOT: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg version="1.1" viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg">
<defs>
<style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
</defs>
<path d="m16 4a12 12 0 0 0-12 12 12 12 0 0 0 0.0722656 1h1.0058594a11 11 0 0 1-0.078125-1 11 11 0 0 1 11-11 11 11 0 0 1 9.183594 5h-4.183594v1h4.796875 0.203125 0.90625 0.09375v-6h-1v4.3886719a12 12 0 0 0-10-5.3886719zm10.951172 11a11 11 0 0 1 0.048828 1 11 11 0 0 1-11 11 11 11 0 0 1-9.1835938-5h4.1835938v-1h-4.796875-1.1113281-0.0917969v1 5h1v-4.384766a12 12 0 0 0 10 5.384766 12 12 0 0 0 12-12 12 12 0 0 0-0.044922-1h-1.003906z" class="ColorScheme-Text" fill="currentColor"/>
</svg>"##;

const SVG_SYSTEM_SUSPEND: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg">
    <style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
    <g class="ColorScheme-Text" fill="currentColor">
        <path d="M9.762 5.762a12 12 0 0 0-.004.008A12 12 0 0 0 4 16a12 12 0 0 0 12 12 12 12 0 0 0 10.238-5.762l-.732-.732A11 11 0 0 1 20 23 11 11 0 0 1 9 12a11 11 0 0 1 1.486-5.502 11 11 0 0 1 .008-.004zM8.8 7.702A12 12 0 0 0 8 12a12 12 0 0 0 12 12 12 12 0 0 0 4.326-.82A11 11 0 0 1 16 27 11 11 0 0 1 5 16a11 11 0 0 1 3.8-8.299zM22 6v1h2.293l-2 2-.293.293V10h4V9h-2.293l2-2L26 6.707V6h-.707z"/>
        <path d="M20 13v1h2.293l-2 2-.293.293V17h4v-1h-2.293l2-2 .293-.293V13h-.707zm-6-5v1h2.293l-2 2-.293.293V12h4v-1h-2.293l2-2L18 8.707V8h-.707z"/>
    </g>
</svg>"##;

const SVG_SYSTEM_HIBERNATE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg">
    <style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
    <g class="ColorScheme-Text" fill="currentColor">
        <path d="M9.79 5.242l-.868.5 2.146 3.717-3.615-.969-.258.967 4.58 1.227L14.844 16H8.707l-3.353-3.354-.708.708L7.293 16H3v1h4.293l-2.647 2.646.708.708L8.707 17h6.137l-3.069 5.316-4.58 1.227.258.967 3.615-.969-2.146 3.717.867.5 2.147-3.717.968 3.615.965-.26-1.226-4.58L16 17l3.357 5.816-1.226 4.58.965.26.968-3.615 2.147 3.717.867-.5-2.146-3.717 3.615.969.258-.967-4.58-1.227L17.156 17h6.137l3.353 3.354.708-.708L24.707 17H29v-1h-4.293l2.647-2.646-.708-.708L23.293 16H16v-.002l-3.357-5.814 1.226-4.58-.965-.26-.968 3.615zM23 4v1h2.293l-2 2-.293.293V8h4V7h-2.293l2-2L27 4.707V4h-.707z"/>
        <path d="M20 10v1h2.293l-2 2-.293.293V14h4v-1h-2.293l2-2 .293-.293V10h-.707zm-4-5v1h2.293l-2 2-.293.293V9h4V8h-2.293l2-2L20 5.707V5h-.707z"/>
    </g>
</svg>"##;

const SVG_SYSTEM_LOGOUT: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
  <defs id="defs3051">
    <style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
  </defs>
  <path style="fill:currentColor;fill-opacity:1;stroke:none" d="M 16,4 A 12,12 0 0 0 4,16 12,12 0 0 0 16,28 12,12 0 0 0 28,16 12,12 0 0 0 16,4 Z m 0,1 A 11,11 0 0 1 27,16 11,11 0 0 1 16,27 11,11 0 0 1 5,16 11,11 0 0 1 16,5 Z m 3.3,3 L 11,16 19.3,24 20,23.3 12.5,16 20,8.7 19.2,8 Z" class="ColorScheme-Text"/>
</svg>"##;

const SVG_SYSTEM_LOCK: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg version="1.1" viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg">
    <defs>
        <style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
    </defs>
    <path class="ColorScheme-Text" d="m16 4c-6.627417 0-12 5.372583-12 12s5.372583 12 12 12 12-5.372583 12-12-5.372583-12-12-12zm0 1c6.075132 0 11 4.924868 11 11s-4.924868 11-11 11-11-4.924868-11-11 4.924868-11 11-11zm0 3c-2.216 0-4 1.784-4 4v4h-2v8h12v-8h-2v-4c0-2.216-1.784-4-4-4zm0 1c1.662 0 3 1.561 3 3.5v3.5h-6v-3.5c0-1.939 1.338-3.5 3-3.5z" fill="currentColor"/>
</svg>"##;

const SVG_SYSTEM_SWITCH_USER: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
  <defs id="defs3051">
    <style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
  </defs>
  <path style="fill:currentColor;fill-opacity:1;stroke:none" id="path1011" d="m 18,8 c -1.285847,0 -2.416677,0.617761 -3.148438,1.560547 0.329275,0.112144 0.645954,0.250811 0.945313,0.417969 C 16.345252,9.381114 17.12515,9 18,9 c 1.656852,0 3,1.343147 3,3 0,1.398728 -0.960744,2.564273 -2.255859,2.896484 6.21e-4,0.03481 0.0059,0.06856 0.0059,0.103516 0,0.318095 -0.03235,0.627594 -0.08203,0.931641 C 20.557605,15.612437 22,13.980455 22,12 22,9.790862 20.209142,8 18,8 Z M 13,20 C 9.1189423,20 6,22.689616 6,26.435547 6.0033,26.962961 6.06906,27.487484 6.198034,28 H 7.3097713 18.972229 19.856521 C 19.985519,27.487484 19.997113,26.963414 20,26.436 20,22.690069 16.881058,20 13,20 Z m 0.027,0.992 c 3.813868,0 5.973,2.274365 5.973,5.444 -0.0011,0.174256 -0.01135,0.347611 -0.029,0.521 H 7.0290003 c -0.01765,-0.173389 -0.02741,-0.347197 -0.029,-0.521453 0,-3.169635 2.179706,-5.443547 5.9727347,-5.443547 z m 6.224205,-3.992043 c -0.25686,0 -0.509035,0.01606 -0.758429,0.04297 -0.12585,0.383697 -0.291433,0.751062 -0.492776,1.097662 0.410411,-0.09117 0.830028,-0.138336 1.251205,-0.140627 3.153826,1e-6 5.710515,2.46245 5.710515,5.500033 -9e-4,0.166968 -0.009,0.333807 -0.0264,0.500003 h -4.53029 c 0.108739,0.321192 0.202605,0.648801 0.269709,0.986334 l 5.134641,0.01367 C 25.93326,24.508838 25.99723,24.005434 26,23.499992 26,19.910121 22.97845,16.999953 19.251205,16.999954 Z M 13,11 c -2.209143,0 -3.9999997,1.790862 -3.9999997,4 0,2.20914 1.7908567,4.000001 3.9999997,4 2.209142,1e-6 4,-1.79086 4,-4 0,-2.209138 -1.790858,-4 -4,-4 z m 0,1 c 1.656852,0 3,1.343147 3,3 0,1.656854 -1.343148,3.000001 -3,3 -1.656852,1e-6 -3,-1.343146 -3,-3 0,-1.656853 1.343148,-3 3,-3 z m 3.027,-8.200001 c -2.034737,0 -3.684211,1.645485 -3.684211,3.675293 V 8.090575 L 10.736,6.388 10.123,7.130953 12.356,9.358 13,10 13.582,9.257328 15.646,7.03 15.002,6.388291 13.295,8.091 V 7.475 c 0,-1.449863 1.278621,-2.624917 2.732,-2.624917 1.453379,0 1.973,0.519045 1.973,1.968908 h 1 C 19,4.718823 17.883088,3.839078 16.027,3.799999 Z" class="ColorScheme-Text"/>
</svg>"##;

const SVG_DIALOG_CANCEL: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
  <defs id="defs3051">
    <style type="text/css" id="current-color-scheme">.ColorScheme-Text { color: #fcfcfc; } </style>
  </defs>
 <path style="fill:currentColor;fill-opacity:1;stroke:none" d="M16 4A12 12 0 0 0 7.886719 7.179688L7.885 7.178A12 12 0 0 0 7.847656 7.214844 12 12 0 0 0 4 16 12 12 0 0 0 16 28 12 12 0 0 0 24.11328 24.820312L24.12 24.822A12 12 0 0 0 24.15234 24.785156 12 12 0 0 0 28 16 12 12 0 0 0 16 4M16 5A11 11 0 0 1 27 16 11 11 0 0 1 24.11523 23.408203L8.592 7.885A11 11 0 0 1 16 5M7.885 8.592L23.408 24.12A11 11 0 0 1 16 27 11 11 0 0 1 5 16 11 11 0 0 1 7.884766 8.591797" class="ColorScheme-Text"/>
</svg>"##;

/// Cached GPU textures for the KDE logo, all action icons, and the cancel button icon.
pub struct IconSet {
    pub kde_logo: TextureHandle,
    pub cancel: TextureHandle,
    actions: HashMap<&'static str, TextureHandle>,
}

impl IconSet {
    /// Rasterizes and uploads all dialog icons for the given theme palette.
    pub fn load(ctx: &egui::Context, palette: &ThemePalette) -> Self {
        let kde_logo = load_texture(ctx, "kde-logo", "kde", None, palette, 80);
        let cancel = load_texture(ctx, "dialog-cancel", "dialog-cancel", None, palette, 32);

        let mut actions = HashMap::with_capacity(ACTIONS.len());
        for action in ACTIONS {
            let handle = load_texture(
                ctx,
                action.icon,
                action.icon,
                Some(action.fallback_icon),
                palette,
                72,
            );
            actions.insert(action.icon, handle);
        }

        Self {
            kde_logo,
            cancel,
            actions,
        }
    }

    /// Returns the cached texture handle for an action icon name.
    pub fn action_icon(&self, name: &str) -> Option<&TextureHandle> {
        self.actions.get(name)
    }
}

/// Builds the window taskbar/titlebar icon (64x64 RGBA) from the Breeze `system-shutdown` SVG.
pub fn build_window_icon() -> Option<Arc<IconData>> {
    let dark_palette = ThemePalette::for_mode(true);
    let pixmap = rasterize_svg_icon("system-shutdown", &dark_palette, 64)?;
    let width = pixmap.width();
    let height = pixmap.height();
    let rgba = pixmap.take_demultiplied();
    Some(Arc::new(IconData {
        rgba,
        width,
        height,
    }))
}

fn embedded_svg(name: &str) -> Option<&'static str> {
    match name {
        "kde" => Some(SVG_KDE_LOGO),
        "system-shutdown" => Some(SVG_SYSTEM_SHUTDOWN),
        "system-reboot" => Some(SVG_SYSTEM_REBOOT),
        "system-suspend" => Some(SVG_SYSTEM_SUSPEND),
        "system-suspend-hibernate" => Some(SVG_SYSTEM_HIBERNATE),
        "system-log-out" => Some(SVG_SYSTEM_LOGOUT),
        "system-lock-screen" => Some(SVG_SYSTEM_LOCK),
        "system-switch-user" => Some(SVG_SYSTEM_SWITCH_USER),
        "dialog-cancel" => Some(SVG_DIALOG_CANCEL),
        _ => None,
    }
}

fn load_svg_source(name: &str, dark: bool) -> Option<String> {
    let disk_candidates: Vec<String> = if name == "kde" {
        vec!["/usr/share/icons/breeze/apps/48/kde.svg".to_string()]
    } else if dark {
        vec![
            format!("/usr/share/icons/breeze-dark/actions/32/{name}.svg"),
            format!("/usr/share/icons/breeze/actions/32/{name}.svg"),
        ]
    } else {
        vec![
            format!("/usr/share/icons/breeze/actions/32/{name}.svg"),
            format!("/usr/share/icons/breeze-dark/actions/32/{name}.svg"),
        ]
    };

    for path in disk_candidates {
        if Path::new(&path).is_file()
            && let Ok(content) = fs::read_to_string(&path)
        {
            return Some(content);
        }
    }

    embedded_svg(name).map(str::to_string)
}

/// Normalizes KDE `.ColorScheme-Text` and `currentColor` references for `resvg`.
pub fn apply_color_scheme(svg: &str, icon_color_hex: &str) -> String {
    if !svg.contains("ColorScheme-Text") {
        return svg.to_string();
    }

    svg.replace("#fcfcfc", icon_color_hex)
        .replace("#FCFCFC", icon_color_hex)
        .replace("#232629", icon_color_hex)
        .replace("#eff0f1", icon_color_hex)
        .replace("currentColor", icon_color_hex)
}

/// Rasterizes a named Breeze SVG icon into a premultiplied `tiny_skia::Pixmap` of `size x size`.
pub fn rasterize_svg_icon(name: &str, palette: &ThemePalette, size: u32) -> Option<Pixmap> {
    let raw_svg = load_svg_source(name, palette.dark)?;
    let themed_svg = apply_color_scheme(&raw_svg, palette.icon_color_hex);

    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&themed_svg, &options).ok()?;
    let mut pixmap = Pixmap::new(size, size)?;

    let svg_size = tree.size();
    let scale_x = size as f32 / svg_size.width();
    let scale_y = size as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);
    let offset_x = (size as f32 - svg_size.width() * scale) * 0.5;
    let offset_y = (size as f32 - svg_size.height() * scale) * 0.5;
    let transform = Transform::from_translate(offset_x, offset_y).pre_scale(scale, scale);

    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Some(pixmap)
}

fn load_texture(
    ctx: &egui::Context,
    texture_id: &str,
    icon_name: &str,
    fallback_icon: Option<&str>,
    palette: &ThemePalette,
    size_px: u32,
) -> TextureHandle {
    let pixmap = rasterize_svg_icon(icon_name, palette, size_px)
        .or_else(|| fallback_icon.and_then(|fb| rasterize_svg_icon(fb, palette, size_px)));

    let dim = [size_px as usize, size_px as usize];
    let color_image = match pixmap {
        Some(pixmap) => ColorImage::from_rgba_premultiplied(dim, pixmap.data()),
        None => ColorImage::filled(dim, egui::Color32::TRANSPARENT),
    };

    ctx.load_texture(texture_id, color_image, TextureOptions::LINEAR)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_ICON_NAMES: [&str; 9] = [
        "kde",
        "system-shutdown",
        "system-reboot",
        "system-suspend",
        "system-suspend-hibernate",
        "system-log-out",
        "system-lock-screen",
        "system-switch-user",
        "dialog-cancel",
    ];

    #[test]
    fn all_embedded_svgs_rasterize_in_dark_and_light_modes() {
        for dark in [true, false] {
            let palette = ThemePalette::for_mode(dark);
            for icon in ALL_ICON_NAMES {
                let raw = embedded_svg(icon).expect("embedded SVG missing");
                let themed = apply_color_scheme(raw, palette.icon_color_hex);
                let tree = usvg::Tree::from_str(&themed, &usvg::Options::default())
                    .expect("SVG failed to parse");
                let mut pixmap = Pixmap::new(64, 64).expect("pixmap allocation failed");
                let svg_size = tree.size();
                let scale = (64.0 / svg_size.width()).min(64.0 / svg_size.height());
                resvg::render(
                    &tree,
                    Transform::from_scale(scale, scale),
                    &mut pixmap.as_mut(),
                );

                let non_transparent = pixmap.pixels().iter().filter(|p| p.alpha() > 0).count();
                assert!(
                    non_transparent > 10,
                    "icon {icon} (dark={dark}) rendered blank"
                );

                if icon != "kde" {
                    let expected_rgb = if dark { (252, 252, 252) } else { (35, 38, 41) };
                    let has_expected_color = pixmap.pixels().iter().any(|p| {
                        p.alpha() == 255
                            && p.red() == expected_rgb.0
                            && p.green() == expected_rgb.1
                            && p.blue() == expected_rgb.2
                    });
                    assert!(
                        has_expected_color,
                        "icon {icon} (dark={dark}) did not use expected theme color {:?}",
                        expected_rgb
                    );
                }
            }
        }
    }

    #[test]
    fn window_icon_builds_with_valid_rgba_buffer() {
        let icon = build_window_icon().expect("window icon should rasterize");
        assert_eq!(icon.width, 64);
        assert_eq!(icon.height, 64);
        assert_eq!(icon.rgba.len(), 64 * 64 * 4);
    }
}
