# Journey 2: Ticket-Lifecycle

Testet: `ticket create`, `ticket get`, `ticket update`, `ticket comment`, `ticket search`, `ticket list`, Visibility-Defaults bei Create/Update, explizites Clear via `--no-visibility-group`

## Zusätzliche Voraussetzung

Vor dem Start einen gültigen Visibility-Gruppennamen als `$VIS_GROUP` festlegen. Die Gruppe muss vom aktuellen Nutzer auf Issues gesetzt und wieder entfernt werden können.

## Schritte

### 1. Ticket ohne Visibility-Default erstellen

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

**Erwartung**: Valides JSON. `idReadable` entspricht `$TICKET_ID`. Wenn das JSON ein Visibility-Feld enthält, dann darf darin keine Gruppe aus einem impliziten Default auftauchen.

### 4. Ticket mit expliziter Visibility aktualisieren

```
ytd ticket update $TICKET_ID --visibility-group "$VIS_GROUP" --json '{"summary": "[YTD-TEST] Ticket Lifecycle Test (updated)", "description": "Beschreibung wurde aktualisiert."}'
```

**Erwartung**: Gibt die Ticket-ID aus. Exit-Code 0.

### 5. Update und Visibility verifizieren

```
ytd ticket get $TICKET_ID --format raw
```

**Erwartung**: Summary enthält `(updated)`. Description ist `Beschreibung wurde aktualisiert.` Die Visibility referenziert `$VIS_GROUP`.

### 6. Visibility explizit löschen

```
ytd ticket update $TICKET_ID --no-visibility-group --json '{"description": "Beschreibung wurde aktualisiert und Visibility wurde geleert."}'
```

**Erwartung**: Exit-Code 0.

### 7. Clear verifizieren

```
ytd ticket get $TICKET_ID --format raw
```

**Erwartung**: Description ist `Beschreibung wurde aktualisiert und Visibility wurde geleert.` Im JSON ist keine eingeschränkte Visibility mit `$VIS_GROUP` mehr vorhanden.

### 8. Ticket mit Default aus Umgebung erstellen

```
env YTD_VISIBILITY_GROUP="$VIS_GROUP" ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Ticket Lifecycle Default Visibility", "description": "Erstellt mit Default-Visibility aus Umgebung."}'
```

**Erwartung**: Gibt nur die Ticket-ID aus. Exit-Code 0.

**Merke** die ID als `$DEFAULT_TICKET_ID`.

### 9. Default-Visibility verifizieren

```
ytd ticket get $DEFAULT_TICKET_ID --format raw
```

**Erwartung**: Die Visibility referenziert `$VIS_GROUP`.

### 10. Default per `--no-visibility-group` beim Update übersteuern

```
env YTD_VISIBILITY_GROUP="$VIS_GROUP" ytd ticket update $DEFAULT_TICKET_ID --no-visibility-group --json '{"description": "Default-Visibility wurde per Flag entfernt."}'
```

**Erwartung**: Gibt die Ticket-ID aus. Exit-Code 0.

### 11. Override-Clear verifizieren

```
ytd ticket get $DEFAULT_TICKET_ID --format raw
```

**Erwartung**: Description ist `Default-Visibility wurde per Flag entfernt.` Im JSON ist keine eingeschränkte Visibility mit `$VIS_GROUP` mehr vorhanden.

### 12. Kommentar hinzufügen

```
ytd ticket comment $TICKET_ID "[YTD-TEST] Dies ist ein Test-Kommentar."
```

**Erwartung**: Exit-Code 0.

### 13. Kommentar verifizieren

```
ytd ticket get $TICKET_ID
```

**Erwartung**: Kommentar-Sektion enthält den Text `[YTD-TEST] Dies ist ein Test-Kommentar.`

### 14. Ticket suchen

```
ytd ticket search "[YTD-TEST] Ticket Lifecycle" --project $PROJECT
```

**Erwartung**: Ergebnis enthält `$TICKET_ID` und `$DEFAULT_TICKET_ID`.

### 15. Tickets auflisten

```
ytd ticket list --project $PROJECT
```

**Erwartung**: `$TICKET_ID` und `$DEFAULT_TICKET_ID` sind in der Liste enthalten.

## Cleanup

```
ytd ticket delete $TICKET_ID -y
ytd ticket delete $DEFAULT_TICKET_ID -y
```
