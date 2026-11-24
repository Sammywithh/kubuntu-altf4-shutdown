# Kubuntu 26.04 Alt+F4 Smart Shutdown Dialog

Natives Herunterfahren- und Abmeldemenü für Kubuntu (KDE Plasma 6) in Rust (`eframe`/`egui`, `resvg`, `zbus`) im KDE-Breeze-Design, gesteuert über `Alt + F4`.

---

## Funktionen

- **Kontextabhängige `Alt + F4`-Steuerung**:
  - Aktives Anwendungsfenster: `Alt + F4` schließt das Fenster (Standardverhalten).
  - Aktiver Desktop (oder alle Fenster minimiert/geschlossen): `Alt + F4` öffnet das Herunterfahren-Menü.
- **KDE-Breeze-Oberfläche**:
  - Sieben Aktionskacheln mit KDE-Breeze-System-Icons:
    - **Herunterfahren** (`H` / `Enter`)
    - **Neu starten** (`N`)
    - **Energie sparen** (`E`)
    - **Ruhezustand** (`R`)
    - **Abmelden** (`A`)
    - **Sperren** (`S`)
    - **Benutzer wechseln** (`B`)
  - **Dynamische Beschreibung**: Zeigt bei Maus-Hover oder Tastaturfokus eine Beschreibung der gewählten Aktion an.
  - **Tastatursteuerung**: Navigation über Pfeiltasten und `Tab`/`Shift+Tab`, Schnelltasten (`H`, `N`, `E`, `R`, `A`, `S`, `B`), `Enter`/`Leertaste` zum Bestätigen und `Esc` zum Abbrechen.
  - **Automatisches Dark / Light Theme**: Liest `~/.config/kdeglobals` und passt Farbpalette sowie SVG-Icons an KDE Breeze Dark bzw. Light an.
- **Natives Rust-Binary und D-Bus-Integration**:
  - On-Demand-Aktivierung über den Session-Bus (`org.kubuntu.ShutdownDialog` auf `/Dialog`), direkte System- und Session-D-Bus-Aufrufe über `zbus` ohne Laufzeitabhängigkeit von Python oder PyQt.

---

## Kompilieren und Installation

### Voraussetzungen zum Selbstkompilieren

- Rust Toolchain (`cargo` / `rustc` >= 1.95)

### Bauen und Installieren aus dem Quellcode

```bash
git clone https://github.com/Sammywithh/kubuntu-altf4-shutdown.git
cd kubuntu-altf4-shutdown
./install.sh
```

Sofern `Cargo.toml` vorliegt, kompiliert `./install.sh` das Release-Binary reproduzierbar aus dem Quellcode (`cargo build --release --locked`). Wird `./install.sh` aus einem entpackten Linux-Release-Archiv ohne `Cargo.toml` ausgeführt, installiert es das enthaltene vorkompilierte Binary.

Ablauf der Installation:
1. Installiert das Binary nach `~/.local/bin/kubuntu-shutdown-dialog` (`0755`).
2. Erstellt die D-Bus-Service-Datei und den Desktop-Starter aus Vorlagen (`0644`).
3. Installiert das KWin-6-Skript für die `Alt + F4`-Erkennung.
4. Konfiguriert das KWin-Plugin und die Tastenkombination `Alt + F4`.

---

## Testen und CLI-Optionen

Direktes Kompilieren und Starten des Menüs aus dem Arbeitsverzeichnis:

```bash
./test_dialog.sh
```

Verfügbare Optionen von `kubuntu-shutdown-dialog`:
- `--native`: Öffnet direkt den offiziellen KDE Plasma 6 Abmeldedialog.
- `--daemon`: Startet den D-Bus-Hintergrunddienst ohne offenes Fenster.
- `--no-dbus`: Öffnet den Dialog ohne D-Bus-Registrierung.
- `-h`, `--help`: Zeigt die Hilfe an.
- `-V`, `--version`: Zeigt die Versionsnummer an.

---

## Automatisierter Linux-Build (GitHub Actions)

Über [`.github/workflows/build.yml`](.github/workflows/build.yml) werden bei Pushes und Pull Requests auf `main`/`master` sowie bei `v*`-Tags folgende Schritte mit minimalen Berechtigungen (`contents: read`) und auf Commit-SHAs gepinnten Actions ausgeführt:
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings` und `cargo test --locked`,
- Sicherheitsprüfung der Abhängigkeiten in `Cargo.lock` über `cargo audit`,
- Kompilierung des Release-Binaries (`cargo build --release --locked`) und Erstellung des Distributionsarchivs (`kubuntu-altf4-shutdown-linux-x86_64.tar.gz`) samt `SHA256SUMS.txt`,
- Veröffentlichung als GitHub Release in einem isolierten Release-Job ausschließlich bei `v*`-Tags.

---

## Deinstallation

```bash
./uninstall.sh
```

Stellt die Standard-`Alt + F4`-Tastenkombination von KWin wieder her und entfernt die installierten Benutzerdateien.

---

## Lizenz und Drittanbieter-Ressourcen

Dieses Projekt ist unter der **GNU General Public License v3.0 or later (`GPL-3.0-or-later`)** lizenziert.

Die in `src/icons.rs` als Fallback eingebetteten SVG-Grafiken stammen aus dem **KDE Breeze Icon Theme** (`breeze-icons`, KDE Community) und stehen unter der **GNU Lesser General Public License v3.0 or later (`LGPL-3.0-or-later`)**, die mit `GPL-3.0-or-later` kompatibel ist.
