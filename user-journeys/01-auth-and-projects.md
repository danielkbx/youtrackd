# Journey 1: Auth & Projekte

Testet: `whoami`, `project list`, `project get`, `--format raw`, `--no-meta`

## Cleanup

Kein Cleanup nötig — diese Journey erzeugt keine Entities.

## Schritte

### 1. Aktuellen User prüfen

```
ytd whoami
```

**Erwartung**: Gibt User-Daten aus (login, fullName, email). Exit-Code 0.

### 2. Projekte auflisten

```
ytd project list
```

**Erwartung**: Mindestens ein Projekt wird angezeigt. Das vom User gewählte Testprojekt (`$PROJECT`) ist enthalten.

### 3. Projekt-Details abrufen

```
ytd project get $PROJECT
```

**Erwartung**: Zeigt name, shortName, description. `shortName` entspricht `$PROJECT`.

### 4. JSON-Output testen

```
ytd project get $PROJECT --format raw
```

**Erwartung**: Valides JSON. Enthält Felder `id`, `name`, `shortName`.

### 5. --no-meta testen

```
ytd project list --no-meta
```

**Erwartung**: Keine `id`-Felder in der Ausgabe.

### 6. Fehlerfall: Unbekanntes Projekt

```
ytd project get NONEXISTENT_PROJECT_XYZ
```

**Erwartung**: Exit-Code != 0. Fehlermeldung auf stderr.
