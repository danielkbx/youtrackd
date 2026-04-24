# Journey 12: Kommentare

Testet: `ticket comments`, `article comments`, `comment get`, `comment update`, `comment delete`, kodierte Kommentar-IDs

Diese Journey prüft die zentrale Kommentar-ID-Invariante: Jede vom Tool ausgegebene Kommentar-ID muss direkt mit `ytd comment get|update|delete` funktionieren.

## Schritte

### 1. Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Comment Journey Ticket", "description": "Ticket fuer Kommentar-Journey."}'
```

**Erwartung**: Gibt nur die Ticket-ID aus. Exit-Code 0.

**Merke** die ID als `$TICKET_ID`.

### 2. Artikel erstellen

```
ytd article create --project $PROJECT --json '{"summary": "[YTD-TEST] Comment Journey Article", "content": "Artikel fuer Kommentar-Journey."}'
```

**Erwartung**: Gibt nur die Artikel-ID aus. Exit-Code 0.

**Merke** die ID als `$ARTICLE_ID`.

### 3. Ticket-Kommentar erstellen

```
ytd ticket comment $TICKET_ID "[YTD-TEST] Ticket-Kommentar initial"
```

**Erwartung**: Exit-Code 0.

### 4. Artikel-Kommentar erstellen

```
ytd article comment $ARTICLE_ID "[YTD-TEST] Artikel-Kommentar initial"
```

**Erwartung**: Exit-Code 0.

### 5. Ticket-Kommentare als JSON listen

```
ytd ticket comments $TICKET_ID --format raw
```

**Erwartung**: Valides JSON-Array. Ein Kommentar enthält `[YTD-TEST] Ticket-Kommentar initial`.

Für diesen Kommentar gilt:
- `id` beginnt mit `$TICKET_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `ticket`
- `parentId` ist `$TICKET_ID`

**Merke** `id` als `$TICKET_COMMENT_ID`.

### 6. Artikel-Kommentare als JSON listen

```
ytd article comments $ARTICLE_ID --format raw
```

**Erwartung**: Valides JSON-Array. Ein Kommentar enthält `[YTD-TEST] Artikel-Kommentar initial`.

Für diesen Kommentar gilt:
- `id` beginnt mit `$ARTICLE_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `article`
- `parentId` ist `$ARTICLE_ID`

**Merke** `id` als `$ARTICLE_COMMENT_ID`.

### 7. Ticket-Kommentar per globalem Command abrufen

```
ytd comment get $TICKET_COMMENT_ID --format raw
```

**Erwartung**: `id` entspricht `$TICKET_COMMENT_ID`, `parentType` ist `ticket`, `parentId` ist `$TICKET_ID`, `text` enthält `[YTD-TEST] Ticket-Kommentar initial`.

### 8. Artikel-Kommentar per globalem Command abrufen

```
ytd comment get $ARTICLE_COMMENT_ID --format raw
```

**Erwartung**: `id` entspricht `$ARTICLE_COMMENT_ID`, `parentType` ist `article`, `parentId` ist `$ARTICLE_ID`, `text` enthält `[YTD-TEST] Artikel-Kommentar initial`.

### 9. Ticket-Kommentar aktualisieren

```
ytd comment update $TICKET_COMMENT_ID "[YTD-TEST] Ticket-Kommentar aktualisiert"
```

**Erwartung**: Gibt `$TICKET_COMMENT_ID` aus. Exit-Code 0.

### 10. Artikel-Kommentar aktualisieren

```
ytd comment update $ARTICLE_COMMENT_ID "[YTD-TEST] Artikel-Kommentar aktualisiert"
```

**Erwartung**: Gibt `$ARTICLE_COMMENT_ID` aus. Exit-Code 0.

### 11. Updates verifizieren

```
ytd comment get $TICKET_COMMENT_ID --format raw
ytd comment get $ARTICLE_COMMENT_ID --format raw
```

**Erwartung**: Ticket-Kommentar enthält `Ticket-Kommentar aktualisiert`. Artikel-Kommentar enthält `Artikel-Kommentar aktualisiert`.

### 12. Eingebettete Ticket-Kommentar-ID prüfen

```
ytd ticket get $TICKET_ID --format raw
```

**Erwartung**: Falls `comments` Kommentarobjekte enthält, ist deren `id` kodiert und beginnt mit `$TICKET_ID:`. Jede gefundene Kommentar-`id` funktioniert mit `ytd comment get`.

### 13. Ticket-Kommentar löschen

```
ytd comment delete $TICKET_COMMENT_ID -y
```

**Erwartung**: Gibt `$TICKET_COMMENT_ID` aus. Exit-Code 0.

### 14. Artikel-Kommentar löschen

```
ytd comment delete $ARTICLE_COMMENT_ID -y
```

**Erwartung**: Gibt `$ARTICLE_COMMENT_ID` aus. Exit-Code 0.

### 15. Deletes verifizieren

```
ytd comment get $TICKET_COMMENT_ID
ytd comment get $ARTICLE_COMMENT_ID
```

**Erwartung**: Beide Commands schlagen fehl oder zeigen, falls YouTrack soft-deleted Kommentare noch lesbar macht, eindeutig `deleted: yes/true`. Die tatsächliche YouTrack-Semantik dokumentieren.

## Cleanup

```
ytd ticket delete $TICKET_ID -y
ytd article delete $ARTICLE_ID -y
```
