# Journey 5: Attachments

Testet: `ticket attach`, `ticket attachments`, `article attach`, `article attachments`

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
ytd ticket attachments $TICKET_ID
```

**Erwartung**: Enthält `ytd-test-attachment.txt` mit Dateigröße.

## Artikel-Attachments

### 5. Datei an Artikel anhängen

```
ytd article attach $ARTICLE_ID /tmp/ytd-test-attachment.txt
```

**Erwartung**: Exit-Code 0.

### 6. Artikel-Attachments auflisten

```
ytd article attachments $ARTICLE_ID
```

**Erwartung**: Enthält `ytd-test-attachment.txt`.

## Cleanup

```
rm /tmp/ytd-test-attachment.txt
ytd ticket delete $TICKET_ID -y
ytd article delete $ARTICLE_ID -y
```
