# Project Memory

Discoveries and decisions not derivable from the code. Append new entries — never overwrite existing ones unless factually wrong.

---

## YouTrack API: Always use `?fields=` filter
Date: 2026-04-21
YouTrack returns all fields by default, which produces very large responses. Always request only needed fields via `?fields=id,summary,description,...`. This is critical for agent use cases where output goes into the context window. Apply this in every core API function from the start — it's hard to retrofit.

## YouTrack REST API Reference
Date: 2026-04-21
Official docs: https://www.jetbrains.com/help/youtrack/devportal/rest-api-reference.html

Key endpoint paths (verified from docs):
- Issues: `GET/POST /api/issues`, `GET/POST /api/issues/{issueID}`
- Issue Comments: `GET/POST /api/issues/{issueID}/comments`
- Articles: `GET/POST /api/articles`, `GET/POST /api/articles/{articleID}`
- Projects: `GET/POST /api/admin/projects`, `GET/POST /api/admin/projects/{projectID}`
- Current User: `GET /api/users/me` (returns `Me` entity, extends `User`)

## YouTrack API: Default $top is 42
Date: 2026-04-21
The server default for `$top` (max results) is 42 entries, not 100. Always set `$top` explicitly when listing resources to get predictable results. Use `$top=100` or higher as needed.

## YouTrack API: Article project is read-only after creation
Date: 2026-04-21
The Article entity's `project` field is marked read-only. It can be set during creation (POST /api/articles) but not changed afterward. The POST body for article creation requires `{ project: { id: "..." }, summary: "...", content: "..." }`.

## YouTrack API: Issue creation requires project database ID
Date: 2026-04-21
POST /api/issues requires `{ project: { id: "<databaseID>" }, summary: "..." }`. The `id` here is the internal database ID, not the shortName. Either resolve shortName → ID first via GET /api/admin/projects/{shortName}?fields=id, or test if the API also accepts shortName in the id field.

## YouTrack API: Articles support query parameter for search
Date: 2026-04-21
GET /api/articles accepts a `query` parameter, same as issues. No separate search endpoint needed. Filter by project using YouTrack search syntax in the query string.

## Implementierung: Route-Validierung vor Config-Laden
Date: 2026-04-21
In `src/cli/index.ts` muss die Validierung des Kommandos (ob es überhaupt existiert) VOR dem Laden der Config via `getConfig()` passieren. Sonst bekommt der Nutzer bei einem Tippfehler "Not logged in" statt "Unknown command" — irreführend und schwer zu debuggen. Lösung: `KNOWN`-Map im Entry-Point, die alle gültigen resource/action-Kombinationen kennt.

## Implementierung: fetch-Mock in Bun-Tests
Date: 2026-04-21
Bun's `typeof fetch` erfordert das `preconnect`-Property, das normale Mock-Funktionen nicht haben. Lösung: `as unknown as typeof fetch` beim Zuweisen von `globalThis.fetch` in Tests. Außerdem: `RequestInfo` ist in Bun nicht global verfügbar — stattdessen `string | URL | Request` verwenden.

## Implementierung: Bun + pnpm PATH-Problem
Date: 2026-04-21
Im Claude-Agent-Kontext sind Shell-Profile nicht geladen. Bun liegt in `~/.bun/bin/`, Node/pnpm benötigen nvm. Alle CLI-Befehle müssen mit:
```bash
export NVM_DIR="$HOME/.nvm" && source "$NVM_DIR/nvm.sh" && export PATH="$HOME/.bun/bin:$PATH"
```
…präfixiert werden. Alternativ: In `.agents/process.md` dokumentiert.

## YouTrack API: Issue comment POST requires only text field
Date: 2026-04-21
POST /api/issues/{issueID}/comments — required body: `{ text: "..." }`. Optional: visibility object. The muteUpdateNotifications query param suppresses notifications.

## Implementierung: Visibility-Group-Auflösung und Clear-Verhalten
Date: 2026-04-22
Verifiziert durch Code und Tests: Die Auflösung für Ticket/Article-Create+Update ist `--visibility-group` → `YTD_VISIBILITY_GROUP` → gespeicherter `config visibility-group`-Wert. `--no-visibility-group` überschreibt geerbte Defaults. Bei `update` wird dann ein `LimitedVisibility`-Payload mit leerem `permittedGroups` gesendet (Clear). Bei `create` wird das `visibility`-Feld stattdessen komplett weggelassen. Die Kombination `--visibility-group` + `--no-visibility-group` ist ein Input-Fehler.

## YouTrack API: Comment operations are parent-scoped
Date: 2026-04-24
Specific comment get/update/delete operations require the parent resource path: `/api/issues/{issueID}/comments/{commentID}` or `/api/articles/{articleID}/comments/{commentID}`. `ytd` therefore exposes encoded comment IDs as `<ticket-id>:<comment-id>` and `<article-id>:<comment-id>`. Parent type is inferred from ID shape: article IDs use `<PROJECT>-A-<NUMBER>`, ticket IDs use `<PROJECT>-<NUMBER>`. The public `id` field for any CLI comment output must always be encoded; raw YouTrack comment IDs may only appear as `ytId`.

## Implementierung: Comment visibility semantics
Date: 2026-04-24
New comments (`ticket comment`, `article comment`) apply the configured visibility default from `--visibility-group`, `YTD_VISIBILITY_GROUP`, or stored `config visibility-group`. `--no-visibility-group` suppresses inherited defaults for comment creation. Existing comment updates (`comment update`) intentionally ignore env/config defaults unless an explicit visibility flag is present. `comment update --visibility-group <group>` sets comment visibility; `comment update --no-visibility-group` clears it with an empty `permittedGroups` payload.

## YouTrack API: Existing comment attachment upload unsupported via comment update
Date: 2026-04-24
Curl probe in DWP showed that updating an existing issue comment with `attachments:[{id}]` returns HTTP 200 but does not assign the attachment to the comment. The attachment remains a parent issue attachment with `comment: null`, and the comment response returns `attachments: []`. `ytd` does not implement `comment attach` until a supported API flow is found. Follow-up ticket: DWP-24.

## Implementierung: Attachment IDs are parent-scoped
Date: 2026-04-24
YouTrack attachment get/delete endpoints are scoped to tickets or articles, even when an attachment belongs to a comment. `ytd` therefore exposes encoded attachment IDs as `<ticket-id>:<attachment-id>` and `<article-id>:<attachment-id>`. The public `id` field for any CLI attachment output must always be encoded; raw YouTrack attachment IDs may only appear as `ytId`. Comment-owned attachments include an encoded `commentId` when YouTrack reports `comment(id)`.

## Implementierung: Sprint IDs are board-scoped
Date: 2026-04-24
YouTrack sprint get/update/delete endpoints are scoped under Agile boards: `/api/agiles/{agileID}/sprints/{sprintID}`. `ytd` therefore exposes sprint IDs as `<board-id>:<sprint-id>`. The public `id` field for CLI sprint output must always use this sprint-id, while the raw YouTrack sprint ID may only appear as `ytId`. `current` is intentionally not accepted as a sprint-id; use `ytd sprint current` or `ytd sprint current --board <board-id>` to resolve current sprints to real IDs.
