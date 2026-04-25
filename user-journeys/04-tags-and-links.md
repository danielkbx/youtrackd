# Journey 4: Tags & Links

Testet: `tag list`, `ticket tag`, `ticket untag`, `ticket link`, `ticket links`

## Vorbereitung

Zwei Tickets werden benötigt.

### 1. Erstes Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Tags and Links - Ticket A"}'
```

**Merke** die ID als `$TICKET_A`.

### 2. Zweites Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Tags and Links - Ticket B"}'
```

**Merke** die ID als `$TICKET_B`.

## Tags testen

### 3. Verfügbare Tags anzeigen

```
ytd tag list --project $PROJECT
```

**Erwartung**: Liste von Tags, gefiltert auf das Testprojekt. Exit-Code 0.

**Merke** einen vorhandenen Tag-Namen als `$TAG` (z.B. den ersten in der Liste). Falls keine Tags existieren, diesen Abschnitt überspringen.

### 4. Tag hinzufügen

```
ytd ticket tag $TICKET_A $TAG
```

**Erwartung**: Exit-Code 0.

### 5. Tag verifizieren

```
ytd ticket get $TICKET_A --format json
```

**Erwartung**: JSON enthält `tags`-Array mit einem Eintrag, dessen `name` dem `$TAG` entspricht.

### 6. Tag entfernen

```
ytd ticket untag $TICKET_A $TAG
```

**Erwartung**: Exit-Code 0.

### 7. Tag-Entfernung verifizieren

```
ytd ticket get $TICKET_A --format json
```

**Erwartung**: `tags`-Array ist leer oder enthält `$TAG` nicht mehr.

## Links testen

### 8. Tickets verlinken

```
ytd ticket link $TICKET_A $TICKET_B --type "relates to"
```

**Erwartung**: Exit-Code 0.

Falls `--type` nicht angegeben wird, soll ein sinnvoller Default verwendet werden.

### 9. Links anzeigen

```
ytd ticket links $TICKET_A
```

**Erwartung**: Enthält `$TICKET_B` und den Link-Typ. Verlinkte Tickets werden im kompakten Ticketformat angezeigt: Ticket-ID, Summary und, falls von YouTrack geliefert, wichtige Arbeitsfelder wie State, Assignee oder Priority.

### 10. Links auch beim anderen Ticket sichtbar

```
ytd ticket links $TICKET_B
```

**Erwartung**: Enthält `$TICKET_A` im kompakten Ticketformat.

## Cleanup

```
ytd ticket delete $TICKET_A -y
ytd ticket delete $TICKET_B -y
```
