# Journey 4: Tags & Links

Testet: `tag list`, `ticket tag`, `ticket untag`, `ticket link`, `ticket unlink`, `ticket links`, `ticket link-types`

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

### 8. Link-Typen anzeigen

```
ytd ticket link-types
```

**Erwartung**: Enthält mindestens `Relates`. Falls die Instanz Standard-Linktypen bereitstellt, enthält die Ausgabe auch `Subtask` mit `subtask of` und `parent for`.

### 9. Tickets per Link-Typ-Namen verlinken

```
ytd ticket link $TICKET_A $TICKET_B --type Relates
```

**Erwartung**: Exit-Code 0.

Falls `--type` nicht angegeben wird, soll ein sinnvoller Default verwendet werden.

### 10. Links anzeigen

```
ytd ticket links $TICKET_A
```

**Erwartung**: Enthält `$TICKET_B` und den Link-Typ. Verlinkte Tickets werden im kompakten Ticketformat angezeigt: Ticket-ID, Summary und, falls von YouTrack geliefert, wichtige Arbeitsfelder wie State, Assignee oder Priority.

### 11. Links auch beim anderen Ticket sichtbar

```
ytd ticket links $TICKET_B
```

**Erwartung**: Enthält `$TICKET_A` im kompakten Ticketformat.

### 12. Link entfernen

```
ytd ticket unlink $TICKET_A $TICKET_B --type Relates
```

**Erwartung**: Exit-Code 0.

### 13. Entfernten Link in beide Richtungen verifizieren

```
ytd ticket links $TICKET_A
ytd ticket links $TICKET_B
```

**Erwartung**: Die Ausgaben enthalten das jeweils andere Ticket nicht mehr als `relates to`-Link.

### 14. Legacy-Phrase weiter unterstützen

```
ytd ticket link $TICKET_A $TICKET_B --type "relates to"
ytd ticket unlink $TICKET_A $TICKET_B --type "relates to"
```

**Erwartung**: Beide Kommandos beenden sich mit Exit-Code 0. Die alte Command-Phrase wird weiterhin akzeptiert.

### 15. Gerichteten Standard-Linktyp prüfen, falls vorhanden

```
ytd ticket link $TICKET_A $TICKET_B --type Subtask
ytd ticket unlink $TICKET_A $TICKET_B --type Subtask
ytd ticket link $TICKET_B $TICKET_A --type Subtask --direction outward
ytd ticket unlink $TICKET_B $TICKET_A --type Subtask --direction outward
```

**Erwartung**: Wenn `Subtask` in `ytd ticket link-types` vorhanden ist, beenden sich alle Kommandos mit Exit-Code 0. Ohne `--direction` wird die inward-Richtung (`subtask of`) verwendet; `--direction outward` verwendet `parent for`.

### 16. Default-Linktyp beim Entfernen verifizieren

```
ytd ticket link $TICKET_A $TICKET_B
ytd ticket unlink $TICKET_A $TICKET_B
```

**Erwartung**: Beide Kommandos beenden sich mit Exit-Code 0. Ohne `--type` wird der Default-Linktyp `Relates` verwendet.

### 17. Default-Unlink verifizieren

```
ytd ticket links $TICKET_A
ytd ticket links $TICKET_B
```

**Erwartung**: Die Ausgaben enthalten das jeweils andere Ticket nicht mehr als `relates to`-Link.

## Cleanup

```
ytd ticket delete $TICKET_A -y
ytd ticket delete $TICKET_B -y
```
