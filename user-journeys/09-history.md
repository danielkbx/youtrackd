# Journey 9: Activity History

Testet: `ticket history`

Diese Journey erzeugt ein Ticket, nimmt mehrere Änderungen vor und prüft dann, ob die History alle Änderungen korrekt wiedergibt.

## Schritte

### 1. Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] History Test"}'
```

**Merke** die ID als `$TICKET_ID`.

### 2. Änderungen vornehmen

Summary updaten:

```
ytd ticket update $TICKET_ID --json '{"summary": "[YTD-TEST] History Test (updated)"}'
```

Kommentar hinzufügen:

```
ytd ticket comment $TICKET_ID "[YTD-TEST] History-Kommentar"
```

Priority setzen:

```
ytd ticket set $TICKET_ID Priority Critical
```

### 3. History abrufen

```
ytd ticket history $TICKET_ID
```

(Alle Activity-Kategorien werden standardmäßig angezeigt.)

**Erwartung**: Chronologisches Activity-Log mit mindestens:
- Erstellung des Tickets
- Summary-Änderung
- Kommentar
- Priority-Änderung auf Critical

Jeder Eintrag enthält Timestamp und Author.

### 4. History als JSON

```
ytd ticket history $TICKET_ID --format raw
```

**Erwartung**: Valides JSON-Array. Jeder Eintrag hat `timestamp`, `author`, `field` oder Aktivitätstyp.

Wenn Kommentar-Aktivitäten Kommentarobjekte in `added`, `removed` oder `target` enthalten, dann sind deren `id`-Felder kodierte IDs, die mit `$TICKET_ID:` beginnen. Jede gefundene Kommentar-ID funktioniert mit `ytd comment get <id>`, solange der Kommentar nicht gelöscht wurde.

## Cleanup

```
ytd ticket delete $TICKET_ID -y
```
