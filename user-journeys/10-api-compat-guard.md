# Absicherung: API-Kompatibilität

Dies ist keine User Journey.

Diese Datei beschreibt eine technische Absicherungs-Suite für Befehle, deren Implementierung potenziell von der veröffentlichten YouTrack-Dokumentation oder OpenAPI-Beschreibung abweicht oder auf serverseitig toleriertem Verhalten beruht.

Ziel ist nicht primär, einen realistischen Nutzer-Workflow abzubilden, sondern:

- dokumentierte API-Pfade gegen die aktuelle Implementierung abzugleichen
- instanzspezifisch toleriertes Verhalten sichtbar zu machen
- Regressionen in risikobehafteten Requests früh zu erkennen

Ein erfolgreicher Lauf bedeutet je nach Fall:

- dokumentierte API funktioniert wie erwartet
- oder die Zielinstanz akzeptiert das aktuelle Verhalten weiterhin

Ein erfolgreicher Lauf ist daher nicht automatisch ein Beleg dafür, dass die Implementierung vollständig dokumentationskonform ist.

## Fokus

Diese Absicherung prüft gezielt:

- Projektauflösung bei `ticket create`
- Projektauflösung bei `article create`
- Artikel-Suche und projektbezogene Artikelliste
- Ticket-Attachments
- Artikel-Attachments
- Custom-Field-Updates über `ticket set`
- Ticket-Verlinkung über `ticket link`
- Worklog-Erstellung über `ticket log`

## Vorbereitung

Eine kleine Test-Datei wird benötigt. Der Agent erstellt sie:

```
echo "Dies ist eine YTD-Test-Datei für API-Kompatibilitätsprüfungen." > /tmp/ytd-api-compat-attachment.txt
```

## Prüfschritte

### 1. Zielprojekt validieren

```
ytd project get $PROJECT --format raw
```

**Erwartung**: Valides JSON mit `id`, `name`, `shortName`. `shortName` entspricht `$PROJECT`.

**Merke** die Projekt-ID als `$PROJECT_DB_ID`.

### 2. Ticket im Zielprojekt erstellen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] API Compat Guard Ticket", "description": "Technischer Guard für API-Kompatibilität."}'
```

**Erwartung**: Gibt nur die Ticket-ID aus. Exit-Code 0.

**Merke** die ID als `$TICKET_ID`.

### 3. Ticket-Projekt verifizieren

```
ytd ticket get $TICKET_ID --format raw
```

**Erwartung**: `project.shortName` entspricht `$PROJECT`.

### 4. Fehlerverhalten bei ungültigem Ticket-Projekt prüfen

```
ytd ticket create --project NONEXISTENT_PROJECT_XYZ --json '{"summary": "[YTD-TEST] Invalid Project Ticket"}'
```

**Erwartung**: Exit-Code ungleich 0. Verständliche Fehlermeldung. Kein stilles Success-Verhalten.

### 5. Artikel im Zielprojekt erstellen

```
ytd article create --project $PROJECT --json '{"summary": "[YTD-TEST] API Compat Guard Article", "content": "Technischer Guard für dokumentationskritische API-Pfade."}'
```

**Erwartung**: Gibt nur die Artikel-ID aus. Exit-Code 0.

**Merke** die ID als `$ARTICLE_ID`.

### 6. Artikel-Projekt verifizieren

```
ytd article get $ARTICLE_ID --format raw
```

**Erwartung**: Artikel existiert. `project.shortName` entspricht `$PROJECT`.

### 7. Artikel-Suche prüfen

```
ytd article search "[YTD-TEST] API Compat Guard Article" --project $PROJECT
```

**Erwartung**: Ergebnis enthält `$ARTICLE_ID`.

**Interpretation bei Fehlschlag**: Kandidat für Doku-/Implementierungsabweichung bei Artikel-Suche, nicht automatisch ein generischer Laufzeitfehler.

### 8. Artikelliste für Projekt prüfen

```
ytd article list --project $PROJECT
```

**Erwartung**: Ergebnis enthält `$ARTICLE_ID`.

**Interpretation bei Fehlschlag**: Kandidat für Doku-/Implementierungsabweichung bei projektbezogener Artikelliste.

### 9. Datei an Ticket anhängen

```
ytd ticket attach $TICKET_ID /tmp/ytd-api-compat-attachment.txt
```

**Erwartung**: Exit-Code 0.

### 10. Ticket-Attachment persistent verifizieren

```
ytd ticket attachments $TICKET_ID --format raw
```

**Erwartung**: JSON enthält `ytd-api-compat-attachment.txt`. Eintrag hat `size` > 0.

**Interpretation bei Fehlschlag**: Kandidat für Multipart-Inkompatibilität oder falsches Feldformat beim Upload.

### 11. Datei an Artikel anhängen

```
ytd article attach $ARTICLE_ID /tmp/ytd-api-compat-attachment.txt
```

**Erwartung**: Exit-Code 0.

### 12. Artikel-Attachment persistent verifizieren

```
ytd article attachments $ARTICLE_ID --format raw
```

**Erwartung**: JSON enthält `ytd-api-compat-attachment.txt`. Eintrag hat `size` > 0.

**Interpretation bei Fehlschlag**: Kandidat für Doku-/Server-Verhaltensabweichung bei Artikel-Attachments.

### 13. Aktuelle Custom Fields lesen

```
ytd ticket fields $TICKET_ID --format raw
```

**Erwartung**: Valides JSON-Array. Enthält typische Felder wie `Priority`, `State` oder `Type`.

### 14. Priority setzen

```
ytd ticket set $TICKET_ID Priority Major
```

**Erwartung**: Exit-Code 0.

### 15. Priority-Verifikation

```
ytd ticket get $TICKET_ID --format raw
```

**Erwartung**: In `customFields` existiert ein Eintrag `Priority`, dessen Wert `Major` ist.

### 16. Zweites Ticket für Link-Prüfung erzeugen

```
ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] API Compat Guard Link Target"}'
```

**Erwartung**: Gibt nur die Ticket-ID aus. Exit-Code 0.

**Merke** die ID als `$LINK_TARGET_ID`.

### 17. Tickets verlinken

```
ytd ticket link $TICKET_ID $LINK_TARGET_ID --type "relates to"
```

**Erwartung**: Exit-Code 0.

### 18. Link in beide Richtungen verifizieren

```
ytd ticket links $TICKET_ID
ytd ticket links $LINK_TARGET_ID
```

**Erwartung**: Beide Ausgaben referenzieren jeweils das andere Ticket.

### 19. Zeit auf Ticket buchen

```
ytd ticket log $TICKET_ID 30m "[YTD-TEST] API Compat Guard Worklog"
```

**Erwartung**: Exit-Code 0.

### 20. Worklog strukturiert verifizieren

```
ytd ticket worklog $TICKET_ID --format raw
```

**Erwartung**: JSON enthält einen Eintrag mit Text `[YTD-TEST] API Compat Guard Worklog` und `duration.minutes`.

## Interpretation

Ein Fehlschlag in dieser Datei ist nicht automatisch ein Produktfehler in `ytd`.

Mögliche Ursachen:

- tatsächlicher Fehler in der CLI-Implementierung
- projektspezifische Validierung oder Workflow-Regeln
- Abweichung zwischen offizieller Dokumentation und tatsächlichem Serververhalten
- Verhalten, das von der Instanz toleriert wird, aber nicht öffentlich dokumentiert ist

Diese Datei dient dazu, genau diese Fälle sichtbar zu machen.

## Cleanup

```
rm -f /tmp/ytd-api-compat-attachment.txt
ytd ticket delete $TICKET_ID -y
ytd ticket delete $LINK_TARGET_ID -y
ytd article delete $ARTICLE_ID -y
```
