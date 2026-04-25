# Journey 15: Current Sprints

Testet: `sprint current` ohne `--board`, Ausgabe mit Board-/Projekt-Kontext und wiederverwendbarer sprint-id

## Cleanup

Diese Journey erzeugt keine Daten. Kein Cleanup nötig.

## Voraussetzungen

- Der aktuelle Nutzer darf Agile Boards lesen.
- Falls keine Boards mit current Sprint existieren, darf die Ausgabe ein leeres JSON-Array sein.

## Current Sprints listen

### 1. Alle current Sprints als Text ausgeben

```
ytd sprint current
```

**Erwartung**: Exit-Code 0. Ausgabe enthält alle Boards, die einen current Sprint haben. Boards ohne current Sprint werden ausgelassen.

### 2. Alle current Sprints als JSON ausgeben

```
ytd sprint current --format raw
```

**Erwartung**: Exit-Code 0. Ausgabe ist ein valides JSON-Array.

Falls das Array Einträge enthält, muss jeder Eintrag mindestens enthalten:

- `id`: öffentliche sprint-id im Format `<board-id>:<sprint-id>`
- `ytId`: rohe YouTrack-Sprint-ID
- `boardId`: Board-ID
- `boardName`: lesbarer Board-Name, falls YouTrack ihn liefert
- `projects`: Array mit Projekt-IDs, Kürzeln und lesbaren Projektnamen
- `name`: Sprint-Name, falls YouTrack ihn liefert

## Wiederverwendbare sprint-id prüfen

### 3. Erste sprint-id übernehmen

Falls Schritt 2 mindestens einen Eintrag liefert, den ersten Wert aus `id` als `$SPRINT_ID` merken.

### 4. Sprint mit sprint-id abrufen

```
ytd sprint get $SPRINT_ID --format raw
```

**Erwartung**: Exit-Code 0. Ausgabe ist ein valides JSON-Objekt mit `id == $SPRINT_ID`.

## Kein `current` als sprint-id

### 5. `current` als ID-Anteil schlägt fehl

```
ytd sprint get 108-4:current
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass `current` keine sprint-id ist.
