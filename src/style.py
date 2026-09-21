"""
Kubuntu 26.04 Breeze Theme Styling for Shutdown Dialog.
Supports automatic Dark/Light mode detection and authentic KDE Breeze aesthetics.
"""

import os
import configparser


def is_dark_mode():
    """Detects whether KDE Plasma is using a dark color scheme by reading kdeglobals."""
    try:
        config_path = os.path.expanduser("~/.config/kdeglobals")
        if os.path.exists(config_path):
            config = configparser.ConfigParser(strict=False)
            config.read(config_path)
            
            # Check LookAndFeelPackage
            if "KDE" in config and "LookAndFeelPackage" in config["KDE"]:
                laf = config["KDE"]["LookAndFeelPackage"].lower()
                if "dark" in laf or "black" in laf:
                    return True
                if "light" in laf or ("breeze" in laf and "dark" not in laf):
                    return False
                    
            # Check General ColorScheme
            if "General" in config and "ColorScheme" in config["General"]:
                cs = config["General"]["ColorScheme"].lower()
                if "dark" in cs or "black" in cs:
                    return True
                if "light" in cs:
                    return False

            # Check BackgroundNormal color brightness
            if "Colors:Window" in config and "BackgroundNormal" in config["Colors:Window"]:
                rgb_str = config["Colors:Window"]["BackgroundNormal"]
                rgb = [int(c.strip()) for c in rgb_str.split(",") if c.strip().isdigit()]
                if len(rgb) >= 3:
                    brightness = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2]
                    return brightness < 128
    except Exception as e:
        print(f"Note: Error reading KDE color scheme ({e}), defaulting to dark.")

    return True  # Default to modern dark style for Kubuntu


def get_stylesheet(dark=None):
    """Returns the QSS stylesheet for the Kubuntu Breeze theme."""
    if dark is None:
        dark = is_dark_mode()

    if dark:
        bg_main = "#232629"
        bg_card = "#2a2e32"
        bg_card_hover = "#31363b"
        bg_card_pressed = "#1b1e20"
        border_color = "rgba(255, 255, 255, 0.12)"
        border_hover = "#3daee9"
        text_primary = "#fcfcfc"
        text_secondary = "#bdc3c7"
        accent_color = "#3daee9"
        btn_cancel_bg = "#31363b"
        btn_cancel_hover = "#3a4047"
    else:
        bg_main = "#eff0f1"
        bg_card = "#fcfcfc"
        bg_card_hover = "#e4e5e7"
        bg_card_pressed = "#dadbdc"
        border_color = "rgba(0, 0, 0, 0.12)"
        border_hover = "#1d99f3"
        text_primary = "#232629"
        text_secondary = "#606468"
        accent_color = "#1d99f3"
        btn_cancel_bg = "#e0e2e4"
        btn_cancel_hover = "#d2d5d8"

    return f"""
    QDialog {{
        background-color: {bg_main};
        border-radius: 12px;
    }}

    QLabel#titleLabel {{
        color: {text_primary};
        font-size: 17px;
        font-weight: bold;
    }}

    QLabel#subtitleLabel {{
        color: {text_secondary};
        font-size: 13px;
    }}

    QLabel#descriptionLabel {{
        color: {text_primary};
        font-size: 12px;
        line-height: 1.4;
        padding: 8px 14px;
        background-color: {bg_card};
        border: 1px solid {border_color};
        border-radius: 8px;
    }}

    /* Action Tile Button (QToolButton) */
    QToolButton[class="actionButton"] {{
        background-color: {bg_card};
        color: {text_primary};
        border: 1px solid {border_color};
        border-radius: 10px;
        padding: 8px 4px;
        font-size: 12px;
        font-weight: 500;
        text-align: center;
    }}

    QToolButton[class="actionButton"]:hover {{
        background-color: {bg_card_hover};
        border: 1.5px solid {border_hover};
    }}

    QToolButton[class="actionButton"]:focus {{
        background-color: {bg_card_hover};
        border: 2px solid {border_hover};
        outline: none;
    }}

    QToolButton[class="actionButton"]:pressed {{
        background-color: {bg_card_pressed};
        border: 1.5px solid {border_hover};
    }}

    /* Cancel Button */
    QPushButton#cancelButton {{
        background-color: {btn_cancel_bg};
        color: {text_primary};
        border: 1px solid {border_color};
        border-radius: 6px;
        padding: 6px 22px;
        font-size: 13px;
        font-weight: 500;
    }}

    QPushButton#cancelButton:hover {{
        background-color: {btn_cancel_hover};
        border: 1px solid {border_hover};
    }}

    QPushButton#cancelButton:focus {{
        border: 2px solid {border_hover};
        outline: none;
    }}
    """
