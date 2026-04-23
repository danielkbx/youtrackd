# Journey 11: Visibility-Defaults und Config-Precedence

Testet: isolierte `YTD_CONFIG`-Dateien, Reihenfolge `--visibility-group` → `YTD_VISIBILITY_GROUP` → gespeicherter Config-Wert, explizites Override mit `--no-visibility-group`, Cleanup von Temp-Dateien und Env Vars

## Zusätzliche Voraussetzung

Vor dem Start:

- Drei gültige Visibility-Gruppennamen festlegen, wenn die Instanz das zulässt:
  - `$VIS_GROUP_CONFIG` für gespeicherte Config-Defaults
  - `$VIS_GROUP_ENV` für `YTD_VISIBILITY_GROUP`
  - `$VIS_GROUP_CLI` für das explizite CLI-Flag
- Zwei isolierte Config-Dateien vorbereiten, die beide gültige `url`- und `token`-Werte für die Zielinstanz enthalten.
- Wenn die aktuelle Anmeldung bereits aus einer Config-Datei kommt, kann diese als Vorlage kopiert werden. Wenn die aktuelle Anmeldung nur über `YOUTRACK_URL` und `YOUTRACK_TOKEN` erfolgt, die Dateien manuell mit denselben Werten anlegen.

Wenn nicht drei unterschiedliche Gruppen verfügbar sind, mindestens zwei unterschiedliche Gruppen verwenden und die Wiederverwendung im Testprotokoll notieren. Mit nur einer Gruppe ist diese Journey nicht aussagekräftig genug für einen echten Precedence-Nachweis.

## Vorbereitung

### 1. Temp-Dateien anlegen

```
CONFIG_A=$(mktemp /tmp/ytd-config-a.XXXXXX.json)
CONFIG_B=$(mktemp /tmp/ytd-config-b.XXXXXX.json)
```

**Erwartung**: Beide Pfade existieren und sind unterschiedlich.

### 2. Ausgangskonfiguration schreiben

Datei `CONFIG_A` enthält gültige Zugangsdaten plus einen gespeicherten Default:

```json
{
  "url": "https://your-instance.youtrack.cloud",
  "token": "perm:...",
  "visibilityGroup": "REPLACE_WITH_VIS_GROUP_CONFIG"
}
```

Datei `CONFIG_B` enthält dieselben Zugangsdaten, aber einen anderen gespeicherten Default:

```json
{
  "url": "https://your-instance.youtrack.cloud",
  "token": "perm:...",
  "visibilityGroup": "REPLACE_WITH_VIS_GROUP_CONFIG_B"
}
```

`CONFIG_B` darf denselben Wert wie `CONFIG_A` verwenden, solange sich mindestens Env-Var und CLI-Flag noch beobachtbar davon unterscheiden.

### 3. Störende Env Vars entfernen

```
unset YTD_VISIBILITY_GROUP
unset YOUTRACK_URL
unset YOUTRACK_TOKEN
```

**Erwartung**: Die folgenden Schritte beziehen Auth und gespeicherte Defaults ausschließlich aus `YTD_CONFIG`.

## Prüfschritte

### 4. Stored Config als Default für Ticket-Create verwenden

```
env YTD_CONFIG="$CONFIG_A" ytd ticket create --project $PROJECT --json '{"summary": "[YTD-TEST] Visibility Config Ticket", "description": "Ticket mit Default aus isolierter Config."}'
```

**Erwartung**: Gibt nur die Ticket-ID aus.

**Merke** die ID als `$CONFIG_TICKET_ID`.

### 5. Stored-Config-Default verifizieren

```
env YTD_CONFIG="$CONFIG_A" ytd ticket get $CONFIG_TICKET_ID --format raw
```

**Erwartung**: Die Visibility referenziert `$VIS_GROUP_CONFIG`.

### 6. Env Var gewinnt gegen gespeicherten Config-Wert

```
env YTD_CONFIG="$CONFIG_A" YTD_VISIBILITY_GROUP="$VIS_GROUP_ENV" ytd ticket update $CONFIG_TICKET_ID --json '{"description": "Visibility kommt jetzt aus der Env-Var."}'
```

**Erwartung**: Gibt nur die Ticket-ID aus.

### 7. Env-Precedence verifizieren

```
env YTD_CONFIG="$CONFIG_A" ytd ticket get $CONFIG_TICKET_ID --format raw
```

**Erwartung**: Die Visibility referenziert `$VIS_GROUP_ENV`, nicht `$VIS_GROUP_CONFIG`.

### 8. `--no-visibility-group` sticht Env Var und Config bei Ticket-Update aus

```
env YTD_CONFIG="$CONFIG_A" YTD_VISIBILITY_GROUP="$VIS_GROUP_ENV" ytd ticket update $CONFIG_TICKET_ID --no-visibility-group --json '{"description": "Visibility wurde trotz Defaults explizit geleert."}'
```

**Erwartung**: Gibt nur die Ticket-ID aus.

### 9. Clear verifizieren

```
env YTD_CONFIG="$CONFIG_A" ytd ticket get $CONFIG_TICKET_ID --format raw
```

**Erwartung**: Im JSON ist keine eingeschränkte Visibility mit `$VIS_GROUP_ENV` oder `$VIS_GROUP_CONFIG` mehr vorhanden.

### 10. CLI-Flag gewinnt gegen Env Var und Config bei Artikel-Create

```
env YTD_CONFIG="$CONFIG_B" YTD_VISIBILITY_GROUP="$VIS_GROUP_ENV" ytd article create --project $PROJECT --visibility-group "$VIS_GROUP_CLI" --json '{"summary": "[YTD-TEST] Visibility Config Article", "content": "Artikel mit explizitem CLI-Override."}'
```

**Erwartung**: Gibt nur die Artikel-ID aus.

**Merke** die ID als `$CONFIG_ARTICLE_ID`.

### 11. CLI-Precedence verifizieren

```
env YTD_CONFIG="$CONFIG_B" ytd article get $CONFIG_ARTICLE_ID --format raw
```

**Erwartung**: Die Visibility referenziert `$VIS_GROUP_CLI`, auch wenn `CONFIG_B` einen gespeicherten Default und die Env-Var andere Werte liefern.

### 12. `--no-visibility-group` sticht Env Var und Config bei Artikel-Update aus

```
env YTD_CONFIG="$CONFIG_B" YTD_VISIBILITY_GROUP="$VIS_GROUP_ENV" ytd article update $CONFIG_ARTICLE_ID --no-visibility-group --json '{"content": "Visibility wurde trotz Config und Env explizit entfernt."}'
```

**Erwartung**: Gibt nur die Artikel-ID aus.

### 13. Artikel-Clear verifizieren

```
env YTD_CONFIG="$CONFIG_B" ytd article get $CONFIG_ARTICLE_ID --format raw
```

**Erwartung**: Im JSON ist keine eingeschränkte Visibility mit `$VIS_GROUP_CLI`, `$VIS_GROUP_ENV` oder dem gespeicherten Config-Wert mehr vorhanden.

### 14. Isolierung der Config-Dateien gegeneinander prüfen

```
env YTD_CONFIG="$CONFIG_A" ytd whoami
env YTD_CONFIG="$CONFIG_B" ytd whoami
```

**Erwartung**: Beide Aufrufe funktionieren unabhängig voneinander. Änderungen an einem Default haben die jeweils andere Datei nicht implizit verändert.

## Cleanup

```
env YTD_CONFIG="$CONFIG_A" ytd ticket delete $CONFIG_TICKET_ID -y
env YTD_CONFIG="$CONFIG_B" ytd article delete $CONFIG_ARTICLE_ID -y
unset YTD_CONFIG
unset YTD_VISIBILITY_GROUP
unset YOUTRACK_URL
unset YOUTRACK_TOKEN
rm -f "$CONFIG_A" "$CONFIG_B"
```
