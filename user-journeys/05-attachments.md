# Journey 5: Attachments

Testet: `ticket attach`, `ticket attachments`, `article attach`, `article attachments`, `comment attach`, `comment attachments`, `attachment get`, `attachment download`, `attachment delete`

## Vorbereitung

Eine kleine Test-Datei wird benötigt. Der Agent erstellt sie:

```
echo "Dies ist eine YTD-Test-Datei. Kann gelöscht werden." > /tmp/ytd-test-attachment.txt
```

### 1. Ticket erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Attachment Test Ticket"}'
```

**Merke** die ID als `$TICKET_ID`.

### 2. Artikel erstellen

```
ytd article create --project $PROJECT --json '{"summary": "[YTD-TEST] Attachment Test Article", "content": "Test-Artikel für Attachment-Upload."}'
```

**Merke** die ID als `$ARTICLE_ID`.

## Ticket-Attachments

### 3. Datei an Ticket anhängen

```
ytd ticket attach $TICKET_ID /tmp/ytd-test-attachment.txt
```

**Erwartung**: Exit-Code 0. Bestätigung oder Dateiname in der Ausgabe.

### 4. Ticket-Attachments auflisten

```
ytd ticket attachments $TICKET_ID --format json
```

**Erwartung**: Valides JSON-Array. Enthält `ytd-test-attachment.txt` mit Dateigröße.

Für dieses Attachment gilt:
- `id` beginnt mit `$TICKET_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `ticket`
- `parentId` ist `$TICKET_ID`

**Merke** `id` als `$TICKET_ATTACHMENT_ID`.

### 5. Ticket-Attachment global abrufen

```
ytd attachment get $TICKET_ATTACHMENT_ID --format json
```

**Erwartung**: `id` entspricht `$TICKET_ATTACHMENT_ID`, `parentType` ist `ticket`, `name` ist `ytd-test-attachment.txt`.

### 6. Ticket-Attachment herunterladen

```
ytd attachment download $TICKET_ATTACHMENT_ID --output /tmp/ytd-downloaded-attachment.txt
```

**Erwartung**: Exit-Code 0. `/tmp/ytd-downloaded-attachment.txt` existiert und enthält den Inhalt der Test-Datei.

### 7. Ticket-Attachment löschen

```
ytd attachment delete $TICKET_ATTACHMENT_ID -y
```

**Erwartung**: Gibt `$TICKET_ATTACHMENT_ID` aus. Anschließendes `ytd attachment get $TICKET_ATTACHMENT_ID` schlägt fehl oder zeigt, falls YouTrack soft-deleted Attachments noch lesbar macht, eindeutig `removed: yes/true`. Die tatsächliche YouTrack-Semantik dokumentieren.

## Artikel-Attachments

### 8. Datei an Artikel anhängen

```
ytd article attach $ARTICLE_ID /tmp/ytd-test-attachment.txt
```

**Erwartung**: Exit-Code 0.

### 9. Artikel-Attachments auflisten

```
ytd article attachments $ARTICLE_ID --format json
```

**Erwartung**: Valides JSON-Array. Enthält `ytd-test-attachment.txt`.

Für dieses Attachment gilt:
- `id` beginnt mit `$ARTICLE_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `article`
- `parentId` ist `$ARTICLE_ID`

**Merke** `id` als `$ARTICLE_ATTACHMENT_ID`.

### 10. Artikel-Attachment global abrufen

```
ytd attachment get $ARTICLE_ATTACHMENT_ID --format json
```

**Erwartung**: `id` entspricht `$ARTICLE_ATTACHMENT_ID`, `parentType` ist `article`, `name` ist `ytd-test-attachment.txt`.

## Kommentar-Attachments

### 11. Ticket-Kommentar erstellen

```
ytd ticket comment $TICKET_ID "[YTD-TEST] Ticket-Kommentar fuer Attachment"
```

**Erwartung**: Exit-Code 0.

```
ytd ticket comments $TICKET_ID --format json
```

**Merke** die kodierte Kommentar-ID des Test-Kommentars als `$TICKET_COMMENT_ID`.

### 12. Datei an Ticket-Kommentar anhängen

```
ytd comment attach $TICKET_COMMENT_ID /tmp/ytd-test-attachment.txt
```

**Erwartung**: Exit-Code 0. Ausgabe ist `Attached ytd-test-attachment.txt`.

### 13. Ticket-Kommentar-Attachments auflisten

```
ytd comment attachments $TICKET_COMMENT_ID --format json
```

**Erwartung**: Valides JSON-Array. Enthält `ytd-test-attachment.txt`.

Für dieses Attachment gilt:
- `id` beginnt mit `$TICKET_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `ticket`
- `parentId` ist `$TICKET_ID`
- `commentId` ist `$TICKET_COMMENT_ID`

**Merke** `id` als `$TICKET_COMMENT_ATTACHMENT_ID`.

```
ytd attachment get $TICKET_COMMENT_ATTACHMENT_ID --format json
```

**Erwartung**: `id` entspricht `$TICKET_COMMENT_ATTACHMENT_ID`, `parentType` ist `ticket`, `name` ist `ytd-test-attachment.txt`, `commentId` ist `$TICKET_COMMENT_ID`.

### 14. Artikel-Kommentar erstellen

```
ytd article comment $ARTICLE_ID "[YTD-TEST] Artikel-Kommentar fuer Attachment"
```

**Erwartung**: Exit-Code 0.

```
ytd article comments $ARTICLE_ID --format json
```

**Merke** die kodierte Kommentar-ID des Test-Kommentars als `$ARTICLE_COMMENT_ID`.

### 15. Datei an Artikel-Kommentar anhängen

```
ytd comment attach $ARTICLE_COMMENT_ID /tmp/ytd-test-attachment.txt
```

**Erwartung**: Exit-Code 0. Ausgabe ist `Attached ytd-test-attachment.txt`.

### 16. Artikel-Kommentar-Attachments auflisten

```
ytd comment attachments $ARTICLE_COMMENT_ID --format json
```

**Erwartung**: Valides JSON-Array. Enthält `ytd-test-attachment.txt`.

Für dieses Attachment gilt:
- `id` beginnt mit `$ARTICLE_ID:`
- `ytId` ist vorhanden und nicht leer
- `parentType` ist `article`
- `parentId` ist `$ARTICLE_ID`
- `commentId` ist `$ARTICLE_COMMENT_ID`

**Merke** `id` als `$ARTICLE_COMMENT_ATTACHMENT_ID`.

```
ytd attachment get $ARTICLE_COMMENT_ATTACHMENT_ID --format json
```

**Erwartung**: `id` entspricht `$ARTICLE_COMMENT_ATTACHMENT_ID`, `parentType` ist `article`, `name` ist `ytd-test-attachment.txt`, `commentId` ist `$ARTICLE_COMMENT_ID`.

## Cleanup

```
rm /tmp/ytd-test-attachment.txt
rm -f /tmp/ytd-downloaded-attachment.txt
ytd ticket delete $TICKET_ID -y
ytd article delete $ARTICLE_ID -y
```
