#!/usr/bin/env python3
"""
Kubuntu 26.04 Style Alt+F4 Shutdown Dialog.
Provides a modern KDE Breeze style power menu for Kubuntu.
"""

import sys
import os
import argparse
from PyQt6 import QtWidgets, QtCore, QtGui
from PyQt6.QtCore import Qt, QSize
from PyQt6.QtWidgets import (
    QApplication, QDialog, QVBoxLayout, QHBoxLayout,
    QGridLayout, QPushButton, QToolButton, QLabel, QWidget, QFrame
)

from actions import ACTIONS, action_kde_native_prompt
from style import get_stylesheet, is_dark_mode


class ActionTileButton(QToolButton):
    """A large action tile button with icon on top and label underneath (KDE Breeze Style)."""

    hovered = QtCore.pyqtSignal(str)

    def __init__(self, action_id, action_data, parent=None):
        super().__init__(parent)
        self.action_id = action_id
        self.action_data = action_data
        
        self.setProperty("class", "actionButton")
        self.setCursor(QtGui.QCursor(Qt.CursorShape.PointingHandCursor))
        self.setSizePolicy(QtWidgets.QSizePolicy.Policy.Expanding, QtWidgets.QSizePolicy.Policy.Fixed)
        self.setFixedSize(140, 95)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextUnderIcon)

        # Set icon
        icon_name = action_data["icon"]
        fallback = QtGui.QIcon.fromTheme(action_data.get("fallback_icon", ""))
        icon = QtGui.QIcon.fromTheme(icon_name, fallback)
        self.setIcon(icon)
        self.setIconSize(QSize(36, 36))

        # Text with hotkey
        hotkey = action_data.get("hotkey", "")
        title = action_data["title"]
        self.setText(f"{title}\n({hotkey})")

    def enterEvent(self, event):
        self.hovered.emit(self.action_data["description"])
        super().enterEvent(event)

    def focusInEvent(self, event):
        self.hovered.emit(self.action_data["description"])
        super().focusInEvent(event)


class KubuntuShutdownDialog(QDialog):
    """Main shutdown dialog matching Kubuntu 26.04 Breeze style."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Kubuntu – Sitzung beenden")
        self.setWindowIcon(QtGui.QIcon.fromTheme("system-shutdown", QtGui.QIcon.fromTheme("kde")))
        self.setWindowFlags(
            Qt.WindowType.Dialog
            | Qt.WindowType.CustomizeWindowHint
            | Qt.WindowType.WindowTitleHint
            | Qt.WindowType.WindowCloseButtonHint
            | Qt.WindowType.WindowStaysOnTopHint
        )
        self.setModal(True)

        self.buttons = []
        self.init_ui()
        self.apply_theme()

    def apply_theme(self):
        """Applies the KDE Breeze stylesheet."""
        self.setStyleSheet(get_stylesheet())

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(28, 24, 28, 24)
        main_layout.setSpacing(16)
        main_layout.setSizeConstraint(QtWidgets.QLayout.SizeConstraint.SetFixedSize)

        # --- Header ---
        header_layout = QHBoxLayout()
        header_layout.setSpacing(14)

        icon_label = QLabel(self)
        kde_icon = QtGui.QIcon.fromTheme(
            "distributor-logo-kubuntu",
            QtGui.QIcon.fromTheme("kde", QtGui.QIcon.fromTheme("system-shutdown"))
        )
        icon_label.setPixmap(kde_icon.pixmap(40, 40))
        header_layout.addWidget(icon_label)

        title_vbox = QVBoxLayout()
        title_vbox.setSpacing(3)

        self.title_label = QLabel("Was möchten Sie tun?", self)
        self.title_label.setObjectName("titleLabel")
        title_vbox.addWidget(self.title_label)

        self.subtitle_label = QLabel("Wählen Sie eine Aktion für Ihre Kubuntu-Sitzung:", self)
        self.subtitle_label.setObjectName("subtitleLabel")
        title_vbox.addWidget(self.subtitle_label)

        header_layout.addLayout(title_vbox)
        header_layout.addStretch()
        main_layout.addLayout(header_layout)

        # --- Buttons Grid ---
        # Row 1: 4 items (Herunterfahren, Neu starten, Energie sparen, Ruhezustand)
        # Row 2: 3 items (Abmelden, Sperren, Benutzer wechseln)
        grid_layout = QGridLayout()
        grid_layout.setSpacing(12)

        button_order = [
            ("shutdown", 0, 0),
            ("reboot", 0, 1),
            ("suspend", 0, 2),
            ("hibernate", 0, 3),
            ("logout", 1, 0),
            ("lock", 1, 1),
            ("switch_user", 1, 2),
        ]

        for action_id, row, col in button_order:
            if action_id in ACTIONS:
                data = ACTIONS[action_id]
                btn = ActionTileButton(action_id, data, self)
                btn.clicked.connect(lambda checked, a=data: self.execute_action(a))
                btn.hovered.connect(self.update_description)
                grid_layout.addWidget(btn, row, col)
                self.buttons.append(btn)

        main_layout.addLayout(grid_layout)

        # --- Description Label ---
        self.desc_label = QLabel(
            ACTIONS["shutdown"]["description"], self
        )
        self.desc_label.setObjectName("descriptionLabel")
        self.desc_label.setWordWrap(True)
        self.desc_label.setMinimumHeight(44)
        main_layout.addWidget(self.desc_label)

        # --- Footer (Cancel button) ---
        footer_layout = QHBoxLayout()
        footer_layout.addStretch()

        self.cancel_btn = QPushButton("Abbrechen", self)
        self.cancel_btn.setObjectName("cancelButton")
        self.cancel_btn.setCursor(QtGui.QCursor(Qt.CursorShape.PointingHandCursor))
        self.cancel_btn.setIcon(QtGui.QIcon.fromTheme("dialog-cancel"))
        self.cancel_btn.setMinimumHeight(34)
        self.cancel_btn.clicked.connect(self.reject)
        footer_layout.addWidget(self.cancel_btn)

        main_layout.addLayout(footer_layout)

        # Set default focus to Shutdown button
        if self.buttons:
            self.buttons[0].setFocus()

    def update_description(self, text):
        self.desc_label.setText(text)

    def execute_action(self, action_data):
        """Closes dialog and executes the selected action."""
        self.accept()
        action_data["execute"]()

    def show_centered(self):
        """Centers and shows the dialog on active screen."""
        self.apply_theme()
        screen = QtGui.QGuiApplication.primaryScreen()
        if screen:
            geo = screen.availableGeometry()
            hint = self.sizeHint()
            self.move(
                geo.center().x() - hint.width() // 2,
                geo.center().y() - hint.height() // 2
            )
        if self.buttons:
            self.buttons[0].setFocus()
            self.update_description(ACTIONS["shutdown"]["description"])
        self.show()
        self.raise_()
        self.activateWindow()

    def keyPressEvent(self, event: QtGui.QKeyEvent):
        key = event.key()
        text = event.text().upper()

        if key == Qt.Key.Key_Escape:
            self.reject()
            return

        # Check hotkeys
        for btn in self.buttons:
            hotkey = btn.action_data.get("hotkey", "").upper()
            if hotkey and text == hotkey:
                btn.setFocus()
                self.execute_action(btn.action_data)
                return

        super().keyPressEvent(event)


class DBusService(QtCore.QObject):
    """Exposes showDialog method via D-Bus."""

    def __init__(self, dialog, app):
        super().__init__()
        self.dialog = dialog
        self.app = app

    @QtCore.pyqtSlot()
    def showDialog(self):
        self.dialog.show_centered()

    @QtCore.pyqtSlot()
    def showNativePrompt(self):
        action_kde_native_prompt()


def main():
    parser = argparse.ArgumentParser(description="Kubuntu Alt+F4 Shutdown Menu")
    parser.add_argument(
        "--native",
        action="store_true",
        help="Open official KDE Plasma 6 Logout greeter directly",
    )
    parser.add_argument(
        "--daemon",
        action="store_true",
        help="Run as D-Bus service in background",
    )
    args = parser.parse_args()

    if args.native:
        action_kde_native_prompt()
        return 0

    app = QApplication(sys.argv)
    app.setApplicationName("Kubuntu Shutdown Dialog")
    app.setDesktopFileName("kubuntu-shutdown-dialog")
    # Prevent app from quitting when dialog is closed if running in daemon mode
    app.setQuitOnLastWindowClosed(not args.daemon)

    dialog = KubuntuShutdownDialog()

    # Register D-Bus service
    try:
        from PyQt6 import QtDBus
        bus = QtDBus.QDBusConnection.sessionBus()
        if bus.isConnected():
            service_adaptor = DBusService(dialog, app)
            
            # Export with custom adaptor interface
            try:
                class ShutdownAdaptor(QtDBus.QDBusAbstractAdaptor):
                    QtCore.pyqtClassInfo("D-Bus Interface", "org.kubuntu.ShutdownDialog")

                    def __init__(self, parent):
                        super().__init__(parent)

                    @QtCore.pyqtSlot()
                    def showDialog(self):
                        self.parent().showDialog()

                    @QtCore.pyqtSlot()
                    def showNativePrompt(self):
                        self.parent().showNativePrompt()

                adaptor = ShutdownAdaptor(service_adaptor)
            except Exception as e:
                print(f"Note: Adaptor error: {e}")

            bus.registerService("org.kubuntu.ShutdownDialog")
            bus.registerObject("/Dialog", service_adaptor, QtDBus.QDBusConnection.RegisterOption.ExportAllSlots | QtDBus.QDBusConnection.RegisterOption.ExportAdaptors)
    except Exception as e:
        print(f"Note: D-Bus service registration: {e}")

    if args.daemon:
        return app.exec()

    dialog.show_centered()
    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
