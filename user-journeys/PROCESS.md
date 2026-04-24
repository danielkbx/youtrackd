# User Journey and Guard Test Process

## Voraussetzungen

- `ytd` ist gebaut und im PATH (oder als `./target/release/ytd` erreichbar)
- Der Agent ist eingeloggt (`ytd whoami` funktioniert)

## Ablauf

### 1. Projekt-Auswahl

**Bevor irgendeine Journey ausgeführt wird**, muss der Agent:

1. `ytd project list` ausführen und dem User die Projekte anzeigen
2. Den User **explizit fragen**, in welchem Projekt die Tests laufen sollen
3. Die **Bestätigung des Users abwarten** — niemals selbstständig ein Projekt wählen
4. Das bestätigte Projekt-Kürzel (z.B. `TESTPROJ`) für alle Journeys verwenden

Für Journeys mit Visibility-Defaults muss der Agent zusätzlich vor dem Start einen gültigen Visibility-Gruppennamen erfragen, den der aktuelle Nutzer auf Tickets und Artikeln setzen und wieder entfernen darf. Diese Bestätigung ebenfalls abwarten.

### 2. Journey- und Guard-Ausführung

Die Markdown-Dateien in diesem Verzeichnis fallen in zwei Gruppen:

- **User Journeys**: fachliche End-to-End-Abläufe aus Nutzersicht
- **Technical Guards**: technische Absicherungen für API-Kompatibilität, dokumentierte Request-Shapes und risikobehaftete Integrationspunkte

Der Agent:

1. Liest die entsprechende Datei
2. Führt die beschriebenen Schritte sequenziell aus
3. Prüft nach jedem Schritt, ob das erwartete Ergebnis eingetreten ist
4. Führt am Ende den **Cleanup** durch (in der Datei beschrieben)

### 3. Naming-Konvention für Test-Entities

Alle vom Test erzeugten Entities verwenden dieses Muster:

- **Prefix**: `[YTD-TEST]`
- **Ticket-Summaries**: `[YTD-TEST] <Beschreibung>`
- **Artikel-Summaries**: `[YTD-TEST] <Beschreibung>`
- **Kommentare**: `[YTD-TEST] <Text>`
- **Tags**: Nur vorhandene Tags verwenden, keine neuen anlegen

So sind Test-Entities sofort erkennbar und können bei Bedarf manuell aufgeräumt werden.

### 4. Cleanup-Regeln

| Entity-Typ | Cleanup-Aktion |
|---|---|
| Ticket | `ytd ticket delete <id> -y` |
| Artikel | `ytd article delete <id> -y` |
| Tags | Wieder entfernen, wenn im Test hinzugefügt (vor dem Delete) |
| Links | Werden mit dem Ticket gelöscht |
| Temp-Config-Dateien | Mit `rm -f` löschen |
| Temporäre Env Vars | Mit `unset` entfernen |

### 5. Fehlerbehandlung

- Schlägt ein Schritt fehl, **trotzdem den Cleanup ausführen**
- Fehler dokumentieren: welcher Schritt, welcher Command, welche Ausgabe
- Nach dem Cleanup dem User eine Zusammenfassung geben: bestanden / fehlgeschlagen

### 6. Reihenfolge

Die User Journeys sind unabhängig voneinander und können in beliebiger Reihenfolge ausgeführt werden. Technical Guards sind ergänzende technische Prüfungen und ersetzen keine fachlichen Journeys.

Empfohlene Reihenfolge:

1. `01-auth-and-projects.md` — Grundlagen, kein Cleanup nötig
2. `02-ticket-lifecycle.md` — Ticket CRUD
3. `03-article-lifecycle.md` — Artikel CRUD
4. `04-tags-and-links.md` — Tags und Verlinkungen
5. `05-attachments.md` — Datei-Upload
6. `06-time-tracking.md` — Arbeitszeit
7. `07-custom-fields.md` — Feldwerte setzen
8. `08-search-and-boards.md` — Saved Searches, Boards
9. `09-history.md` — Activity-Log
10. `10-api-compat-guard.md` — technische Absicherung gegen API-Drift und dokumentationskritische Integrationspunkte
11. `11-visibility-defaults.md` — Visibility-Defaults, `YTD_CONFIG`-Isolation und Override-Reihenfolge
12. `12-comments.md` — globale Kommentar-Kommandos und kodierte Kommentar-IDs
