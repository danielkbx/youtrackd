# Journey 2: Ticket-Lifecycle

Testet: `ticket create`, `ticket get`, `ticket update`, `ticket comment`, `ticket search`, `ticket list`

## Schritte

### 1. Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Ticket Lifecycle Test", "description": "Automatisch erzeugtes Test-Ticket. Kann ignoriert oder gelöscht werden."}'
```

**Erwartung**: Gibt nur die Ticket-ID aus (z.B. `PROJ-123`). Exit-Code 0.

**Merke** die ID als `$TICKET_ID`.

### 2. Ticket abrufen

```
ytd ticket get $TICKET_ID
```

**Erwartung**: Summary enthält `[YTD-TEST] Ticket Lifecycle Test`. Description ist vorhanden.

### 3. Ticket abrufen (JSON)

```
ytd ticket get $TICKET_ID --format raw
```

**Erwartung**: Valides JSON. `idReadable` entspricht `$TICKET_ID`.

### 4. Ticket updaten

```
ytd ticket update $TICKET_ID --json '{"summary": "[YTD-TEST] Ticket Lifecycle Test (updated)", "description": "Beschreibung wurde aktualisiert."}'
```

**Erwartung**: Gibt die Ticket-ID aus. Exit-Code 0.

### 5. Update verifizieren

```
ytd ticket get $TICKET_ID
```

**Erwartung**: Summary enthält `(updated)`. Description ist `Beschreibung wurde aktualisiert.`

### 6. Kommentar hinzufügen

```
ytd ticket comment $TICKET_ID "[YTD-TEST] Dies ist ein Test-Kommentar."
```

**Erwartung**: Exit-Code 0.

### 7. Kommentar verifizieren

```
ytd ticket get $TICKET_ID
```

**Erwartung**: Kommentar-Sektion enthält den Text `[YTD-TEST] Dies ist ein Test-Kommentar.`

### 8. Ticket suchen

```
ytd ticket search "[YTD-TEST] Ticket Lifecycle" --project $PROJECT
```

**Erwartung**: Ergebnis enthält `$TICKET_ID`.

### 9. Tickets auflisten

```
ytd ticket list --project $PROJECT
```

**Erwartung**: `$TICKET_ID` ist in der Liste enthalten.

## Cleanup

```
ytd ticket delete $TICKET_ID -y
```
