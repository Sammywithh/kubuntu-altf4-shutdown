//! Kubuntu 26.04 Breeze Theme Styling for Shutdown Dialog.
//! Supports automatic Dark/Light mode detection via `~/.config/kdeglobals`
//! and authentic KDE Breeze aesthetics.

use eframe::egui::Color32;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Color palette matching KDE Breeze Dark / Light themes.
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub dark: bool,
    pub bg_main: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_card_pressed: Color32,
    pub border_color: Color32,
    pub border_hover: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub btn_cancel_bg: Color32,
    pub btn_cancel_hover: Color32,
    pub icon_color_hex: &'static str,
}

impl ThemePalette {
    /// Builds the Breeze Dark or Breeze Light palette.
    pub const fn for_mode(dark: bool) -> Self {
        if dark {
            Self {
                dark: true,
                bg_main: Color32::from_rgb(0x23, 0x26, 0x29),
                bg_card: Color32::from_rgb(0x2a, 0x2e, 0x32),
                bg_card_hover: Color32::from_rgb(0x31, 0x36, 0x3b),
                bg_card_pressed: Color32::from_rgb(0x1b, 0x1e, 0x20),
                border_color: Color32::from_rgba_premultiplied(31, 31, 31, 31),
                border_hover: Color32::from_rgb(0x3d, 0xae, 0xe9),
                text_primary: Color32::from_rgb(0xfc, 0xfc, 0xfc),
                text_secondary: Color32::from_rgb(0xbd, 0xc3, 0xc7),
                btn_cancel_bg: Color32::from_rgb(0x31, 0x36, 0x3b),
                btn_cancel_hover: Color32::from_rgb(0x3a, 0x40, 0x47),
                icon_color_hex: "#fcfcfc",
            }
        } else {
            Self {
                dark: false,
                bg_main: Color32::from_rgb(0xef, 0xf0, 0xf1),
                bg_card: Color32::from_rgb(0xfc, 0xfc, 0xfc),
                bg_card_hover: Color32::from_rgb(0xe4, 0xe5, 0xe7),
                bg_card_pressed: Color32::from_rgb(0xda, 0xdb, 0xdc),
                border_color: Color32::from_rgba_premultiplied(0, 0, 0, 31),
                border_hover: Color32::from_rgb(0x1d, 0x99, 0xf3),
                text_primary: Color32::from_rgb(0x23, 0x26, 0x29),
                text_secondary: Color32::from_rgb(0x60, 0x64, 0x68),
                btn_cancel_bg: Color32::from_rgb(0xe0, 0xe2, 0xe4),
                btn_cancel_hover: Color32::from_rgb(0xd2, 0xd5, 0xd8),
                icon_color_hex: "#232629",
            }
        }
    }

    /// Detects the active KDE theme and returns the corresponding palette.
    pub fn detect() -> Self {
        Self::for_mode(is_dark_mode())
    }
}

/// Detects whether KDE Plasma is using a dark color scheme by reading `~/.config/kdeglobals`.
pub fn is_dark_mode() -> bool {
    let Some(home) = env::var_os("HOME") else {
        return true;
    };
    let config_path: PathBuf = PathBuf::from(home).join(".config/kdeglobals");
    let Ok(content) = fs::read_to_string(&config_path) else {
        return true;
    };

    detect_dark_mode_from_ini(&content).unwrap_or(true)
}

/// Parses KDE `kdeglobals` INI content to determine dark (`Some(true)`) or light (`Some(false)`).
pub fn detect_dark_mode_from_ini(content: &str) -> Option<bool> {
    let mut section = String::new();
    let mut look_and_feel: Option<String> = None;
    let mut color_scheme: Option<String> = None;
    let mut background_normal: Option<String> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        match section.as_str() {
            "KDE" if key == "LookAndFeelPackage" => {
                look_and_feel = Some(value.to_ascii_lowercase());
            }
            "General" if key == "ColorScheme" => {
                color_scheme = Some(value.to_ascii_lowercase());
            }
            "Colors:Window" if key == "BackgroundNormal" => {
                background_normal = Some(value.to_string());
            }
            _ => {}
        }
    }

    if let Some(laf) = look_and_feel {
        if laf.contains("dark") || laf.contains("black") {
            return Some(true);
        }
        if laf.contains("light") || (laf.contains("breeze") && !laf.contains("dark")) {
            return Some(false);
        }
    }

    if let Some(cs) = color_scheme {
        if cs.contains("dark") || cs.contains("black") {
            return Some(true);
        }
        if cs.contains("light") {
            return Some(false);
        }
    }

    if let Some(rgb_str) = background_normal {
        let rgb: Vec<u32> = rgb_str
            .split(',')
            .filter_map(|c| c.trim().parse::<u32>().ok())
            .collect();
        if rgb.len() >= 3 {
            let brightness =
                0.299 * (rgb[0] as f64) + 0.587 * (rgb[1] as f64) + 0.114 * (rgb[2] as f64);
            return Some(brightness < 128.0);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_breeze_dark_look_and_feel() {
        let ini = r#"
[KDE]
LookAndFeelPackage=org.kde.breezedark.desktop
"#;
        assert_eq!(detect_dark_mode_from_ini(ini), Some(true));
    }

    #[test]
    fn detects_breeze_light_look_and_feel() {
        let ini = r#"
[KDE]
LookAndFeelPackage=org.kde.breeze.desktop
"#;
        assert_eq!(detect_dark_mode_from_ini(ini), Some(false));
    }

    #[test]
    fn detects_color_scheme_fallback() {
        let dark_ini = r#"
[General]
ColorScheme=BreezeDark
"#;
        let light_ini = r#"
[General]
ColorScheme=BreezeLight
"#;
        assert_eq!(detect_dark_mode_from_ini(dark_ini), Some(true));
        assert_eq!(detect_dark_mode_from_ini(light_ini), Some(false));
    }

    #[test]
    fn detects_rgb_brightness_when_names_absent() {
        let dark_rgb = r#"
[Colors:Window]
BackgroundNormal=35,38,41
"#;
        let light_rgb = r#"
[Colors:Window]
BackgroundNormal=239,240,241
"#;
        assert_eq!(detect_dark_mode_from_ini(dark_rgb), Some(true));
        assert_eq!(detect_dark_mode_from_ini(light_rgb), Some(false));
    }

    #[test]
    fn returns_none_for_empty_or_unrecognized_ini() {
        assert_eq!(detect_dark_mode_from_ini(""), None);
        assert_eq!(
            detect_dark_mode_from_ini("[General]\nColorScheme=Custom\n"),
            None
        );
    }
}
