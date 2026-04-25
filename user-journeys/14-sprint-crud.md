# Journey 14: Sprint CRUD

Testet: `sprint create`, `sprint get`, `sprint update`, `sprint list`, `sprint current`, `sprint delete`

## Cleanup

Diese Journey erzeugt ein Agile Board und einen Sprint. Cleanup immer ausführen, auch wenn ein Zwischenschritt fehlschlägt:

```
ytd sprint delete $SPRINT_ID -y
ytd board delete $BOARD_ID -y
```

Falls `$SPRINT_ID` oder `$BOARD_ID` nicht gesetzt wurde, den jeweiligen Cleanup überspringen.

**Hinweis**: Board- und Sprint-Verwaltung können zusätzliche YouTrack-Berechtigungen erfordern. Falls der aktuelle Nutzer keine Boards oder Sprints anlegen darf, den Permission-Fehler dokumentieren und die Journey als blockiert markieren.

## Voraussetzungen

- `$PROJECT` ist das vom User bestätigte Zielprojekt-Kürzel.
- Der aktuelle Nutzer darf Agile Boards und Sprints für `$PROJECT` erstellen, ändern und löschen.
- Alle erzeugten Boards und Sprints verwenden den Prefix `[YTD-TEST]`.

## Testboard anlegen

### 1. Board mit Sprint-Support anlegen

```
BOARD_ID=$(ytd board create --name "[YTD-TEST] Sprint CRUD Board" --project $PROJECT --template scrum)
```

**Erwartung**: Exit-Code 0. Stdout enthält nur die Board-ID.

**Merke** die Ausgabe als `$BOARD_ID`.

## Sprint anlegen

### 2. Sprint mit Pflicht-Flags anlegen

```
SPRINT_ID=$(ytd sprint create --board $BOARD_ID --name "[YTD-TEST] Sprint CRUD")
```

**Erwartung**: Exit-Code 0. Stdout enthält nur die sprint-id im Format `<board-id>:<sprint-id>`, z.B. `$BOARD_ID:113-6`.

**Merke** die Ausgabe als `$SPRINT_ID`.

### 3. Sprint abrufen

```
ytd sprint get $SPRINT_ID
```

**Erwartung**: Exit-Code 0. Ausgabe enthält den Namen `[YTD-TEST] Sprint CRUD`.

### 4. Sprint als JSON abrufen

```
ytd sprint get $SPRINT_ID --format raw
```

**Erwartung**: Valides JSON mit mindestens `id`, `ytId`, `boardId`, `boardName` und `name`. `id` entspricht `$SPRINT_ID`, `boardId` entspricht `$BOARD_ID`, `boardName` ist `[YTD-TEST] Sprint CRUD Board`, `name` ist `[YTD-TEST] Sprint CRUD`.

## Sprint aktualisieren

### 5. Sprint per `--name` aktualisieren

```
ytd sprint update $SPRINT_ID --name "[YTD-TEST] Sprint CRUD Renamed"
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$SPRINT_ID`.

### 6. Rename prüfen

```
ytd sprint get $SPRINT_ID --format raw
```

**Erwartung**: Valides JSON. `name` ist `[YTD-TEST] Sprint CRUD Renamed`.

### 7. Sprint per JSON aktualisieren

```
ytd sprint update $SPRINT_ID --json '{"goal":"[YTD-TEST] sprint goal"}'
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$SPRINT_ID`.

### 8. JSON-Update prüfen

```
ytd sprint get $SPRINT_ID --format raw
```

**Erwartung**: Valides JSON. Falls YouTrack das Feld im Response zurückgibt, ist `goal` `[YTD-TEST] sprint goal`.

### 9. Flag gewinnt gegen JSON

```
ytd sprint update $SPRINT_ID --name "[YTD-TEST] Sprint CRUD Flag Wins" --json '{"name":"[YTD-TEST] Sprint CRUD JSON Name"}'
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$SPRINT_ID`.

```
ytd sprint get $SPRINT_ID --format raw
```

**Erwartung**: `name` ist `[YTD-TEST] Sprint CRUD Flag Wins`, nicht der JSON-Name.

## Sprint suchen und current prüfen

### 10. Sprint in Board-Liste finden

```
ytd sprint list --board $BOARD_ID --format raw
```

**Erwartung**: Valides JSON-Array. Ein Eintrag hat `id == $SPRINT_ID`, `ytId` ist der rohe YouTrack-Sprint-ID-Anteil, `boardId == $BOARD_ID` und `boardName == "[YTD-TEST] Sprint CRUD Board"`.

### 10b. Sprint in globaler Sprint-Liste finden

```
ytd sprint list --format raw
```

**Erwartung**: Valides JSON-Array. Ein Eintrag hat `id == $SPRINT_ID`, `ytId` ist der rohe YouTrack-Sprint-ID-Anteil, `boardId == $BOARD_ID` und `boardName == "[YTD-TEST] Sprint CRUD Board"`. Weitere Sprints aus anderen Boards sind erlaubt.

### 11. Current Sprint für Board prüfen

```
ytd sprint current --board $BOARD_ID --format raw
```

**Erwartung**: Entweder Exit-Code 0 mit validem JSON-Objekt, das `id`, `ytId`, `boardId`, `boardName`, `projects` und `name` enthält, oder Exit-Code ungleich 0 mit klarer Meldung, dass das Board keinen current Sprint hat.

## Validierungsfehler

### 12. Create ohne Namen schlägt fehl

```
ytd sprint create --board $BOARD_ID
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass `--name` oder JSON `name` erforderlich ist.

### 13. Create ohne Board schlägt fehl

```
ytd sprint create --name "[YTD-TEST] Missing Board"
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass `--board` erforderlich ist.

### 14. Update ohne Felder schlägt fehl

```
ytd sprint update $SPRINT_ID
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass mindestens `--name` oder JSON erforderlich ist.

### 15. Update mit Nicht-Objekt-JSON schlägt fehl

```
ytd sprint update $SPRINT_ID --json '[]'
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass ein JSON-Objekt erforderlich ist.

### 16. `current` ist keine sprint-id

```
ytd sprint get $BOARD_ID:current
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass `current` keine sprint-id ist.

## Sprint löschen

### 17. Sprint löschen

```
ytd sprint delete $SPRINT_ID -y
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$SPRINT_ID`.

### 18. Löschen prüfen

```
ytd sprint get $SPRINT_ID
```

**Erwartung**: Exit-Code ungleich 0 oder YouTrack-Fehler, dass der Sprint nicht gefunden wurde.

**Setze** `$SPRINT_ID` danach zurück, damit der allgemeine Cleanup nicht erneut löscht.

## Board löschen

### 19. Testboard löschen

```
ytd board delete $BOARD_ID -y
```

**Erwartung**: Exit-Code 0. Stdout enthält nur `$BOARD_ID`.

**Setze** `$BOARD_ID` danach zurück, damit der allgemeine Cleanup nicht erneut löscht.
