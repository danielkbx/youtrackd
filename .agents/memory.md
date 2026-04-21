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
