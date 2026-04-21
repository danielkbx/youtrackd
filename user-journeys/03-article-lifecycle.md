# Journey 3: Artikel-Lifecycle

Testet: `article create`, `article get`, `article update`, `article append`, `article comment`, `article comments`, `article search`, `article list`, `--format md`

## Schritte

### 1. Artikel erstellen

```
ytd article create --project $PROJECT --json '{"summary": "[YTD-TEST] Article Lifecycle Test", "content": "Erster Absatz des Test-Artikels."}'
```

**Erwartung**: Gibt nur die Artikel-ID aus (z.B. `PROJ-A-1`). Exit-Code 0.

**Merke** die ID als `$ARTICLE_ID`.

### 2. Artikel abrufen

```
ytd article get $ARTICLE_ID
```

**Erwartung**: Summary enthält `[YTD-TEST] Article Lifecycle Test`. Content enthält `Erster Absatz`.

### 3. Artikel updaten

```
ytd article update $ARTICLE_ID --json '{"summary": "[YTD-TEST] Article Lifecycle Test (updated)"}'
```

**Erwartung**: Gibt die Artikel-ID aus. Exit-Code 0.

### 4. Text anhängen

```
ytd article append $ARTICLE_ID "\n\nZweiter Absatz, per append hinzugefügt."
```

**Erwartung**: Exit-Code 0.

### 5. Append verifizieren

```
ytd article get $ARTICLE_ID
```

**Erwartung**: Content enthält sowohl `Erster Absatz` als auch `Zweiter Absatz`.

### 6. Kommentar zu Artikel hinzufügen

```
ytd article comment $ARTICLE_ID "[YTD-TEST] Kommentar zum Test-Artikel."
```

**Erwartung**: Exit-Code 0.

### 7. Artikel-Kommentare anzeigen

```
ytd article comments $ARTICLE_ID
```

**Erwartung**: Enthält `[YTD-TEST] Kommentar zum Test-Artikel.`

### 8. Artikel suchen

```
ytd article search "[YTD-TEST] Article Lifecycle" --project $PROJECT
```

**Erwartung**: Ergebnis enthält `$ARTICLE_ID`.

### 9. Artikel auflisten

```
ytd article list --project $PROJECT
```

**Erwartung**: `$ARTICLE_ID` ist in der Liste enthalten.

### 10. Artikel als Markdown abrufen

```
ytd article get $ARTICLE_ID --format md
```

**Erwartung**: Ausgabe beginnt mit `# [YTD-TEST] Article Lifecycle Test (updated)` (H1 aus Summary). Danach folgt der Content als Markdown-Body.

### 11. Artikel als Markdown in Datei schreiben

```
ytd article get $ARTICLE_ID --format md > /tmp/ytd-test-article.md
```

**Erwartung**: Datei `/tmp/ytd-test-article.md` existiert. Inhalt beginnt mit `# [YTD-TEST]`. Datei enthält sowohl `Erster Absatz` als auch `Zweiter Absatz`.

## Cleanup

```
ytd article delete $ARTICLE_ID -y
rm -f /tmp/ytd-test-article.md
```
