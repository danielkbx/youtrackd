# Journey 7: Custom Fields

Testet: `ticket fields`, `ticket status`, `ticket set`

## Schritte

### 1. Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Custom Fields Test"}'
```

**Merke** die ID als `$TICKET_ID`.

### 2. Aktuelle Feldwerte anzeigen

```
ytd ticket fields $TICKET_ID
```

**Erwartung**: Liste der Custom Fields mit Namen und aktuellen Werten. Typische Felder: State, Priority, Type, Assignee. Exit-Code 0.

### 3. Felder als JSON

```
ytd ticket fields $TICKET_ID --format json
```

**Erwartung**: Valides JSON-Array. Jedes Feld hat `name` und `value`.

### 4. Priority setzen

```
ytd ticket set $TICKET_ID Priority Major
```

**Erwartung**: Exit-Code 0.

### 5. Priority verifizieren

```
ytd ticket fields $TICKET_ID
```

**Erwartung**: Priority ist `Major`.

### 6. Type setzen (falls vorhanden)

```
ytd ticket set $TICKET_ID Type Task
```

**Erwartung**: Exit-Code 0 (oder Fehler, falls das Feld nicht existiert — dann überspringen).

### 7. Verfügbare Statuswerte anzeigen

```
ytd ticket status $TICKET_ID
```

**Erwartung**: Exit-Code 0. Ausgabe enthält die für das Projekt gültigen Statuswerte; der aktuelle Status ist mit `*` markiert.

### 8. State auf Done setzen

```
ytd ticket status $TICKET_ID Done
```

**Erwartung**: Exit-Code 0. Falls `Done` kein gültiger Statuswert im Projekt ist, muss der Fehler die gültigen Werte nennen.

### 9. Endstatus verifizieren

```
ytd ticket get $TICKET_ID --format json
```

**Erwartung**: Custom Fields enthalten State=Done und Priority=Major.

## Cleanup

```
ytd ticket delete $TICKET_ID -y
```
