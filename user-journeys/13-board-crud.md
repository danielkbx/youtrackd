# Journey 13: Board CRUD

Testet: `board create`, `board update`, `board get`, `board list`, `board delete`

## Cleanup

Diese Journey erzeugt ein Agile Board. Cleanup immer ausführen, auch wenn ein Zwischenschritt fehlschlägt:

```
ytd board delete $BOARD_ID -y
```

Falls `$BOARD_ID` nicht gesetzt wurde, ist kein Cleanup nötig.

**Hinweis**: Board-Erstellung und -Löschung können zusätzliche YouTrack-Berechtigungen erfordern. Falls der aktuelle Nutzer keine Boards anlegen darf, den Permission-Fehler dokumentieren und die Journey als blockiert markieren.

## Voraussetzungen

- `$PROJECT` ist das vom User bestätigte Zielprojekt-Kürzel.
- Der aktuelle Nutzer darf Agile Boards für `$PROJECT` erstellen, ändern und löschen.
- Alle erzeugten Boards verwenden den Prefix `[YTD-TEST]`.

## Board anlegen

### 1. Board mit Pflicht-Flags anlegen

```
BOARD_ID=$(ytd board create --name "[YTD-TEST] Board CRUD" --project $PROJECT --template scrum)
```

**Erwartung**: Exit-Code 0. Stdout enthält nur die Board-ID.

**Merke** die Ausgabe als `$BOARD_ID`.

### 2. Board abrufen

```
ytd board get $BOARD_ID
```

**Erwartung**: Exit-Code 0. Ausgabe enthält den Namen `[YTD-TEST] Board CRUD` und das Projekt `$PROJECT`.

### 3. Board als JSON abrufen

```
ytd board get $BOARD_ID --format json
```

**Erwartung**: Valides JSON mit mindestens `id`, `name`, `projects` und `sprints`. `id` entspricht `$BOARD_ID`, `name` ist `[YTD-TEST] Board CRUD`.

## Board aktualisieren

### 4. Board per `--name` aktualisieren

```
ytd board update $BOARD_ID --name "[YTD-TEST] Board CRUD Renamed"
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$BOARD_ID`.

### 5. Rename prüfen

```
ytd board get $BOARD_ID --format json
```

**Erwartung**: Valides JSON. `name` ist `[YTD-TEST] Board CRUD Renamed`.

### 6. Board per JSON aktualisieren

```
ytd board update $BOARD_ID --json '{"orphansAtTheTop":true}'
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$BOARD_ID`.

### 7. JSON-Update prüfen

```
ytd board get $BOARD_ID --format json
```

**Erwartung**: Valides JSON. Falls YouTrack das Feld im Response zurückgibt, ist `orphansAtTheTop` auf `true` gesetzt.

### 8. Board per stdin-JSON aktualisieren

```
printf '%s\n' '{"hideOrphansSwimlane":false}' | ytd board update $BOARD_ID
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$BOARD_ID`. Stdin-JSON wird akzeptiert.

### 9. Flag gewinnt gegen JSON

```
ytd board update $BOARD_ID --name "[YTD-TEST] Board CRUD Flag Wins" --json '{"name":"[YTD-TEST] Board CRUD JSON Name"}'
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$BOARD_ID`.

```
ytd board get $BOARD_ID --format json
```

**Erwartung**: `name` ist `[YTD-TEST] Board CRUD Flag Wins`, nicht der JSON-Name.

## Board suchen und filtern

### 10. Board in Projektliste finden

```
ytd board list --project $PROJECT --format json
```

**Erwartung**: Valides JSON-Array. Ein Eintrag hat `id == $BOARD_ID`.

## Validierungsfehler

### 11. Create ohne Namen schlägt fehl

```
ytd board create --project $PROJECT --template scrum
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass `--name` oder JSON `name` erforderlich ist.

### 12. Create ohne Projekt schlägt fehl

```
ytd board create --name "[YTD-TEST] Missing Project" --template scrum
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass `--project` oder JSON `projects` erforderlich ist.

### 13. Ungültiges Template schlägt lokal fehl

```
ytd board create --name "[YTD-TEST] Invalid Template" --project $PROJECT --template invalid-template
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung nennt die erlaubten Template-Werte. Es wird kein Board erstellt.

### 14. Update ohne Felder schlägt fehl

```
ytd board update $BOARD_ID
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass mindestens `--name` oder JSON erforderlich ist.

### 15. Update mit Nicht-Objekt-JSON schlägt fehl

```
ytd board update $BOARD_ID --json '[]'
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass ein JSON-Objekt erforderlich ist.

## Board löschen

### 16. Board löschen

```
ytd board delete $BOARD_ID -y
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$BOARD_ID`.

### 17. Löschen prüfen

```
ytd board get $BOARD_ID
```

**Erwartung**: Exit-Code ungleich 0 oder YouTrack-Fehler, dass das Board nicht gefunden wurde.

**Setze** `$BOARD_ID` danach zurück, damit der allgemeine Cleanup nicht erneut löscht.
