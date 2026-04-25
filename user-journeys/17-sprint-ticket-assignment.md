# Journey 17: Sprint Ticket Assignment

Testet: `sprint ticket list`, `sprint ticket add`, `sprint ticket remove`, board-scoped sprint IDs, duplicate add, and remove error behavior.

## Cleanup

Diese Journey erzeugt ein Ticket, ein Agile Board und einen Sprint. Cleanup immer ausführen, auch wenn ein Zwischenschritt fehlschlägt:

```bash
ytd sprint ticket remove $SPRINT_ID $TICKET_ID || true
ytd ticket delete $TICKET_ID -y
ytd sprint delete $SPRINT_ID -y
ytd board delete $BOARD_ID -y
```

## Voraussetzungen

- `$PROJECT` ist vom User bestätigt.
- Der aktuelle Nutzer darf Tickets in `$PROJECT` erstellen und löschen.
- Der aktuelle Nutzer darf Agile Boards und Sprints für `$PROJECT` erstellen, ändern und löschen.
- Der aktuelle Nutzer darf Tickets zu Sprints hinzufügen und aus Sprints entfernen.
- Alle erzeugten Entities verwenden den Prefix `[YTD-TEST]`.

**Hinweis**: Board-, Sprint- und Sprint-Zuweisungsoperationen können zusätzliche YouTrack-Berechtigungen erfordern. Falls der aktuelle Nutzer diese Rechte nicht hat, den Permission-Fehler dokumentieren und die Journey als blockiert markieren.

## Testdaten anlegen

### 1. Board anlegen

```bash
BOARD_ID=$(ytd board create --name "[YTD-TEST] Sprint Ticket Assignment" --project $PROJECT --template scrum)
```

**Erwartung**: Exit-Code 0. Stdout enthält nur die Board-ID.

### 2. Sprint anlegen

```bash
SPRINT_ID=$(ytd sprint create --board $BOARD_ID --name "[YTD-TEST] Sprint Ticket Assignment")
```

**Erwartung**: Exit-Code 0. Stdout enthält eine sprint-id im Format `<board-id>:<sprint-id>`. Der Board-Anteil entspricht `$BOARD_ID`.

### 3. Ticket anlegen

```bash
TICKET_ID=$(ytd ticket create --project $PROJECT --json '{"summary":"[YTD-TEST] Sprint Ticket Assignment"}')
```

**Erwartung**: Exit-Code 0. Stdout enthält nur die lesbare Ticket-ID.

## Sprint-Tickets listen

### 4. Sprint-Tickets vor dem Hinzufügen als JSON listen

```bash
ytd sprint ticket list $SPRINT_ID --format json
```

**Erwartung**: Exit-Code 0. Ausgabe ist ein valides JSON-Array. Falls das Ticket bereits wegen Board-Default-Verhalten enthalten ist, dies dokumentieren; die weiteren Add/Remove-Schritte trotzdem ausführen.

## Ticket hinzufügen

### 5. Ticket zum Sprint hinzufügen

```bash
ytd sprint ticket add $SPRINT_ID $TICKET_ID
```

**Erwartung**: Exit-Code 0. Stdout enthält `$TICKET_ID`.

### 6. Ticket ist in Sprint-Ticket-Liste sichtbar

```bash
ytd sprint ticket list $SPRINT_ID --format json
```

**Erwartung**: Exit-Code 0. Ausgabe enthält ein Issue mit `id == "$TICKET_ID"`. Falls eine rohe YouTrack-Datenbank-ID vorhanden ist, steht sie in `ytId`, nicht in `idReadable`.

```bash
ytd sprint ticket list $SPRINT_ID
```

**Erwartung**: Exit-Code 0. Textausgabe verwendet das kompakte Ticket-Listenformat und enthält `$TICKET_ID` mit Summary sowie, falls vorhanden, wichtige Arbeitsfelder wie State, Assignee oder Priority.

### 7. Ticket-Sprints enthalten die exakte sprint-id

```bash
ytd ticket sprints $TICKET_ID --format json
```

**Erwartung**: Exit-Code 0. Ausgabe enthält einen Sprint mit `id == "$SPRINT_ID"`.

Falls weitere Sprints auf anderen Boards auftauchen, ist das erlaubt. Die Journey darf nur verlangen, dass der exakte `$SPRINT_ID` enthalten ist.

### 8. Duplicate Add ist erfolgreich

```bash
ytd sprint ticket add $SPRINT_ID $TICKET_ID
```

**Erwartung**: Exit-Code 0. Stdout enthält `$TICKET_ID`.

### 9. Duplicate Add erzeugt keinen doppelten Eintrag

```bash
ytd sprint ticket list $SPRINT_ID --format json
```

**Erwartung**: Exit-Code 0. `id == "$TICKET_ID"` kommt höchstens einmal in der Ausgabe vor. `idReadable` wird nicht ausgegeben.

## Ticket entfernen

### 10. Ticket aus Sprint entfernen

```bash
ytd sprint ticket remove $SPRINT_ID $TICKET_ID
```

**Erwartung**: Exit-Code 0. Stdout enthält `$TICKET_ID`.

### 11. Ticket ist nicht mehr in Sprint-Ticket-Liste sichtbar

```bash
ytd sprint ticket list $SPRINT_ID --format json
```

**Erwartung**: Exit-Code 0. Ausgabe enthält kein Issue mit `id == "$TICKET_ID"`.

### 12. Ticket-Sprints enthalten die exakte sprint-id nicht mehr

```bash
ytd ticket sprints $TICKET_ID --format json
```

**Erwartung**: Exit-Code 0. Ausgabe enthält keinen Sprint mit `id == "$SPRINT_ID"`.

Falls weitere Sprints auf anderen Boards auftauchen, ist das erlaubt.

### 13. Erneutes Remove schlägt fehl

```bash
ytd sprint ticket remove $SPRINT_ID $TICKET_ID
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung kommt von YouTrack und beschreibt typischerweise, dass die Entity nicht gefunden wurde.

## Validierung ungültiger Eingaben

### 14. Raw Sprint-ID wird abgelehnt

```bash
RAW_SPRINT_ID=${SPRINT_ID#*:}
ytd sprint ticket list $RAW_SPRINT_ID
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass die öffentliche sprint-id im Format `<board-id>:<sprint-id>` verwendet werden muss.

### 15. `current` wird als sprint-id abgelehnt

```bash
ytd sprint ticket list $BOARD_ID:current
```

**Erwartung**: Exit-Code ungleich 0. Fehlermeldung erklärt, dass `current` keine sprint-id ist und `ytd sprint current` verwendet werden soll.

## Cleanup

### 16. Ticket löschen

```bash
ytd ticket delete $TICKET_ID -y
```

**Erwartung**: Exit-Code 0.

### 17. Sprint löschen

```bash
ytd sprint delete $SPRINT_ID -y
```

**Erwartung**: Exit-Code 0.

### 18. Board löschen

```bash
ytd board delete $BOARD_ID -y
```

**Erwartung**: Exit-Code 0.

### 19. Cleanup prüfen

```bash
ytd ticket get $TICKET_ID
ytd sprint get $SPRINT_ID
ytd board get $BOARD_ID
```

**Erwartung**: Alle drei Commands schlagen fehl oder melden, dass die Entity nicht gefunden wurde.
