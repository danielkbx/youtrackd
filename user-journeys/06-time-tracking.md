# Journey 6: Time Tracking

Testet: `ticket log`, `ticket worklog`

## Schritte

### 1. Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Time Tracking Test"}'
```

**Merke** die ID als `$TICKET_ID`.

### 2. Zeit buchen (einfach)

```
ytd ticket log $TICKET_ID 30m "[YTD-TEST] Erste Zeitbuchung"
```

**Erwartung**: Exit-Code 0.

### 3. Zeit buchen (mit Datum)

```
ytd ticket log $TICKET_ID 1h "[YTD-TEST] Zweite Zeitbuchung" --date 2025-01-15
```

**Erwartung**: Exit-Code 0.

### 4. Work Items anzeigen

```
ytd ticket worklog $TICKET_ID
```

**Erwartung**: Zwei Einträge sichtbar:
- 30 Minuten mit Text `Erste Zeitbuchung`
- 1 Stunde mit Text `Zweite Zeitbuchung` und Datum 2025-01-15

### 5. Work Items als JSON

```
ytd ticket worklog $TICKET_ID --format raw
```

**Erwartung**: Valides JSON-Array mit 2 Einträgen. Jeder hat `duration`, `text`, `date`.

## Cleanup

```
ytd ticket delete $TICKET_ID -y
```
