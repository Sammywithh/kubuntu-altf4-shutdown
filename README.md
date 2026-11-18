# Kubuntu 26.04 Alt+F4 Smart Shutdown Dialog

Ein modernes, natives **Herunterfahren- und Abmelde-Menü für Kubuntu (KDE Plasma 6)** in **Rust** (`eframe`/`egui`, `resvg`, `zbus`) im Stil von Kubuntu 26.04 / KDE Breeze, gesteuert über **`Alt + F4`**.

---

## ✨ Highlights & Funktionen

- **Intelligente `Alt + F4`-Erkennung**:
  - Wenn ein normales Anwendungsfenster aktiv ist: **`Alt + F4` schließt das Fenster** (Standardverhalten).
  - Wenn der Desktop aktiv ist (oder alle Fenster minimiert/geschlossen sind): **`Alt + F4` öffnet das Kubuntu Herunterfahren-Menü**.
- **Authentisches Kubuntu 26.04 / KDE Breeze Design**:
  - Große, übersichtliche Aktionskacheln mit originalen KDE Breeze System-Icons:
    - **Herunterfahren** (`H` / Enter)
    - **Neu starten** (`N`)
    - **Energie sparen** (`E`)
    - **Ruhezustand** (`R`)
    - **Abmelden** (`A`)
    - **Sperren** (`S`)
    - **Benutzer wechseln** (`B`)
  - **Dynamischer Informationstext**: Zeigt bei Hover oder Tastaturfokus eine genaue Beschreibung der gewählten Aktion an.
  - **Volle Tastatursteuerung**: Navigation mit Pfeiltasten und `Tab`/`Shift+Tab`, Schnelltasten (`H`, `N`, `E`, `R`, `A`, `S`, `B`), `Enter`/`Leertaste` zum Bestätigen und `Esc` zum Abbrechen.
  - **Automatisches Dark / Light Theme**: Liest `~/.config/kdeglobals` und passt Farben sowie SVG-Icons automatisch an KDE Breeze Dark / Light an.
- **Natives Rust-Binary (Zero Python Runtime, Zero CPU im Hintergrund)**:
  - Wird als schnelles natives Binary per D-Bus On-Demand aktiviert (`org.kubuntu.ShutdownDialog` auf `/Dialog`) und benötigt keine Python- oder PyQt-Abhängigkeiten zur Laufzeit.

---

## 🛠️ Kompilieren & Installation

### Voraussetzungen zum Selbstkompilieren
- Rust Toolchain (`cargo` / `rustc` ≥ 1.95)

### Bauen & Installieren
```bash
git clone https://github.com/Sammywithh/kubuntu-altf4-shutdown.git
cd kubuntu-altf4-shutdown
cargo build --release --locked
./install.sh
```

> **Hinweis**: Wenn du ein vorkompiliertes Linux-Release-Archiv aus den GitHub Releases / Actions-Artefakten entpackst, erkennt `./install.sh` das beiliegende `kubuntu-shutdown-dialog`-Binary automatisch und installiert es ohne Cargo.

Das Installationsskript:
1. Installiert das native Rust-Binary nach `~/.local/bin/kubuntu-shutdown-dialog`.
2. Registriert den D-Bus Dienst für On-Demand-Aktivierung.
3. Installiert das KWin 6 Skript für intelligentes `Alt+F4`.
4. Schaltet die Tastenkombination `Alt+F4` live scharf.

---

## 🧪 Testen & Optionen

Direktes Starten des Menüs (kompiliert bei Bedarf automatisch im Release-Modus):
```bash
./test_dialog.sh
```

Verfügbare CLI-Flags von `kubuntu-shutdown-dialog`:
- `--native`: Öffnet direkt das offizielle KDE Plasma 6 Abmeldemenü.
- `--daemon`: Startet den schlanken D-Bus-Hintergrunddienst ohne offenes Fenster.
- `--no-dbus`: Öffnet den Dialog direkt ohne D-Bus-Registrierung.
- `--help` / `--version`: Zeigt Hilfe bzw. Version an.

---

## ⚙️ Automatisierter Linux-Build (GitHub Actions)

Über [`.github/workflows/build.yml`](.github/workflows/build.yml) wird bei jedem Push und Pull Request auf `main`/`master` sowie bei Tags `v*`:
- `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings` und `cargo test` ausgeführt,
- das optimierte Linux-Release-Binary (`kubuntu-shutdown-dialog`) gebaut,
- bei Pushes/Workflow-Läufen auf `main`/`master` das kompilierte Binary (`./kubuntu-shutdown-dialog`) automatisch mit `[skip ci]` zurück in den Branch committet und gepusht,
- ein vollständiges Installationsarchiv (`kubuntu-altf4-shutdown-linux-x86_64.tar.gz`) samt SHA256-Prüfsumme als Artefakt hochgeladen (und bei `v*`-Tags an ein GitHub Release angehängt).

---

## 🗑️ Deinstallation

```bash
./uninstall.sh
```
Dadurch wird die originale `Alt+F4` Verknüpfung von KWin wiederhergestellt und alle installierten Dateien werden sauber entfernt.
