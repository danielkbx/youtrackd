# Journey 16: Ticket Sprints

Testet: `ticket sprints` und Wiederverwendung der zurückgegebenen sprint-id mit `sprint get`

## Cleanup

Diese Journey erzeugt keine Daten. Kein Cleanup nötig.

## Voraussetzungen

- `$TICKET_ID` ist ein vom User bestätigtes Ticket.
- Das Ticket ist mindestens einem Sprint zugeordnet.
- Falls kein geeignetes Ticket existiert, den User bitten, ein Ticket in der YouTrack-Weboberfläche einem Sprint zuzuordnen. Ticket-zu-Sprint-Zuordnung per `ytd` ist in dieser Journey bewusst nicht Teil des Scopes.

## Ticket-Sprints lesen

### 1. Sprints eines Tickets als Text ausgeben

```
ytd ticket sprints $TICKET_ID
```

**Erwartung**: Exit-Code 0. Ausgabe enthält die Sprints, denen das Ticket zugeordnet ist.

### 2. Sprints eines Tickets als JSON ausgeben

```
ytd ticket sprints $TICKET_ID --format raw
```

**Erwartung**: Exit-Code 0. Ausgabe ist ein valides JSON-Array mit mindestens einem Eintrag.

Jeder Eintrag muss mindestens enthalten:

- `id`: öffentliche sprint-id im Format `<board-id>:<sprint-id>`
- `ytId`: rohe YouTrack-Sprint-ID
- `boardId`: Board-ID
- `agile.id`: Board-ID
- `agile.name`: lesbarer Board-Name, falls YouTrack ihn liefert
- `name`: Sprint-Name, falls YouTrack ihn liefert

## Wiederverwendbare sprint-id prüfen

### 3. Erste sprint-id übernehmen

Aus Schritt 2 den ersten Wert aus `id` als `$SPRINT_ID` merken.

### 4. Sprint abrufen

```
ytd sprint get $SPRINT_ID --format raw
```

**Erwartung**: Exit-Code 0. Ausgabe ist ein valides JSON-Objekt mit `id == $SPRINT_ID`.
