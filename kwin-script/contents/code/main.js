/*
    Kubuntu Alt+F4 Smart Shutdown Handler
    - Closes active window if a window is focused
    - Shows Kubuntu shutdown menu if Desktop is focused (or all windows minimized)
*/

function isCloseableWindow(w) {
    if (!w) {
        console.info("Kubuntu Alt+F4: No active window -> Desktop detected.");
        return false;
    }

    // Check desktop, dock, or panel properties
    if (w.desktopWindow || w.dock || w.isDesktop || w.isDock) {
        console.info("Kubuntu Alt+F4: Window is desktop/dock -> Desktop detected.");
        return false;
    }

    // Plasma desktop / panel check
    if (w.resourceClass === "plasmashell" || w.resourceName === "plasmashell") {
        if (!w.normalWindow && !w.dialog) {
            console.info("Kubuntu Alt+F4: plasmashell background -> Desktop detected.");
            return false;
        }
    }

    // Check if normal window or dialog
    if (typeof w.normalWindow !== "undefined" && !w.normalWindow && !w.dialog) {
        console.info("Kubuntu Alt+F4: Not a normal/dialog window -> Desktop detected.");
        return false;
    }

    // Check if closeable
    if (typeof w.closeable !== "undefined" && !w.closeable) {
        console.info("Kubuntu Alt+F4: Window not closeable -> Desktop detected.");
        return false;
    }

    if (typeof w.deleted !== "undefined" && w.deleted) {
        return false;
    }

    return true;
}

function showShutdownMenu() {
    console.info("Kubuntu Alt+F4: Calling showShutdownMenu via DBus...");

    callDBus(
        "org.kubuntu.ShutdownDialog",
        "/Dialog",
        "org.kubuntu.ShutdownDialog",
        "showDialog",
        function() {}
    );
}

function handleAltF4() {
    var win = workspace.activeWindow;
    var winClass = win ? (win.resourceClass || "unknown") : "none";
    console.info("Kubuntu Alt+F4 triggered. Active window class:", winClass);

    if (isCloseableWindow(win)) {
        console.info("Kubuntu Alt+F4: Closing active window of class:", winClass);
        win.closeWindow();
    } else {
        showShutdownMenu();
    }
}

// Register global shortcut in KWin
registerShortcut(
    "AltF4DesktopShutdown",
    "Fenster schließen oder Herunterfahren-Menü anzeigen",
    "Alt+F4",
    handleAltF4
);

console.info("Kubuntu Alt+F4 Smart Shutdown script initialized.");
