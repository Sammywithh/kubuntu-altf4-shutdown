# Kubuntu 26.04 Alt+F4 Smart Shutdown Dialog

Ein modernes, elegantes und natives **Herunterfahren- und Abmelde-Menü für Kubuntu (KDE Plasma 6)** im Stil von Kubuntu 26.04 / KDE Breeze, gesteuert über die beliebte Tastenkombination **`Alt + F4`**.

---

## ✨ Highlights & Funktionen

- **Intelligente `Alt + F4`-Erkennung**:
  - Wenn ein normales Anwendungsfenster aktiv ist: **`Alt + F4` schließt das Fenster** (Standardverhalten).
  - Wenn der Desktop aktiv ist (oder alle Fenster minimiert/geschlossen sind): **`Alt + F4` öffnet das Kubuntu Herunterfahren-Menü**.
- **Authentisches Kubuntu 26.04 / KDE Breeze Design**:
  - Große, übersichtliche Aktionskacheln mit originalen KDE Breeze System-Icons:
    - ⏻ **Herunterfahren** (`H` / Enter)
    - ⟳ **Neu starten** (`N`)
    - ⏸ **Energie sparen / Standby** (`E`)
    - ❄ **Ruhezustand** (`R`)
    - ↪ **Abmelden** (`A`)
    - 🔒 **Sperren** (`S`)
    - 👤 **Benutzer wechseln** (`B`)
  - **Dynamischer Informationstext**: Zeigt bei Hover oder Tastaturfokus eine genaue Beschreibung der gewählten Aktion an.
  - **Volle Tastatursteuerung**: Navigation mit Pfeiltasten, Schnelltasten (H, N, E, R, A, S, B), `Enter` zum Bestätigen und `Esc` zum Abbrechen.
  - **Automatisches Dark / Light Theme**: Passt sich automatisch dem aktuellen Farbschema von Kubuntu an.
- **Ressourcenschonend (Zero CPU im Hintergrund)**:
  - Läuft nicht permanent im Hintergrund, sondern wird über KWin / D-Bus bei Bedarf blitzschnell aufgerufen.

---

## 🚀 Installation & Aktivierung

### 1. Installation starten
Führe im Projektverzeichnis einfach das Installationsskript aus:

```bash
cd /home/sekonya/KI-Agentic-Projects/kubuntu-altf4-shutdown
./install.sh
```

Das Skript:
1. Richtet die Anwendung in `~/.local/bin/kubuntu-shutdown-dialog` ein.
2. Registriert den D-Bus Dienst für On-Demand Aktivierung.
3. Installiert das KWin 6 Skript für intelligentes Alt+F4.
4. Schaltet die Tastenkombination `Alt+F4` live scharf.

---

## 🧪 Schneller Test

Du kannst das Menü jederzeit direkt im Terminal testen:

```bash
./test_dialog.sh
```

Oder das offizielle KDE Plasma 6 Vollbild-Abmeldemenü testen:

```bash
./test_dialog.sh --native
```

---

## 🗑️ Deinstallation

Falls du jederzeit zum Standard-Verhalten von KDE Plasma zurückkehren möchtest:

```bash
./uninstall.sh
```
Dadurch wird die originale `Alt+F4` Verknüpfung von KWin wiederhergestellt und alle Dateien sauber entfernt.
