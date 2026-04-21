# Journey 8: Saved Searches & Boards

Testet: `search list`, `search run`, `board list`, `board get`

## Cleanup

Kein Cleanup nötig — diese Journey erzeugt keine Entities.

**Hinweis**: Diese Journey setzt voraus, dass im YouTrack-System mindestens eine Saved Search und ein Agile Board existieren. Falls nicht, werden die entsprechenden Schritte übersprungen.

## Saved Searches

### 1. Gespeicherte Suchen auflisten

```
ytd search list --project $PROJECT
```

**Erwartung**: Exit-Code 0. Falls Saved Searches vorhanden: Liste mit Name und Query, gefiltert auf das Testprojekt.

Falls die Liste leer ist, Schritte 2-3 überspringen.

### 2. Saved Search als JSON

```
ytd search list --project $PROJECT --format raw
```

**Erwartung**: Valides JSON-Array. Jeder Eintrag hat `id`, `name`, `query`.

**Merke** den Namen oder die ID der ersten Saved Search als `$SEARCH`.

### 3. Saved Search ausführen

```
ytd search run $SEARCH
```

**Erwartung**: Exit-Code 0. Gibt eine Liste von Issues zurück (kann leer sein, wenn die Query keine Treffer hat).

## Agile Boards

### 4. Boards auflisten

```
ytd board list --project $PROJECT
```

**Erwartung**: Exit-Code 0. Falls Boards vorhanden: Liste mit Name, gefiltert auf Boards die das Testprojekt enthalten.

Falls die Liste leer ist, Schritte 5-6 überspringen.

### 5. Board-Details abrufen

**Merke** die ID des ersten Boards als `$BOARD_ID`.

```
ytd board get $BOARD_ID
```

**Erwartung**: Board-Name, zugeordnete Projekte, Sprints.

### 6. Board als JSON

```
ytd board get $BOARD_ID --format raw
```

**Erwartung**: Valides JSON mit `id`, `name`, `sprints`-Array.
