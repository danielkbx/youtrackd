# Journey 3: Artikel-Lifecycle

Testet: `article create`, `article get`, `article update`, `article append`, `article comment`, `article comments`, `article search`, `article list`, `--format md`, Visibility-Defaults bei Create, explizite Visibility-Änderung bei Update, explizites Clear via `--no-visibility-group`

## Zusätzliche Voraussetzung

Vor dem Start einen gültigen Visibility-Gruppennamen als `$VIS_GROUP` festlegen. Die Gruppe muss vom aktuellen Nutzer auf Artikel gesetzt und wieder entfernt werden können.

## Schritte

### 1. Artikel ohne Visibility-Default erstellen

```
ytd article create --project $PROJECT --json '{"summary": "[YTD-TEST] Article Lifecycle Test", "content": "## Einstieg\n\nErster **Absatz** des Test-Artikels.\n\n| Name | Wert |\n|---|---:|\n| Test | 1 |"}'
```

**Erwartung**: Gibt nur die Artikel-ID aus (z.B. `PROJ-A-1`). Exit-Code 0.

**Merke** die ID als `$ARTICLE_ID`.

### 2. Artikel abrufen

```
ytd article get $ARTICLE_ID
```

**Erwartung**: Summary enthält `[YTD-TEST] Article Lifecycle Test`. Der Text-Output zeigt Metadaten vor dem Content. Content steht nach einer Leerzeile am Ende ohne `Content:`-Label und enthält `Einstieg` und `Erster Absatz des Test-Artikels`, aber keine Markdown-Markierungen wie `##` oder `**`. Die Markdown-Tabelle wird als ASCII-Tabelle mit `+`, `-` und `|` gerendert.

### 3. Artikel mit expliziter Visibility aktualisieren

```
ytd article update $ARTICLE_ID --visibility-group "$VIS_GROUP" --json '{"summary": "[YTD-TEST] Article Lifecycle Test (updated)"}'
```

**Erwartung**: Gibt die Artikel-ID aus. Exit-Code 0.

### 4. Update und Visibility verifizieren

```
ytd article get $ARTICLE_ID --format json
```

**Erwartung**: Summary enthält `(updated)`. Das JSON-Feld `id` entspricht `$ARTICLE_ID`; falls eine rohe YouTrack-Datenbank-ID vorhanden ist, steht sie in `ytId`, nicht in `idReadable`. Die Visibility referenziert `$VIS_GROUP`.

### 5. Text anhängen

```
ytd article append $ARTICLE_ID "\n\nZweiter Absatz, per append hinzugefügt."
```

**Erwartung**: Exit-Code 0.

### 6. Append verifizieren

```
ytd article get $ARTICLE_ID
```

**Erwartung**: Content steht nach einer Leerzeile am Ende der Textausgabe ohne `Content:`-Label und enthält sowohl `Erster Absatz` als auch `Zweiter Absatz`.

### 7. Visibility explizit löschen

```
ytd article update $ARTICLE_ID --no-visibility-group --json '{"content": "## Einstieg\n\nErster **Absatz** des Test-Artikels.\n\n| Name | Wert |\n|---|---:|\n| Test | 1 |\n\nZweiter Absatz, per append hinzugefügt.\n\nVisibility wurde explizit geleert."}'
```

**Erwartung**: Gibt die Artikel-ID aus. Exit-Code 0.

### 8. Clear verifizieren

```
ytd article get $ARTICLE_ID --format json
```

**Erwartung**: Content enthält `Visibility wurde explizit geleert.` Das JSON-Feld `id` entspricht `$ARTICLE_ID`; falls eine rohe YouTrack-Datenbank-ID vorhanden ist, steht sie in `ytId`, nicht in `idReadable`. Im JSON ist keine eingeschränkte Visibility mit `$VIS_GROUP` mehr vorhanden.

### 9. Artikel mit Default aus Umgebung erstellen

```
env YTD_VISIBILITY_GROUP="$VIS_GROUP" ytd article create --project $PROJECT --json '{"summary": "[YTD-TEST] Article Lifecycle Default Visibility", "content": "Erstellt mit Default-Visibility aus Umgebung."}'
```

**Erwartung**: Gibt nur die Artikel-ID aus. Exit-Code 0.

**Merke** die ID als `$DEFAULT_ARTICLE_ID`.

### 10. Default-Visibility verifizieren

```
ytd article get $DEFAULT_ARTICLE_ID --format json
```

**Erwartung**: Die Visibility referenziert `$VIS_GROUP`.

### 11. Default per `--no-visibility-group` beim Update übersteuern

```
env YTD_VISIBILITY_GROUP="$VIS_GROUP" ytd article update $DEFAULT_ARTICLE_ID --no-visibility-group --json '{"content": "Default-Visibility wurde per Flag entfernt."}'
```

**Erwartung**: Gibt die Artikel-ID aus. Exit-Code 0.

### 12. Override-Clear verifizieren

```
ytd article get $DEFAULT_ARTICLE_ID --format json
```

**Erwartung**: Content ist `Default-Visibility wurde per Flag entfernt.` Im JSON ist keine eingeschränkte Visibility mit `$VIS_GROUP` mehr vorhanden.

### 13. Kommentar zu Artikel hinzufügen

```
ytd article comment $ARTICLE_ID "[YTD-TEST] Kommentar zum Test-Artikel."
```

**Erwartung**: Exit-Code 0.

### 14. Artikel-Kommentare anzeigen

```
ytd article comments $ARTICLE_ID
```

**Erwartung**: Enthält `[YTD-TEST] Kommentar zum Test-Artikel.`

### 15. Artikel-Kommentar-ID verifizieren

```
ytd article get $ARTICLE_ID --no-comments
```

**Erwartung**: Exit-Code 0. Ausgabe enthält keine `Comments`-Sektion und nicht den Text `[YTD-TEST] Kommentar zum Test-Artikel.`

```
ytd article comments $ARTICLE_ID --format json
```

**Erwartung**: Valides JSON-Array. Der Test-Kommentar ist enthalten. Seine `id` beginnt mit `$ARTICLE_ID:`, `ytId` ist vorhanden, `parentType` ist `article`, `parentId` ist `$ARTICLE_ID`.

**Merke** die Kommentar-ID als `$ARTICLE_COMMENT_ID`.

```
ytd comment get $ARTICLE_COMMENT_ID --format json
```

**Erwartung**: Der Kommentar wird geladen und enthält `[YTD-TEST] Kommentar zum Test-Artikel.`

### 16. Artikel suchen

```
ytd article search "[YTD-TEST] Article Lifecycle" --project $PROJECT
```

**Erwartung**: Ergebnis enthält `$ARTICLE_ID` und `$DEFAULT_ARTICLE_ID` im Feld `id`. Es gibt kein Feld `idReadable`; falls eine rohe YouTrack-Datenbank-ID ausgegeben wird, heißt sie `ytId`.

### 17. Artikel auflisten

```
ytd article list --project $PROJECT
```

**Erwartung**: `$ARTICLE_ID` und `$DEFAULT_ARTICLE_ID` sind in der Liste im Feld `id` enthalten. Es gibt kein Feld `idReadable`; falls eine rohe YouTrack-Datenbank-ID ausgegeben wird, heißt sie `ytId`.

### 18. Artikel als Markdown abrufen

```
ytd article get $ARTICLE_ID --format md
```

**Erwartung**: Ausgabe beginnt mit `# [YTD-TEST] Article Lifecycle Test (updated)` (H1 aus Summary). Danach folgt der Content als Markdown-Body.

### 19. Artikel als Markdown in Datei schreiben

```
ytd article get $ARTICLE_ID --format md > /tmp/ytd-test-article.md
```

**Erwartung**: Datei `/tmp/ytd-test-article.md` existiert. Inhalt beginnt mit `# [YTD-TEST]`. Datei enthält sowohl `Erster Absatz` als auch `Zweiter Absatz`.

## Cleanup

```
ytd article delete $ARTICLE_ID -y
ytd article delete $DEFAULT_ARTICLE_ID -y
rm -f /tmp/ytd-test-article.md
```
