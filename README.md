# folder-sync

Ein Tool, um mehrere externe Backup-Festplatten ("Clone" desselben Datensatzes)
im Sync zu halten. Läuft als einzelnes Rust-Binary mit eingebetteter
Browser-UI unter `http://localhost:13322`.

## Konzept

- Externe Platten werden unter `/media/{user}/{PartLabel}` gemountet, wobei
  `PartLabel = {Name}_{Nummer}{Clone}` ist (z.&nbsp;B. `Daten_1a`, `Daten_1b`,
  `Videos_2a`, `Videos_2c`).
- Alle Platten mit gleichem `{Name}_{Nummer}` gelten als Clone derselben
  logischen Sicherung.
- Beim Start wird der konfigurierte Scan-Root-Pfad (Standard `/media/$USER`)
  nach passenden Ordnern durchsucht.
- Pro Clone-Gruppe wird ein vereinigter Verzeichnisbaum angezeigt, der zeigt,
  in welchen Clonen eine Datei vorhanden, abweichend oder fehlend ist.
- Sync-/Lösch-Aktionen werden in eine Batch-Warteliste eingereiht und erst auf
  expliziten Klick ausgeführt — mit Live-Fortschritt, Abbruch- und
  Fortsetzen-Möglichkeit.
- Ein Read-Only-Modus erlaubt reine Inspektion ohne Schreibzugriff.

## Schnellstart im Devcontainer

Der Devcontainer (`.devcontainer/devcontainer.json`) bringt Rust und Node
mit. Nach dem ersten Aufbau:

```sh
# Test-Laufwerke erzeugen (Standardpfad: ./fixtures, per Env-Var vorkonfiguriert)
just fixtures

# Backend + Vite-Dev-Server (mit Hot Reload) gemeinsam starten
just dev
```

`just dev` startet den axum-Server und den Vite-Dev-Server zusammen; der
Vite-Server proxyt `/api` und die WebSocket-Verbindung `/api/events` zum
Backend auf Port 13322. Die App ist dann unter `http://localhost:5173`
erreichbar (mit Hot Reload), das reine Backend zusätzlich unter
`http://localhost:13322`.

Der Container setzt `FOLDER_SYNC_SCAN_ROOT` standardmäßig auf
`./fixtures`, sodass `just dev` ohne weitere Flags gegen die generierten
Test-Laufwerke läuft.

## Manuelles Setup (ohne Devcontainer)

Voraussetzungen: Rust (stable) und Node.js (LTS), `just` (`cargo install
just`).

```sh
just fixtures          # Test-Laufwerke erzeugen
just dev                # Dev-Loop mit Hot Reload
# oder:
just build              # Single-Binary bauen (target/release/folder-sync)
./target/release/folder-sync --scan-root ./fixtures
```

## CLI-Flags

| Flag | Env-Variable | Beschreibung |
|---|---|---|
| `--scan-root <PFAD>` | `FOLDER_SYNC_SCAN_ROOT` | Root-Verzeichnis für die Laufwerkssuche. Überschreibt und persistiert in die Config-Datei. |
| `--read-only` | – | Nur Inspektion; Sync/Löschen-Ausführung ist für die gesamte Session deaktiviert. |
| `--port <PORT>` | – | Port für UI/API (Standard `13322`). |
| `--config-path <PFAD>` | `FOLDER_SYNC_CONFIG_PATH` | Alternativer Pfad zur Config-Datei (Standard `~/.config/folder-sync/config.toml`). |

Der Scan-Root lässt sich zusätzlich jederzeit über den Einstellungsdialog in
der UI ändern (wird ebenfalls persistiert).

## Vergleichsmodi

- **Standard**: Vergleich über Dateiname + Größe.
- **Erweitert** (per Toggle in der Baumansicht): zusätzlich Inhaltsvergleich
  über einen schnellen XXH3-Hash (Streaming, mit In-Memory-Cache pro
  Pfad/Größe/Änderungszeit).

## Testen

```sh
cargo test --workspace          # Unit-Tests (Rust)
just fixtures                    # generiert realistische Test-Laufwerke
                                  # (mehrere Clone, fehlende/abweichende/
                                  # identische Dateien, Groß-/Kleinschreibung,
                                  # verschachtelte Ordner)
```

## Projektstruktur

```
crates/core/     Kernlogik ohne Web-Abhängigkeiten: Laufwerkserkennung,
                 Vergleichs-Engine, Merge-Baum, Hashing, Batch-Planung
                 und -Ausführung
crates/server/   axum-Server: REST/WebSocket-API, bettet das gebaute
                 Frontend per rust-embed ein (Single-Binary)
frontend/        Vue 3 + TypeScript SPA (Vite)
xtask/           Testdaten-Generator (`cargo run -p xtask -- gen-fixtures`)
```

Architekturentscheidungen und Meilensteinplan: siehe Commit-Historie dieses
Branches.
