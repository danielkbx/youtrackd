# Journey 12: Kommentare

Testet: `ticket comments`, `article comments`, `comment get`, `comment update`, `comment attach`, `comment attachments`, `comment delete`, kodierte Kommentar-IDs

Diese Journey prüft die zentrale Kommentar-ID-Invariante: Jede vom Tool ausgegebene Kommentar-ID muss direkt mit `ytd comment get|update|attach|delete|attachments` funktionieren.

## Zusätzliche Voraussetzung

Vor dem Start einen gültigen Visibility-Gruppennamen als `$VIS_GROUP` festlegen. Die Gruppe muss vom aktuellen Nutzer auf Ticket- und Artikel-Kommentare gesetzt und wieder entfernt werden können.

Eine kleine Test-Datei wird benötigt. Der Agent erstellt sie:

```
echo "Dies ist eine YTD-Test-Datei fuer Kommentar-Attachments." > /tmp/ytd-comment-attachment.txt
```

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
env YTD_VISIBILITY_GROUP="$VIS_GROUP" ytd ticket comment $TICKET_ID "[YTD-TEST] **Ticket-Kommentar** erstellt"
```

**Erwartung**: Exit-Code 0. Der Kommentar übernimmt die Default-Visibility aus `$VIS_GROUP`.

### 4. Artikel-Kommentar erstellen

```
env YTD_VISIBILITY_GROUP="$VIS_GROUP" ytd article comment $ARTICLE_ID "[YTD-TEST] Artikel-Kommentar erstellt"
```

**Erwartung**: Exit-Code 0. Der Kommentar übernimmt die Default-Visibility aus `$VIS_GROUP`.

### 5. Ticket-Kommentare als JSON listen

```
ytd ticket comments $TICKET_ID --format json
```

**Erwartung**: Valides JSON-Array. Ein Kommentar enthält `[YTD-TEST] **Ticket-Kommentar** erstellt`.

Für diesen Kommentar gilt:
- `id` beginnt mit `$TICKET_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `ticket`
- `parentId` ist `$TICKET_ID`

**Merke** `id` als `$TICKET_COMMENT_ID`.

### 6. Artikel-Kommentare als JSON listen

```
ytd article comments $ARTICLE_ID --format json
```

**Erwartung**: Valides JSON-Array. Ein Kommentar enthält `[YTD-TEST] Artikel-Kommentar erstellt`.

Für diesen Kommentar gilt:
- `id` beginnt mit `$ARTICLE_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `article`
- `parentId` ist `$ARTICLE_ID`

**Merke** `id` als `$ARTICLE_COMMENT_ID`.

### 7. Ticket-Kommentar per globalem Command abrufen

```
ytd comment get $TICKET_COMMENT_ID --format json
```

**Erwartung**: `id` entspricht `$TICKET_COMMENT_ID`, `parentType` ist `ticket`, `parentId` ist `$TICKET_ID`, `text` enthält `[YTD-TEST] **Ticket-Kommentar** erstellt`.

Die `visibility.permittedGroups` enthält `$VIS_GROUP`.

```
ytd comment get $TICKET_COMMENT_ID
```

**Erwartung**: Text-Output zeigt den Kommentartext als Plain Text, also `[YTD-TEST] Ticket-Kommentar erstellt` ohne `**`, und der Text steht nach den Metadaten.

### 8. Artikel-Kommentar per globalem Command abrufen

```
ytd comment get $ARTICLE_COMMENT_ID --format json
```

**Erwartung**: `id` entspricht `$ARTICLE_COMMENT_ID`, `parentType` ist `article`, `parentId` ist `$ARTICLE_ID`, `text` enthält `[YTD-TEST] Artikel-Kommentar erstellt`.

Die `visibility.permittedGroups` enthält `$VIS_GROUP`.

### 9. Datei an Ticket-Kommentar anhängen

```
ytd comment attach $TICKET_COMMENT_ID /tmp/ytd-comment-attachment.txt
```

**Erwartung**: Exit-Code 0. Ausgabe ist `Attached ytd-comment-attachment.txt`.

### 10. Ticket-Kommentar-Attachments listen

```
ytd comment attachments $TICKET_COMMENT_ID --format json
```

**Erwartung**: Valides JSON-Array. Enthält `ytd-comment-attachment.txt`.

Für dieses Attachment gilt:
- `id` beginnt mit `$TICKET_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `ticket`
- `parentId` ist `$TICKET_ID`
- `commentId` ist `$TICKET_COMMENT_ID`

### 11. Datei an Artikel-Kommentar anhängen

```
ytd comment attach $ARTICLE_COMMENT_ID /tmp/ytd-comment-attachment.txt
```

**Erwartung**: Exit-Code 0. Ausgabe ist `Attached ytd-comment-attachment.txt`.

### 12. Artikel-Kommentar-Attachments listen

```
ytd comment attachments $ARTICLE_COMMENT_ID --format json
```

**Erwartung**: Valides JSON-Array. Enthält `ytd-comment-attachment.txt`.

Für dieses Attachment gilt:
- `id` beginnt mit `$ARTICLE_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `article`
- `parentId` ist `$ARTICLE_ID`
- `commentId` ist `$ARTICLE_COMMENT_ID`

### 13. Ticket-Kommentar aktualisieren

```
ytd comment update $TICKET_COMMENT_ID "[YTD-TEST] Ticket-Kommentar aktualisiert"
```

**Erwartung**: Gibt `$TICKET_COMMENT_ID` aus. Exit-Code 0.

### 14. Artikel-Kommentar aktualisieren

```
ytd comment update $ARTICLE_COMMENT_ID "[YTD-TEST] Artikel-Kommentar aktualisiert"
```

**Erwartung**: Gibt `$ARTICLE_COMMENT_ID` aus. Exit-Code 0.

### 15. Updates verifizieren

```
ytd comment get $TICKET_COMMENT_ID --format json
ytd comment get $ARTICLE_COMMENT_ID --format json
```

**Erwartung**: Ticket-Kommentar enthält `Ticket-Kommentar aktualisiert`. Artikel-Kommentar enthält `Artikel-Kommentar aktualisiert`.

Die `visibility.permittedGroups` enthält weiterhin `$VIS_GROUP`. `comment update` ohne Visibility-Flags darf die bestehende Visibility nicht verändern.

### 16. Visibility löschen

```
ytd comment update $TICKET_COMMENT_ID "[YTD-TEST] Ticket-Kommentar public" --no-visibility-group
ytd comment update $ARTICLE_COMMENT_ID "[YTD-TEST] Artikel-Kommentar public" --no-visibility-group
```

**Erwartung**: Beide Commands geben die jeweilige Kommentar-ID aus.

```
ytd comment get $TICKET_COMMENT_ID --format json
ytd comment get $ARTICLE_COMMENT_ID --format json
```

**Erwartung**: Es bleibt keine eingeschränkte Visibility mit `$VIS_GROUP` vorhanden. Je nach YouTrack-Antwort ist `visibility` unlimited, leer oder ohne `permittedGroups`.

### 17. Visibility explizit setzen

```
ytd comment update $TICKET_COMMENT_ID "[YTD-TEST] Ticket-Kommentar restricted again" --visibility-group "$VIS_GROUP"
ytd comment update $ARTICLE_COMMENT_ID "[YTD-TEST] Artikel-Kommentar restricted again" --visibility-group "$VIS_GROUP"
```

**Erwartung**: Beide Commands geben die jeweilige Kommentar-ID aus.

```
ytd comment get $TICKET_COMMENT_ID --format json
ytd comment get $ARTICLE_COMMENT_ID --format json
```

**Erwartung**: Beide Kommentare enthalten `$VIS_GROUP` in `visibility.permittedGroups`.

### 18. Eingebettete Ticket-Kommentar-ID prüfen

```
ytd ticket get $TICKET_ID --format json
```

**Erwartung**: Falls `comments` Kommentarobjekte enthält, ist deren `id` kodiert und beginnt mit `$TICKET_ID:`. Jede gefundene Kommentar-`id` funktioniert mit `ytd comment get`.

### 19. Ticket-Kommentar löschen

```
ytd comment delete $TICKET_COMMENT_ID -y
```

**Erwartung**: Gibt `$TICKET_COMMENT_ID` aus. Exit-Code 0.

### 20. Artikel-Kommentar löschen

```
ytd comment delete $ARTICLE_COMMENT_ID -y
```

**Erwartung**: Gibt `$ARTICLE_COMMENT_ID` aus. Exit-Code 0.

### 21. Deletes verifizieren

```
ytd comment get $TICKET_COMMENT_ID
ytd comment get $ARTICLE_COMMENT_ID
```

**Erwartung**: Beide Commands schlagen fehl oder zeigen, falls YouTrack soft-deleted Kommentare noch lesbar macht, eindeutig `deleted: yes/true`. Die tatsächliche YouTrack-Semantik dokumentieren.

## Cleanup

```
ytd ticket delete $TICKET_ID -y
ytd article delete $ARTICLE_ID -y
rm -f /tmp/ytd-comment-attachment.txt
```
